use std::{collections::HashMap, fs::File, path::PathBuf};

use chrono::{DateTime, Utc};
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use serde::Serialize;
use std::io::{BufReader, Read};
use tauri::async_runtime::spawn_blocking;
use walkdir::WalkDir;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct LocalFileData {
    pub path: PathBuf,
    pub name: String,
    pub update_date: DateTime<Utc>,
    pub sha256: String,
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, Error> {
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
    let res = spawn_blocking(|| get_all_data(root))
        .await
        .unwrap();
    res
}

#[tauri::command]
pub fn get_all_data(root: PathBuf) -> Result<HashMap<PathBuf, LocalFileData>, Error> {
    let mut data_map: HashMap<PathBuf, LocalFileData> = HashMap::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;

        let path = entry.path();

        if let Ok(input) = File::open(path) {
            let reader = BufReader::new(input);
            let digest = sha256_digest(reader)?;
            let sha = HEXUPPER.encode(digest.as_ref());
            let data = LocalFileData {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.into(),
                update_date: entry.metadata()?.modified()?.into(),
                sha256: sha,
            };

            data_map.insert(path.into(), data);
        }
    }
    
    Ok(data_map)
}
