#[cfg(feature = "sled")]
pub mod sled;
#[cfg(feature = "sled")]
pub use sled::*;
#[cfg(feature = "sqlite")]
pub mod sqlite;
use std::{path::Path, sync::LazyLock};

use anyhow::Result;
#[cfg(feature = "sqlite")]
pub use sqlite::*;

pub use crate::utils::db_path;

#[cfg(feature = "sqlite")]
pub static DB: LazyLock<Box<dyn DbOperation<Connection = ()> + Send + Sync>> =
    LazyLock::new(|| new_db(db_path()));

#[cfg(feature = "sled")]
pub static DB: LazyLock<Box<dyn DbOperation<Connection = sled_crate::Db> + Send + Sync>> =
    LazyLock::new(|| new_db(db_path()));

#[allow(unused)]
pub trait DbOperation {
    type Connection;
    fn create_table_if_not_exist(&self, table: &str) -> Self::Connection;
    fn query_from_table(&self, table: &str, key: &[u8]) -> Result<Option<MarsImage>>;
    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()>;
    fn exist_table(&self, table: &str) -> Result<bool>;
    /// Try to insert an item to table
    ///
    /// # Returns
    ///
    /// - If the item already exists, do not insert and return the existing one.
    /// - If the item is inserted successfully, return `None`.
    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>>;
    fn drop_table(&self, table: &str) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarsImage {
    /// the message id in a group
    pub id: i32,
    /// the sha-256 for image
    pub sha: Vec<u8>,
}

impl MarsImage {
    pub fn new(id: i32, sha: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            sha: sha.into(),
        }
    }
}

#[cfg(feature = "sqlite")]
fn new_db(path: impl AsRef<Path>) -> Box<dyn DbOperation<Connection = ()> + Send + Sync> {
    Box::new(Sqlite::new(path.as_ref()).die_with(|e| format!("Cannot attach db backend:{e:?}")))
}

#[cfg(feature = "sled")]
fn new_db(
    path: impl AsRef<Path>,
) -> Box<dyn DbOperation<Connection = sled_crate::Db> + Send + Sync> {
    Box::new(SledDb::new(path.as_ref()))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_table_and_drop_table() {
        let db = new_db(TempDir::new().unwrap());
        db.create_table_if_not_exist("123456789");
        assert!(db.exist_table("123456789").unwrap());
        db.drop_table("123456789").unwrap();
        assert!(!db.exist_table("123456789").unwrap());
    }

    #[test]
    fn test_insert_get() {
        let db = new_db(TempDir::new().unwrap());
        db.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        db.insert_to_table("123456789", item.clone()).unwrap();
        let result = db
            .query_from_table("123456789", &[1, 2, 3, 4, 5, 6])
            .unwrap()
            .unwrap();
        assert_eq!(result, item);
    }

    #[test]
    fn test_insert_or_get_existing() {
        let db = new_db(TempDir::new().unwrap());
        db.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        let result = db.insert_or_get_existing("123456789", item).unwrap();
        assert!(result.is_none());
        let item2 = MarsImage::new(654_321, [1, 2, 3, 4, 5, 6]);
        let result = db.insert_or_get_existing("123456789", item2).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn double_insert_should_fail() {
        let db = new_db(TempDir::new().unwrap());
        db.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        db.insert_to_table("123456789", item.clone()).unwrap();
        assert!(db.insert_to_table("123456789", item).is_err());
    }
}
