pub mod sleddb;
pub mod sqlite;

use std::sync::{LazyLock, Mutex};

use anyhow::Result;

pub use crate::utils::db_path;

pub static DB: LazyLock<DbBackend> = LazyLock::new(|| {
    // DbBackend::new_sqlite().die_with(|e| format!("Cannot attach db backend:
    // {e:?}"))
    DbBackend::new_sled()
});

pub trait DbOperation {
    fn create_table_if_not_exist(&self, table: &str);
    fn query_from_table(&self, table: &str, key: &[u8]) -> Result<Option<MarsImage>>;
    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()>;
    /// Try to insert an item to table
    ///
    /// # Returns
    ///
    /// - If the item already exists, do not insert and return the existing one.
    /// - If the item is inserted successfully, return `None`.
    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>>;
    fn drop_table(&self, table: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct SledDb;

#[derive(Debug)]
#[non_exhaustive]
pub enum DbBackend {
    /// The Text backend using serde to save all data in text. NOT IMPLEMENTED.
    #[allow(unused)]
    Text,
    /// The Binary backend using Sqlite as backend.
    Sqlite(Mutex<rusqlite::Connection>),
    /// `RocksDB`
    Sled(SledDb),
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

/// The path is the database file path.
impl DbBackend {
    pub fn new_sqlite() -> rusqlite::Result<Self> {
        Ok(Self::Sqlite(Mutex::new(rusqlite::Connection::open(
            db_path().with_extension("sqlite3"),
        )?)))
    }
    pub const fn new_sled() -> Self {
        Self::Sled(SledDb)
    }
}

/// Some boring impl
impl DbOperation for DbBackend {
    fn create_table_if_not_exist(&self, table: &str) {
        match self {
            Self::Sqlite(b) => b.lock().unwrap().create_table_if_not_exist(table),
            Self::Sled(b) => b.create_table_if_not_exist(table),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()> {
        match self {
            Self::Sqlite(b) => b.lock().unwrap().insert_to_table(table, item),
            Self::Sled(b) => b.insert_to_table(table, item),
            Self::Text => unimplemented!(),
        }
    }
    fn query_from_table(&self, table: &str, key: &[u8]) -> Result<Option<MarsImage>> {
        match self {
            Self::Sqlite(b) => b.lock().unwrap().query_from_table(table, key),
            Self::Sled(b) => b.query_from_table(table, key),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>> {
        match self {
            Self::Sqlite(b) => b.lock().unwrap().insert_or_get_existing(table, item),
            Self::Sled(b) => b.insert_or_get_existing(table, item),
            Self::Text => unimplemented!(),
        }
    }
    fn drop_table(&self, table: &str) -> Result<()> {
        match self {
            Self::Sqlite(b) => b.lock().unwrap().drop_table(table),
            Self::Sled(b) => b.drop_table(table),
            Self::Text => unimplemented!(),
        }
    }
}
