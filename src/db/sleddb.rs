use anyhow::Result;
use die_exit::DieWith;
use log::debug;
use sled::Db;

use super::{db_path, DbOperation, MarsImage, SledDb};
use crate::utils::{FromVecU8, IntoVecU8};

#[inline]
fn connect(table: &str) -> Db {
    sled::open(db_path().join(table)).die_with(|e| format!("open sled db failed: {e:?}"))
}

impl DbOperation for SledDb {
    fn create_table_if_not_exist(&self, _table: &str) {
        debug!("we could do nothing because `sled::open` will create table.");
    }

    fn query_from_table(&self, table: &str, key: &[u8]) -> Result<Option<MarsImage>> {
        let db = connect(table);
        let value = db
            .get(key)?
            .map(|x| i32::from_vec_u8(x.as_ref()))
            .map(|x| MarsImage::new(x, key));
        Ok(value)
    }

    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()> {
        let db = connect(table);
        let _value = db.insert(item.sha.clone(), item.id.into_vec_u8())?;
        Ok(())
    }

    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>> {
        let db = connect(table);
        let exists = db.get(item.sha.clone())?;
        if let Some(sha) = exists {
            return Ok(Some(MarsImage::new(item.id, sha.to_vec())));
        }
        let value = db.insert(item.sha.clone(), item.id.into_vec_u8())?;
        debug_assert!(value.is_none());
        Ok(None)
    }

    fn drop_table(&self, table: &str) -> Result<()> {
        Ok(std::fs::remove_file(db_path().join(table))?)
    }
}
