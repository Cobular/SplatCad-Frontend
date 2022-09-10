use std::path::PathBuf;

use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Serialize, Deserialize};

pub struct TreeNames;

impl TreeNames {
  // Pre-hash local metadata (versions and stuff)
  pub const BASIC_LOCAL_METADATA: &'static str = "basicMetadataLocal::>>";
  // Post-hash local metadata (the git local tree)
  pub const HASH_LOCAL_METDATA: &'static str = "metaHashLocal::>>";
  // Reference local version (the git HEAD equivalent)
  pub const HASH_HEAD_METDATA: &'static str = "metaHashHead::>>";
  // The latest remote metadata, as known locally (origin/* in git terms)
  pub const HASH_REMOTE_METDATA: &'static str = "metaHashRemote::>>";
}

#[derive(Derivative, Debug, Serialize, Deserialize, Clone)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalFileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    pub update_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalFileData {
    pub name: String,
    pub hash: u128,
    pub metadata: LocalFileMetadata,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiffTypes {
    RightCreate,
    LeftCreate,
    RightNewer,
    LeftNewer,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileDiffData {
    Left(LocalFileData),
    Right(LocalFileData),
    Both(LocalFileData, LocalFileData),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDiff {
    path: PathBuf,
    diff_metadata: FileDiffData,
    diff_type: DiffTypes,
}

impl FileDiff {
    pub fn right_create(path: PathBuf, right: LocalFileData) -> Self {
        Self {
            path,
            diff_metadata: FileDiffData::Right(right),
            diff_type: DiffTypes::RightCreate,
        }
    }

    pub fn right_newer(path: PathBuf, left: LocalFileData, right: LocalFileData) -> Self {
        Self {
            path,
            diff_metadata: FileDiffData::Both(left, right),
            diff_type: DiffTypes::RightNewer,
        }
    }

    pub fn left_create(path: PathBuf, left: LocalFileData) -> Self {
        Self {
            path,
            diff_metadata: FileDiffData::Left(left),
            diff_type: DiffTypes::LeftCreate,
        }
    }

    pub fn left_newer(path: PathBuf, left: LocalFileData, right: LocalFileData) -> Self {
        Self {
            path,
            diff_metadata: FileDiffData::Both(left, right),
            diff_type: DiffTypes::LeftNewer,
        }
    }
}

pub type TreeItem = (PathBuf, LocalFileData);

