use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use futures::prelude::*;
use meowhash::MeowHasher;
use serde::Serialize;
use tokio::fs::read;
use walkdir::WalkDir;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct LocalFileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LocalFileData {
    pub name: String,
    pub hash: u128,
    pub metadata: LocalFileMetadata,
}

pub fn get_metadatas(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileMetadata>, Error> {
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
        };
        Some((metadata.path.clone(), metadata))
    });

    let hashmap = entries.collect::<HashMap<_, _>>();

    Ok(hashmap)
}

#[tauri::command]
pub async fn get_all_data(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    let mut data_map: HashMap<PathBuf, LocalFileData> = HashMap::new();
    println!("entry");

    let metadata = get_metadatas(root)?;

    let mut fs_iter = futures::stream::iter(metadata)
        .map(|(_, metadata)| tokio::spawn(load(metadata)))
        .buffer_unordered(200);

    while let Some(Ok(Ok(data))) = fs_iter.next().await {
        data_map.insert(data.metadata.path.clone(), data);

        if data_map.len() % 200 == 0 {
            println!("{} files loaded", data_map.len());
        }
    }
    println!("{}", data_map.len());
    Ok(data_map)
}

async fn load(metadata: LocalFileMetadata) -> Result<LocalFileData, Error> {
    let bytes = read(&metadata.path).await?; // Vec<u8>
    let hash = MeowHasher::hash(&bytes);

    let result = hash.as_u128();

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
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_all_data() {
        println!("eeeee");
        let root = PathBuf::from(r#"C:\Users\jdc10\Documents\GrabCAD"#);
        let data = get_all_data(root).await.unwrap();
        println!("{:?}", data.len());
    }
}
