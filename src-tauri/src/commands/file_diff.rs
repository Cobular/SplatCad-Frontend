use core::iter::Peekable;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{commands::types::LocalFileData, error::Result};

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

pub fn find_diffs<I>(
    mut left_iter: Peekable<I>,
    mut right_iter: Peekable<I>,
) -> Result<Vec<FileDiff>>
where
    I: Iterator<Item = TreeItem> + ExactSizeIterator,
{
    let mut file_diffs: Vec<FileDiff> = Vec::with_capacity(left_iter.len().max(right_iter.len()));

    while let (Some(left), Some(right)) = (left_iter.peek(), right_iter.peek()) {
        // Compare the keys
        match (left, right) {
            // Right is smaller, so that means this exists only there
            ((left_path, _), (right_path, right_data)) if left_path > right_path => {
                // Remote is smaller, so that means this exists only there
                file_diffs.push(FileDiff::right_create(
                    right_path.clone(),
                    right_data.clone(),
                ));
                right_iter.next();
                continue;
            }
            // Left is smallser, so this exists only there
            ((left_path, left_data), (right_path, _)) if left_path < right_path => {
                // Local is smaller
                file_diffs.push(FileDiff::left_create(left_path.clone(), left_data.clone()));
                left_iter.next();
                continue;
            }
            // If their keys are the same & their hashes are different,
            //  check their metadata to see who is newer
            // Figure out which it is,  advance both iterators and continue
            ((left_path, left_data), (right_path, right_data))
                if left_path == right_path && left_data.hash != right_data.hash =>
            {
                // Figure out which is newer
                match (&left_data.metadata, &right_data.metadata) {
                    // Local time is greater (left is newer)
                    (left_metadata, right_metadata)
                        if left_metadata.modified > right_metadata.modified =>
                    {
                        file_diffs.push(FileDiff::left_newer(
                            left_path.clone(),
                            left_data.clone(),
                            right_data.clone(),
                        ));
                    }
                    // Remote time is greater (right is newer)
                    (left_metadata, right_metadata)
                        if left_metadata.modified < right_metadata.modified =>
                    {
                        file_diffs.push(FileDiff::right_newer(
                            right_path.clone(),
                            left_data.clone(),
                            right_data.clone(),
                        ));
                    }
                    // Both times are equal - merge conflict
                    (_, _) => {
                        return Err(
                            "Found merge conflict - same key, different hash, same modified time"
                                .to_owned()
                                .into(),
                        )
                    }
                };
                left_iter.next();
                right_iter.next();
                continue;
            }
            // Keys are the same and hashes are the same, nothing happens.
            (left, right) => {
                assert_eq!(left.0, right.0);
                assert_eq!(left.1.hash, right.1.hash);
                println!("Same file name and same hash");
                left_iter.next();
                right_iter.next();
                continue;
            }
        };
    }

    // Clean up residuals
    file_diffs.extend(left_iter.map(|(path, data)| FileDiff::left_create(path, data)));
    file_diffs.extend(right_iter.map(|(path, data)| FileDiff::right_create(path, data)));

    Ok(file_diffs)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{TimeZone, Utc};

    use crate::commands::{
        file_diff::{find_diffs, FileDiff, TreeItem},
        types::{LocalFileData, LocalFileMetadata},
    };

    #[test]
    fn test_merge_no_diff() {
        let left: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/both"),
            LocalFileData {
                hash: 0,
                name: "eee".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/both"),
                    modified: Utc.timestamp(100, 0),
                    size: 1,
                    update_time: Utc.timestamp(100, 0),
                },
            },
        )];
        let right: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/both"),
            LocalFileData {
                hash: 0,
                name: "eee".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/both"),
                    modified: Utc.timestamp(100, 0),
                    size: 1,
                    update_time: Utc.timestamp(100, 0),
                },
            },
        )];
        let mut expected: Vec<FileDiff> = vec![];

        let mut res =
            find_diffs(left.into_iter().peekable(), right.into_iter().peekable()).unwrap();

        expected.sort();
        res.sort();

        assert_eq!(expected, res);
    }

    #[test]
    fn test_both_new() {
        let left: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/left"),
            LocalFileData {
                hash: 0,
                name: "eee".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/left"),
                    modified: Utc.timestamp(100, 0),
                    size: 1,
                    update_time: Utc.timestamp(100, 0),
                },
            },
        )];
        let right: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/right"),
            LocalFileData {
                hash: 2,
                name: "ee2".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/right"),
                    modified: Utc.timestamp(100, 0),
                    size: 1,
                    update_time: Utc.timestamp(100, 0),
                },
            },
        )];
        let mut expected: Vec<FileDiff> = vec![
            FileDiff::left_create(
                PathBuf::from("/this/is/in/left"),
                LocalFileData {
                    hash: 0,
                    name: "eee".to_owned(),
                    metadata: LocalFileMetadata {
                        path: PathBuf::from("/this/is/in/left"),
                        modified: Utc.timestamp(100, 0),
                        size: 1,
                        update_time: Utc.timestamp(100, 0),
                    },
                },
            ),
            FileDiff::right_create(
                PathBuf::from("/this/is/in/right"),
                LocalFileData {
                    hash: 2,
                    name: "ee2".to_owned(),
                    metadata: LocalFileMetadata {
                        path: PathBuf::from("/this/is/in/right"),
                        modified: Utc.timestamp(100, 0),
                        size: 1,
                        update_time: Utc.timestamp(100, 0),
                    },
                },
            ),
        ];

        let mut res =
            find_diffs(left.into_iter().peekable(), right.into_iter().peekable()).unwrap();

        expected.sort();
        res.sort();

        assert_eq!(expected, res);
    }

    #[test]
    fn test_right_newer() {
        let left: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/both"),
            LocalFileData {
                hash: 0,
                name: "eee".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/both"),
                    modified: Utc.timestamp(100, 0),
                    size: 1,
                    update_time: Utc::now(),
                },
            },
        )];
        let right: Vec<TreeItem> = vec![(
            PathBuf::from("/this/is/in/both"),
            LocalFileData {
                hash: 2,
                name: "eee".to_owned(),
                metadata: LocalFileMetadata {
                    path: PathBuf::from("/this/is/in/both"),
                    modified: Utc.timestamp(102, 0),
                    size: 1,
                    update_time: Utc::now(),
                },
            },
        )];
        let mut expected: Vec<FileDiff> = vec![FileDiff {
            path: PathBuf::from("/this/is/in/both"),
            diff_type: crate::commands::file_diff::DiffTypes::RightNewer,
            diff_metadata: crate::commands::file_diff::FileDiffData::Both(
                LocalFileData {
                    hash: 0,
                    name: "eee".to_owned(),
                    metadata: LocalFileMetadata {
                        path: PathBuf::from("/this/is/in/both"),
                        modified: Utc.timestamp(100, 0),
                        size: 1,
                        update_time: Utc::now(),
                    },
                },
                LocalFileData {
                    hash: 2,
                    name: "eee".to_owned(),
                    metadata: LocalFileMetadata {
                        path: PathBuf::from("/this/is/in/both"),
                        modified: Utc.timestamp(102, 0),
                        size: 1,
                        update_time: Utc::now(),
                    },
                },
            ),
        }];

        let mut res =
            find_diffs(left.into_iter().peekable(), right.into_iter().peekable()).unwrap();

        expected.sort();
        res.sort();

        assert_eq!(expected, res);
    }
}
