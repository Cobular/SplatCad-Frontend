use std::path::PathBuf;

use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Serialize, Deserialize};



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

pub struct TreeNames;

impl TreeNames {
  pub const BASIC_LOCAL_METADATA: &'static str = "basicMetadataLocal::>>";
  pub const HASH_LOCAL_METDATA: &'static str = "metaHashLocal::>>";
  pub const HASH_HEAD_METDATA: &'static str = "metaHashHead::>>";
  pub const HASH_REMOTE_METDATA: &'static str = "metaHashRemote::>>";
}