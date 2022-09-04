use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde_cbor::{from_slice, to_vec};
use sled::Tree;
use tauri::State;
use tokio::fs::read;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    commands::{types::{TreeNames, LocalFileData, LocalFileMetadata}, file_diff::{find_diffs, FileDiff}},
    error::Result,
};

pub fn get_metadatas(root: &PathBuf) -> Result<impl Iterator<Item = (PathBuf, LocalFileMetadata)>> {
    let entries = WalkDir::new(root).into_iter().filter_map(|entry| {
        let entry = entry.ok()?;
        if !entry.file_type().is_file() {
            return None;
        }

        let path = entry.path().to_path_buf();
        let size = entry.metadata().ok()?.len();
        let modified = entry.metadata().ok()?.modified().ok()?;
        let modified = DateTime::from(modified);
        let metadata = LocalFileMetadata {
            path,
            size,
            modified,
            update_time: Utc::now(),
        };
        Some((metadata.path.clone(), metadata))
    });

    Ok(entries)
}

// There si room to later make this zero copy (Deserialize<'a> vs DeserializeOwned)
fn sort_tree_keys<K, V>(tree: &Tree) -> Result<Vec<(K, V)>>
where
    K: DeserializeOwned + Ord + Clone,
    V: DeserializeOwned,
{
    let mut decoded_keypair = tree
        .iter()
        .filter_map(|item| item.ok())
        .map(|(key, value)| {

            let key_parsed: K = from_slice(&key)?;
            let val_parsed: V = from_slice(&value)?;

            Ok((key_parsed, val_parsed)) as Result<(K, V)>
        })
        .filter_map(|item| match item {
            Ok(item) => Some(item),
            Err(err) => {
                println!("{}", err);
                None
            }
        })
        .collect::<Vec<_>>();
    
    decoded_keypair.sort_by_key(|key| key.0.clone());

    Ok(decoded_keypair)
}

#[tauri::command]
pub async fn get_local_to_head_diff(
    root: PathBuf,
    db: State<'_, sled::Db>,
) -> Result<Vec<FileDiff>> {
    let local_tree_name =
        TreeNames::HASH_LOCAL_METDATA.to_owned() + root.to_string_lossy().as_ref();
    let local_tree = db.open_tree(local_tree_name)?;

    let remote_tree_name =
        TreeNames::HASH_REMOTE_METDATA.to_owned() + root.to_string_lossy().as_ref();
    let remote_tree = db.open_tree(remote_tree_name)?;

    // Need to sort both then itterate over both, doing the <> thing
    let sorted_local_iter = sort_tree_keys::<PathBuf, LocalFileData>(&local_tree)?
        .into_iter()
        .peekable();
    let sorted_remote_iter = sort_tree_keys::<PathBuf, LocalFileData>(&remote_tree)?
        .into_iter()
        .peekable();

    find_diffs(sorted_local_iter, sorted_remote_iter)
}

