//! The implemention for binary db backend.

use std::{path::Path, sync::Mutex};

use anyhow::Result;
use rusqlite::params;

use super::{DbOperation, MarsImage};

pub struct Sqlite {
    pub inner: Mutex<rusqlite::Connection>,
}

impl Sqlite {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: Mutex::new(rusqlite::Connection::open(path)?),
        })
    }

    pub fn new_memory() -> Self {
        Self {
            inner: Mutex::new(
                rusqlite::Connection::open_in_memory().expect("open in memory should success"),
            ),
        }
    }
}

impl DbOperation for Sqlite {
    type Connection = ();
    fn create_table_if_not_exist(&self, table: &str) {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS [{table}] (
                id INTEGER,
                sha BLOB NOT NULL PRIMARY KEY
            );"
        );
        self.inner
            .lock()
            .unwrap()
            .execute(&query, [])
            .expect("Table creation failed");
    }

    fn query_from_table(&self, table: &str, sha: &[u8]) -> Result<Option<MarsImage>> {
        let query = format!("SELECT id, sha FROM [{table}] WHERE sha = ?");
        let lock = self.inner.lock().unwrap();
        let mut stmt = lock.prepare(&query)?;
        let mut rows = stmt.query(params![sha])?;

        Ok(rows.next()?.map(|row| MarsImage {
            id: row.get(0).expect("Failed to get id from row"),
            sha: row.get(1).expect("Failed to get sha from row"),
        }))
    }

    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()> {
        self.create_table_if_not_exist(table);
        let query = format!("INSERT INTO [{table}] (id, sha) VALUES (?1, ?2)");
        self.inner
            .lock()
            .unwrap()
            .execute(&query, params![item.id, item.sha])?;
        Ok(())
    }

    fn insert_or_get_existing(&self, table: &str, item: MarsImage) -> Result<Option<MarsImage>> {
        let sha = item.sha.clone();
        let result = self.insert_to_table(table, item);
        match result.map_err(|e| {
            e.downcast::<rusqlite::Error>()
                .expect("the error cast should be success.")
        }) {
            Ok(()) => {
                // Insert was successful
                Ok(None)
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                // SHA conflict, fetch the existing MarsImage
                self.query_from_table(table, &sha)
            }
            Err(e) => Err(e)?,
        }
    }

    fn drop_table(&self, table: &str) -> Result<()> {
        let query = format!("DROP TABLE [{table}]");
        self.inner.lock().unwrap().execute(&query, params![])?;
        Ok(())
    }

    fn exist_table(&self, table: &str) -> Result<bool> {
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        let lock = self.inner.lock().unwrap();
        let mut stmt = lock.prepare(query)?;
        Ok(stmt.exists(params![table])?)
    }
}
