pub mod binary;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
};

pub use crate::utils::db_path;

pub trait DbOperation {
    fn create_table_if_not_exist(&self, table: &str);
    fn query_from_table(&self, table: &str, key: &[u8]) -> rusqlite::Result<MarsImage>;
    fn insert_to_table(&self, table: &str, id: i32, sha: &[u8]) -> rusqlite::Result<usize>;
    fn insert_or_get_existing(
        &self,
        table: &str,
        id: i32,
        sha: &[u8],
    ) -> rusqlite::Result<Option<MarsImage>>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DbBackend {
    /// The Text backend using serde to save all data in text. NOT IMPLEMENTED.
    #[allow(unused)]
    Text,
    /// The Binary backend using Sqlite as backend.
    Binary(RwLock<rusqlite::Connection>),
}

#[derive(Debug, Clone)]
pub struct MarsImage {
    /// the message id in a group
    pub id: i32,
    /// the sha-256 for image
    pub sha: Vec<u8>,
}

impl DbBackend {
    /// The path is the database file path.
    pub fn new_binary() -> rusqlite::Result<Self> {
        Ok(Self::Binary(RwLock::new(rusqlite::Connection::open(
            db_path().with_extension("sqlite3"),
        )?)))
    }
}

/// Some boring impl
impl DbOperation for DbBackend {
    fn create_table_if_not_exist(&self, table: &str) {
        match self {
            Self::Binary(b) => b.write().unwrap().create_table_if_not_exist(table),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_to_table(&self, table: &str, id: i32, sha: &[u8]) -> rusqlite::Result<usize> {
        match self {
            Self::Binary(b) => b.write().unwrap().insert_to_table(table, id, sha),
            Self::Text => unimplemented!(),
        }
    }
    fn query_from_table(&self, table: &str, key: &[u8]) -> rusqlite::Result<MarsImage> {
        match self {
            Self::Binary(b) => b.read().unwrap().query_from_table(table, key),
            Self::Text => unimplemented!(),
        }
    }
    fn insert_or_get_existing(
        &self,
        table: &str,
        id: i32,
        sha: &[u8],
    ) -> rusqlite::Result<Option<MarsImage>> {
        match self {
            Self::Binary(b) => b.write().unwrap().insert_or_get_existing(table, id, sha),
            Self::Text => unimplemented!(),
        }
    }
}
