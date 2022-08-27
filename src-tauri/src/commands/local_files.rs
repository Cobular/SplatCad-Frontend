use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use data_encoding::HEXUPPER;
use futures::prelude::*;
use serde::Serialize;
use meowhash::MeowHasher;
use tokio::fs::read;
use walkdir::{DirEntry, WalkDir};

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct LocalFileData {
    pub path: PathBuf,
    pub name: String,
    pub update_date: DateTime<Utc>,
    pub sha256: String,
    pub size: u64,
}

#[tauri::command]
pub async fn get_all_data(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    let mut data_map: HashMap<PathBuf, LocalFileData> = HashMap::new();
    println!("entry");

    let entries = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir());
    
    let mut fs_iter = futures::stream::iter(entries)
        .map(|entry| {
            tokio::spawn(load(entry))
        })
        .buffer_unordered(200);

    while let Some(Ok(Ok(data))) = fs_iter.next().await {
        data_map.insert(data.path.clone(), data);

        if data_map.len() % 200 == 0 {
            println!("{} files loaded", data_map.len());
        }
    }
    println!("{}",data_map.len());
    Ok(data_map)
}

async fn load(entry: DirEntry) -> Result<LocalFileData, Error> {
    let path = entry.path();

    let bytes = read(path).await?; // Vec<u8>
    let hash = MeowHasher::hash(&bytes);

    let result = hash.into_bytes();

    let metadata = entry.metadata()?;

    let sha = HEXUPPER.encode(&result);
    Ok(LocalFileData {
        name: entry.file_name().to_string_lossy().to_string(),
        path: path.into(),
        update_date: metadata.modified()?.into(),
        sha256: sha,
        size: metadata.len(),
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
