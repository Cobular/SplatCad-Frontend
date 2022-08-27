use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use data_encoding::HEXUPPER;
use futures::{prelude::*, stream::FuturesUnordered};
use serde::Serialize;
use sha2::{Digest, Sha256};
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
pub async fn get_all_data_command(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    Ok(get_all_data(root).await.unwrap())
}

#[tauri::command]
pub async fn get_all_data(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    let mut data_map: HashMap<PathBuf, LocalFileData> = HashMap::new();
    println!("entry");
    let mut fs_iter = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;

            if !entry.metadata().ok()?.is_file() {
                return None;
            }

            let test = tokio::spawn(async move {load(entry).await});

            Some(test)
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(data) = fs_iter.next().await {
        let data = data.unwrap();
        match data {
            Ok(data) => {
                // println!("data: {:?}", data);
                data_map.insert(data.path.clone(), data);
            }
            Err(err) => println!("{}", err),
        };
        println!("{}",data_map.len());
    }

    Ok(data_map)
}

async fn load(entry: DirEntry) -> Result<LocalFileData, Error> {
    let path = entry.path();

    let mut hasher = Sha256::new();
    let bytes = read(path).await?; // Vec<u8>
    hasher.update(bytes);

    let result = hasher.finalize();

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
        let root = PathBuf::from("/Users/cobular/Documents/GrabCAD");
        let data = get_all_data(root).await.unwrap();
        println!("{:?}", data.len());
    }
}
