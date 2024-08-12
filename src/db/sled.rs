use std::{path::PathBuf, sync::Mutex};

use anyhow::Result;
use die_exit::DieWith;
use sled_crate::Db;
use uluru::LRUCache;

use super::{DbOperation, MarsImage};
use crate::utils::{FromVecU8, IntoVecU8};

#[cfg(feature = "sled")]
#[derive(Debug)]
pub struct SledDb {
    pub path: PathBuf,
    pub connection: Mutex<LRUCache<(String, Db), 50>>,
}

impl SledDb {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        std::fs::create_dir_all(&path).die_with(|e| format!("create database dir failed: {e:?}"));
        Self {
            path,
            connection: Mutex::new(LRUCache::new()),
        }
    }

    /// create a db and insert to cache
    #[inline]
    pub fn connect(&self, table: &str) {
        let conn = sled_crate::open(self.path.join(table))
            .die_with(|e| format!("open sled db failed: {e:?}"));
        self.connection
            .lock()
            .unwrap()
            .insert((table.to_owned(), conn));
    }

    #[inline]
    pub fn get_table(&self, table: &str) -> Option<Db> {
        self.connection
            .lock()
            .unwrap()
            .find(|x| x.0 == table)
            .map(|x| x.1.clone())
    }
}

impl DbOperation for SledDb {
    type Connection = Db;
    fn create_table_if_not_exist(&self, table: &str) -> Self::Connection {
        self.get_table(table).map_or_else(
            || {
                self.connect(table);
                self.get_table(table)
                    .expect("table must exist after connect")
            },
            |temp| temp,
        )
    }

    fn query_from_table(&self, table: &str, key: &[u8]) -> Result<Option<MarsImage>> {
        let db = self.get_table(table);
        if let Some(db) = db {
            Ok(db
                .get(key)?
                .map(|x| i32::from_vec_u8(x.as_ref()))
                .map(|x| MarsImage::new(x, key)))
        } else {
            Ok(None)
        }
    }

    /// This function will return Ok even if the key has already existed
    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()> {
        let db = self.create_table_if_not_exist(table);
        let _value = db.insert(item.sha.clone(), item.id.into_vec_u8())?;
        Ok(())
    }

    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>> {
        let db = self.create_table_if_not_exist(table);
        let exists = db.get(item.sha.clone())?;
        if let Some(id) = exists {
            return Ok(Some(MarsImage::new(i32::from_vec_u8(&id), item.sha)));
        }
        let value = db.insert(item.sha.clone(), item.id.into_vec_u8())?;
        debug_assert!(value.is_none());
        Ok(None)
    }

    fn drop_table(&self, table: &str) -> Result<()> {
        std::fs::remove_dir_all(self.path.join(table))?;
        Ok(())
    }

    fn exist_table(&self, table: &str) -> Result<bool> {
        Ok(std::fs::exists(self.path.join(table))?)
    }
}
