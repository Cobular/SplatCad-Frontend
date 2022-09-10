use std::path::Path;

use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use crate::{db::types::{LocalFileMetadata, TreeItem}, error::Result};


pub fn get_metadatas(root: dyn AsRef<Path>) -> Result<impl Iterator<Item = TreeItem>> {
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