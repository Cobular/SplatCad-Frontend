use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use derivative::Derivative;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use serde_cbor::{from_slice, to_vec};
use tauri::State;
use tokio::fs::read;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

use crate::error::Error;

#[derive(Derivative, Debug, Serialize, Deserialize)]
#[derivative(PartialEq, Eq)]
pub struct LocalFileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    #[derivative(PartialEq="ignore")]
    pub update_time: DateTime<Utc>
}

#[derive(Debug, Serialize)]
pub struct LocalFileData {
    pub name: String,
    pub hash: u128,
    pub metadata: LocalFileMetadata,
}

pub fn get_metadatas(root: &PathBuf) -> Result<impl Iterator<Item = (PathBuf, LocalFileMetadata)>, Error> {
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
            update_time: Utc::now()
        };
        Some((metadata.path.clone(), metadata))
    });

    Ok(entries)
}

#[tauri::command]
pub async fn get_file_diff(
    db: State<'_, sled::Db>,
) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    todo!()
}

#[tauri::command]
pub async fn update_remote_state(
    db: State<'_, sled::Db>,
) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    todo!()
}

#[tauri::command]
pub async fn update_local_state(root: PathBuf, db: State<'_, sled::Db>) -> Result<(), Error> {
    let update_start_time = Utc::now();

    // Get metadata for all files
    let metadata = get_metadatas(&root)?;

    // Create the tree
    let basic_metadata_tree_name = "basicMetadata::>>".to_owned() + root.to_string_lossy().as_ref();
    let hash_tree_name = "hash::>>".to_owned() + root.to_string_lossy().as_ref();
    // Metadata tree always has LocalFileMetadata in it
    let metadata_tree = db.open_tree(basic_metadata_tree_name)?;
    // Hash tree always has LocalFileData in it
    let hash_tree = db.open_tree(hash_tree_name)?;


    // Check metadata for each file, seeing if it needs to be changed
    let files_to_rehash = metadata.into_iter().filter_map(|(path, metadatum)| {
        let key = to_vec(&path).ok()?;
        let value = to_vec(&metadatum).ok()?;

        let prev_val = metadata_tree.insert(&key, value).ok()?;

        match prev_val {
            // If the key doesn't exist, we need to insert it then hash it
            None => {
                Some((path, metadatum))
            }
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
    }).map(|(path, metadatum)| {
        let key = to_vec(&path).unwrap();
        metadata_tree.insert(&key, to_vec(&metadatum).unwrap()).unwrap();
        (path, metadatum)
    });


    // If it does, it needs to be re-hashed
    let mut fs_iter = futures::stream::iter(files_to_rehash)
        .map(|(_, metadata)| tokio::spawn(load(metadata)))
        .buffer_unordered(200)
        .enumerate();

    while let Some((count, Ok(Ok(data)))) = fs_iter.next().await {
        hash_tree.insert(to_vec(&data.metadata.path)?, to_vec(&data)?)?;

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
            hash_tree.remove(&key)?;
        }
    }

    metadata_tree.flush()?;
    hash_tree.flush()?;

    println!(
        "Metadata: {}, Hashed: {}",
        metadata_tree.len(),
        hash_tree.len()
    );
    Ok(())
}

async fn load(metadata: LocalFileMetadata) -> Result<LocalFileData, Error> {
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
    async fn test_get_all_data() {
        println!("eeeee");
        let root = PathBuf::from(GRABCAD_PATH);
        // let db = sled::Config::default().temporary(true).open().unwrap();

        let db = sled::open("test.db").unwrap();
        let state = MyState(&db);
        let state: State<Db> = unsafe { std::mem::transmute(state) };
        update_local_state(root, state).await.unwrap();
    }
}
