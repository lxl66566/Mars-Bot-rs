pub mod binary;

use std::sync::{LazyLock, Mutex};

use die_exit::DieWith;

pub use crate::utils::db_path;

pub static DB: LazyLock<DbBackend> = LazyLock::new(|| {
    DbBackend::new_binary().die_with(|e| format!("Cannot attach db backend: {e:?}"))
});

pub trait DbOperation {
    fn create_table_if_not_exist(&self, table: &str);
    fn query_from_table(&self, table: &str, key: &[u8]) -> rusqlite::Result<MarsImage>;
    fn insert_to_table(&self, table: &str, item: &MarsImage) -> rusqlite::Result<usize>;
    fn insert_or_get_existing(
        &self,
        table: &str,
        item: &MarsImage,
    ) -> rusqlite::Result<Option<MarsImage>>;
    fn drop_table(&self, table: &str) -> rusqlite::Result<()>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DbBackend {
    /// The Text backend using serde to save all data in text. NOT IMPLEMENTED.
    #[allow(unused)]
    Text,
    /// The Binary backend using Sqlite as backend.
    Binary(Mutex<rusqlite::Connection>),
}

#[derive(Debug, Clone)]
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

impl DbBackend {
    /// The path is the database file path.
    pub fn new_binary() -> rusqlite::Result<Self> {
        Ok(Self::Binary(Mutex::new(rusqlite::Connection::open(
            db_path().with_extension("sqlite3"),
        )?)))
    }
}

/// Some boring impl
impl DbOperation for DbBackend {
    fn create_table_if_not_exist(&self, table: &str) {
        match self {
            Self::Binary(b) => b.lock().unwrap().create_table_if_not_exist(table),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_to_table(&self, table: &str, item: &MarsImage) -> rusqlite::Result<usize> {
        match self {
            Self::Binary(b) => b.lock().unwrap().insert_to_table(table, item),
            Self::Text => unimplemented!(),
        }
    }
    fn query_from_table(&self, table: &str, key: &[u8]) -> rusqlite::Result<MarsImage> {
        match self {
            Self::Binary(b) => b.lock().unwrap().query_from_table(table, key),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_or_get_existing(
        &self,
        table: &str,
        item: &MarsImage,
    ) -> rusqlite::Result<Option<MarsImage>> {
        match self {
            Self::Binary(b) => b.lock().unwrap().insert_or_get_existing(table, item),
            Self::Text => unimplemented!(),
        }
    }
    fn drop_table(&self, table: &str) -> rusqlite::Result<()> {
        match self {
            Self::Binary(b) => b.lock().unwrap().drop_table(table),
            Self::Text => unimplemented!(),
        }
    }
}
