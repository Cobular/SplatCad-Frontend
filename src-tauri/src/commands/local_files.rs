use std::{collections::HashMap, fs::File, os::unix::prelude::MetadataExt, path::PathBuf};

use chrono::{DateTime, Utc};
use data_encoding::HEXUPPER;
use futures::future::join_all;
use ring::digest::{Context, Digest as RingDigest, SHA256};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use tauri::async_runtime::spawn_blocking;
use tokio::fs::read;
use walkdir::{DirEntry, WalkDir};

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct LocalFileData {
    pub path: PathBuf,
    pub name: String,
    pub update_date: DateTime<Utc>,
    pub sha256: String,
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<RingDigest, Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

#[tauri::command]
pub async fn get_all_data_command(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    Ok(get_all_data(root).await.unwrap())
}

#[tauri::command]
pub async fn get_all_data(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    let mut data_map: HashMap<PathBuf, LocalFileData> = HashMap::new();

    let fs_iter = WalkDir::new(root).into_iter().filter_map(|entry| {
        let entry = entry.ok()?;

        if !entry.metadata().ok()?.is_file() {
            return None;
        }

        Some(load(entry))
    });

    let hashes = join_all(fs_iter).await.into_iter().filter_map(|res| res.ok());

    for data in hashes {
        data_map.insert(data.path.clone(), data);
    }

    Ok(data_map)
}

fn buffered_load(entry: &DirEntry) -> Result<LocalFileData, Error> {
    let path = entry.path();
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;
    let sha = HEXUPPER.encode(digest.as_ref());
    Ok(LocalFileData {
        name: entry.file_name().to_string_lossy().to_string(),
        path: path.into(),
        update_date: entry.metadata()?.modified()?.into(),
        sha256: sha,
    })
}

async fn load(entry: DirEntry) -> Result<LocalFileData, Error> {
    let path = entry.path();

    let mut hasher = Sha256::new();
    let bytes = read(path).await?; // Vec<u8>
    hasher.update(bytes);

    let result = hasher.finalize();

    let sha = HEXUPPER.encode(&result);
    Ok(LocalFileData {
        name: entry.file_name().to_string_lossy().to_string(),
        path: path.into(),
        update_date: entry.metadata()?.modified()?.into(),
        sha256: sha,
    })
}
