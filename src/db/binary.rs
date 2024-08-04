//! The implemention for binary db backend.

use rusqlite::params;

use super::{DbOperation, MarsImage};

impl DbOperation for rusqlite::Connection {
    fn create_table_if_not_exist(&self, table: &str) {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {table} (
                id INTEGER NOT NULL,
                sha BLOB PRIMARY KEY NOT NULL
            );"
        );
        self.execute(&query, []).expect("Table creation failed");
    }

    fn query_from_table(&self, table: &str, sha: &[u8]) -> rusqlite::Result<MarsImage> {
        let query = format!("SELECT id, sha FROM {table} WHERE sha = ?");
        let mut stmt = self.prepare(&query)?;
        let mut rows = stmt.query(params![sha])?;

        if let Some(row) = rows.next()? {
            let image = MarsImage {
                id: row.get(0)?,
                sha: row.get(1)?,
            };
            Ok(image)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    fn insert_to_table(&self, table: &str, item: &MarsImage) -> rusqlite::Result<usize> {
        self.create_table_if_not_exist(table);
        let query = format!("INSERT INTO {table} (id, sha) VALUES (?1, ?2)");
        self.execute(&query, params![item.id, item.sha])
    }

    fn insert_or_get_existing(
        &self,
        table: &str,
        item: &MarsImage,
    ) -> rusqlite::Result<Option<MarsImage>> {
        let result = self.insert_to_table(table, item);
        match result {
            Ok(_) => {
                // Insert was successful
                Ok(None)
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                // SHA conflict, fetch the existing MarsImage
                Ok(Some(self.query_from_table(table, &item.sha)?))
            }
            Err(e) => Err(e),
        }
    }
    fn drop_table(&self, table: &str) -> rusqlite::Result<()> {
        let query = format!("DROP TABLE {table}");
        self.execute(&query, params![]).map(|_| ())
    }
}
