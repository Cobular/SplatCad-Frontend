use std::{path::{Path, PathBuf}, sync::Arc};

use once_cell::sync::OnceCell;
use serde_cbor::to_vec;
use tauri::{Config, api::path::app_dir};

use crate::error::Result;


static DB: OnceCell<sled::Db> = OnceCell::new();

pub fn init_db(db_path: dyn AsRef<Path>, app_path: dyn AsRef<Path>) -> Result<sled::Db> {
    let db = sled::open(db_path).unwrap();
    let prefs = db.open_tree("preferences")?;
    prefs.insert(
        b"appdata",
        to_vec(&app_path)?
    )?;
    Ok(db)
}

pub fn make_db<'a>(config: Arc<Config>) -> &'a sled::Db {
    if DB.get().is_none() {
        let app_dir = match app_dir(&*config) {
            Some(app_dir) => app_dir,
            None => PathBuf::from("./"),
        };

        let db = init_db(app_dir.join("splatcad.db"), app_dir).unwrap();

        DB.set(db).expect("Failed to create DB")
    }

    DB.get().expect("Failed to get DB")
}

pub fn get_db<'a>() -> &'a sled::Db {
    DB.get()
        .expect("Failed to get DB, be sure to call `make_db` first")
}