#[tauri::command]
pub async fn update_remote_state(
    root: PathBuf,
    remote_state: Vec<LocalFileData>,
    db: State<'_, sled::Db>,
) -> Result<()> {
    let remote_tree_name =
        TreeNames::HASH_REMOTE_METDATA.to_owned() + root.to_string_lossy().as_ref();
    let remote_tree = db.open_tree(remote_tree_name)?;

    remote_tree.clear()?;

    for item in remote_state {
        let key = to_vec(&item.metadata.path)?;
        let value = to_vec(&item)?;

        remote_tree.insert(key, value)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn update_local_state(root: PathBuf, db: State<'_, sled::Db>) -> Result<()> {
    let update_start_time = Utc::now();

    // Get metadata for all files
    let metadata = get_metadatas(&root)?;

    // Create the tree
    let basic_metadata_tree_name =
        TreeNames::BASIC_LOCAL_METADATA.to_owned() + root.to_string_lossy().as_ref();
    let meta_hash_tree_name =
        TreeNames::HASH_LOCAL_METDATA.to_owned() + root.to_string_lossy().as_ref();
    // Metadata tree always has LocalFileMetadata in it
    let metadata_tree = db.open_tree(basic_metadata_tree_name)?;
    // Hash tree always has LocalFileData in it (the superset)
    let meta_hash_tree = db.open_tree(meta_hash_tree_name)?;

    // Check metadata for each file, seeing if it needs to be changed
    let files_to_rehash = metadata
        .into_iter()
        .filter_map(|(path, metadatum)| {
            let key = to_vec(&path).ok()?;
            let value = to_vec(&metadatum).ok()?;

            let prev_val = metadata_tree.insert(&key, value).ok()?;

            match prev_val {
                // If the key doesn't exist, we need to insert it then hash it
                None => Some((path, metadatum)),
                // If the key does exist, we need to check if the metadata is the same
                Some(prev_val) => {
                    let old_thing: LocalFileMetadata = match from_slice(&prev_val) {
                        Ok(old_thing) => old_thing,
                        Err(err) => {
                            println!("Error: {:?}", err);
                            if err.is_data() {
                                // If it's a data error, override and mark for rehashing because something done fucked up
                                return Some((path, metadatum));
                            } else {
                                // If it's not a data error, just ignore it I guess
                                return None;
                            }
                        }
                    };

                    if old_thing == metadatum {
                        None
                    } else {
                        Some((path, metadatum))
                    }
                }
            }
        })
        .map(|(path, metadatum)| {
            let key = to_vec(&path).unwrap();
            metadata_tree
                .insert(&key, to_vec(&metadatum).unwrap())
                .unwrap();
            (path, metadatum)
        });

    // If it does, it needs to be re-hashed
    let mut fs_iter = futures::stream::iter(files_to_rehash)
        .map(|(_, metadata)| tokio::spawn(hash_and_finalize(metadata)))
        .buffer_unordered(200)
        .enumerate();

    while let Some((count, Ok(Ok(data)))) = fs_iter.next().await {
        meta_hash_tree.insert(to_vec(&data.metadata.path)?, to_vec(&data)?)?;

        if count % 200 == 0 {
            println!("{} files re-hashed", count + 1);
        }
    }

    // Remove old items from the trees
    for item in metadata_tree.iter() {
        let (key, metadatum) = item?;

        let key_path: PathBuf = from_slice(&key)?;
        let metadatum: LocalFileMetadata = from_slice(&metadatum)?;

        if metadatum.update_time < update_start_time {
            metadata_tree.remove(&key)?;
            meta_hash_tree.remove(&key)?;
        }
    }

    metadata_tree.flush()?;
    meta_hash_tree.flush()?;

    println!(
        "Metadata: {}, Hashed: {}",
        metadata_tree.len(),
        meta_hash_tree.len()
    );
    Ok(())
}

async fn hash_and_finalize(metadata: LocalFileMetadata) -> Result<LocalFileData> {
    let bytes = read(&metadata.path).await?; // Vec<u8>
    let hash = xxh3_64(&bytes);

    let result = hash as u128;

    Ok(LocalFileData {
        name: metadata
            .path
            .file_name()
            .ok_or_else(|| "No filename".to_string())?
            .to_string_lossy()
            .to_string(),
        metadata,
        hash: result,
    })
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use sled::Db;

    use super::*;
    use std::path::PathBuf;

    #[cfg(target_os = "macos")]
    static GRABCAD_PATH: &str = "/Users/cobular/Documents/GrabCAD";
    #[cfg(target_os = "linux")]
    static GRABCAD_PATH: &str = "/Users/cobular/Documents/GrabCAD";
    #[cfg(target_os = "windows")]
    static GRABCAD_PATH: &str = r#"C:\Users\jdc10\Documents\GrabCAD"#;

    pub struct MyState<'r, T: Send + Sync + 'static>(&'r T);

    #[test]
    fn test_compare_metadata() {
        let modified = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
        let data1 = LocalFileMetadata {
            path: PathBuf::from("test"),
            size: 0,
            modified,
            update_time: Utc::now(),
        };
        let data2 = LocalFileMetadata {
            path: PathBuf::from("test"),
            size: 0,
            modified,
            update_time: Utc.ymd(2014, 11, 28).and_hms(12, 0, 9),
        };

        assert!(data1 == data2);
    }

    #[tokio::test]
    async fn test_get_local_files() {
        println!("eeeee");
        let root = PathBuf::from(GRABCAD_PATH);
        // let db = sled::Config::default().temporary(true).open().unwrap();

        let db = sled::open("test.db").unwrap();
        let state = MyState(&db);
        let state: State<Db> = unsafe { std::mem::transmute(state) };
        update_local_state(root, state).await.unwrap();
    }

    #[tokio::test]
    async fn test_merge_trees() {
        println!("eeeee");
        let root = PathBuf::from(GRABCAD_PATH);

        let db = sled::open("test.db").unwrap();
        let state = MyState(&db);
        let state: State<Db> = unsafe { std::mem::transmute(state) };

        get_file_diff(root, state).await.unwrap();
    }
}