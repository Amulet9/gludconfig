use std::ops::{Deref, DerefMut};

use crate::{error::StorageError, storage_backend::StorageBackend};

pub struct RustQliteImpl {
    connection: async_rusqlite::Connection,
}

impl Deref for RustQliteImpl {
    type Target = async_rusqlite::Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl DerefMut for RustQliteImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

impl RustQliteImpl {
    pub async fn connect(path: &str) -> anyhow::Result<Self> {
        let conn = async_rusqlite::Connection::open(path).await?;
        conn.call(|c| c.execute(include_str!("../../migrations/latest.sql"), ()))
            .await?;
        return Ok(Self { connection: conn });
    }
}

#[async_trait::async_trait]
impl StorageBackend for RustQliteImpl {
    async fn delete_schema(&self, name: String) -> anyhow::Result<()> {
        Ok(self
            .connection
            .call(move |conn| {
                let mut query = conn.prepare_cached("DELETE fROM schemas WHERE name = ?1")?;

                query.execute([name])?;
                Result::<_, async_rusqlite::Error>::Ok(())
            })
            .await?)
    }
    async fn fetch_all(&self) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        Ok(self
            .connection
            .call(move |conn| {
                let mut query = conn.prepare_cached("SELECT name, data FROM schemas")?;
                let rows = query
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                    })?
                    .map(|item| item.map_err(|err| async_rusqlite::Error::Rusqlite(err)))
                    .collect::<Result<Vec<(String, Vec<u8>)>, async_rusqlite::Error>>();
                rows
            })
            .await?)
    }

    async fn new_schema(&self, name: String, data: Vec<u8>) -> anyhow::Result<()> {
        self.connection
            .call(move |conn| {
                let mut statement =
                    conn.prepare_cached("INSERT INTO schemas (name, data) VALUES (?1, ?2)")?;
                statement.execute((name, data))
            })
            .await?;
        Ok(())
    }

    async fn update_schema(&self, name: String, data: Vec<u8>) -> anyhow::Result<()> {
        self.connection
            .call(move |conn| {
                let mut statement =
                    conn.prepare_cached("UPDATE schemas SET data = ?2 WHERE name = ?1")?;
                statement.execute((name, data))
            })
            .await?;
        Ok(())
    }

    async fn fetch_schema(&self, name: String) -> anyhow::Result<Vec<u8>> {
        let s = self
            .connection
            .call(move |conn| {
                let mut statement =
                    conn.prepare_cached("SELECT data FROM schemas WHERE name = ?1")?;
                let mut row = statement.query_map([name], |row| {
                    let data: Vec<u8> = row.get(0)?;
                    Ok(data)
                })?;

                Result::<_, async_rusqlite::Error>::Ok(row.next())
            })
            .await?;
        
        if let Some(s) = s {
            Ok(s?)
        } else {
            anyhow::bail!("Failed to fetch row")
        }
    }
}
