//! The implemention for binary db backend.

use anyhow::Result;
use rusqlite::params;

use super::{DbOperation, MarsImage};

impl DbOperation for rusqlite::Connection {
    fn create_table_if_not_exist(&self, table: &str) {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS [{table}] (
                id INTEGER,
                sha BLOB NOT NULL PRIMARY KEY
            );"
        );
        self.execute(&query, []).expect("Table creation failed");
    }

    fn query_from_table(&self, table: &str, sha: &[u8]) -> Result<Option<MarsImage>> {
        let query = format!("SELECT id, sha FROM [{table}] WHERE sha = ?");
        let mut stmt = self.prepare(&query)?;
        let mut rows = stmt.query(params![sha])?;

        Ok(rows.next()?.map(|row| MarsImage {
            id: row.get(0).expect("Failed to get id from row"),
            sha: row.get(1).expect("Failed to get sha from row"),
        }))
    }

    fn insert_to_table(&self, table: &str, item: MarsImage) -> Result<()> {
        self.create_table_if_not_exist(table);
        let query = format!("INSERT INTO [{table}] (id, sha) VALUES (?1, ?2)");
        self.execute(&query, params![item.id, item.sha])?;
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
        self.execute(&query, params![])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::db::DbOperation;

    fn exist_table(table: &str, conn: &Connection) -> bool {
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
        let mut stmt = conn.prepare(query).unwrap();
        stmt.exists(params![table]).unwrap()
    }

    #[test]
    fn test_create_table_and_drop_table() {
        let conn = Connection::open_in_memory().unwrap();
        conn.create_table_if_not_exist("123456789");
        assert!(exist_table("123456789", &conn));
        conn.drop_table("123456789").unwrap();
        assert!(!exist_table("123456789", &conn));
    }

    #[test]
    fn test_insert_get() {
        let conn = Connection::open_in_memory().unwrap();
        conn.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        conn.insert_to_table("123456789", item.clone()).unwrap();
        let result = conn
            .query_from_table("123456789", &[1, 2, 3, 4, 5, 6])
            .unwrap()
            .unwrap();
        assert_eq!(result, item);
    }

    #[test]
    fn test_insert_or_get_existing() {
        let conn = Connection::open_in_memory().unwrap();
        conn.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        let result = conn.insert_or_get_existing("123456789", item).unwrap();
        assert!(result.is_none());
        let item2 = MarsImage::new(654_321, [1, 2, 3, 4, 5, 6]);
        let result = conn.insert_or_get_existing("123456789", item2).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn double_insert_should_fail() {
        let conn = Connection::open_in_memory().unwrap();
        conn.create_table_if_not_exist("123456789");
        let item = MarsImage::new(123_456, [1, 2, 3, 4, 5, 6]);
        conn.insert_to_table("123456789", item.clone()).unwrap();
        assert!(conn.insert_to_table("123456789", item).is_err());
    }
}
