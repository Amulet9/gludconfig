use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use homedir::get_my_home;
use zvariant::{from_slice, to_bytes};

use crate::{
    impls::rustqlite::RustQliteImpl,
    property::Property,
    schema::{self, Schema},
    storage_backend::StorageBackend,
};

pub struct Storage {
    path: String,
    conn: RustQliteImpl,
}

impl Deref for Storage {
    type Target = async_rusqlite::Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for Storage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

impl Storage {
    pub async fn new() -> anyhow::Result<Storage> {
        let path = format!(
            "{}/.local/share/gludconfig/data.db",
            get_my_home()?
                .ok_or(anyhow::anyhow!("Failed to fetch home directory for user"))?
                .to_string_lossy()
                .to_string()
        );

        Ok(Storage {
            conn: RustQliteImpl::connect(&path).await?,
            path,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub async fn get_schema(&self, schema: String) -> zbus::fdo::Result<Schema> {
        let schema = self
            .conn
            .fetch_schema(schema)
            .await
            .map_err(into_zbus_error)?;
        let ctx = zvariant::EncodingContext::<byteorder::LE>::new_dbus(0);
        let schema: schema::Schema = from_slice(&schema, ctx).map_err(into_zbus_error)?;
        Ok(schema)
    }

    pub async fn fetch_all(&self) -> zbus::fdo::Result<Vec<Schema>> {
        self.conn
            .fetch_all()
            .await
            .map(|value| {
                value
                    .into_iter()
                    .map(|(_, schema)| {
                        let ctx = zvariant::EncodingContext::<byteorder::LE>::new_dbus(0);
                        let schema: schema::Schema = from_slice(&schema, ctx)?;
                        Ok(schema)
                    })
                    .collect::<Result<Vec<Schema>, anyhow::Error>>()
            })
            .map_err(into_zbus_error)?
            .map_err(into_zbus_error)
    }

    pub async fn new_schema(&self, schema: &Schema) -> zbus::fdo::Result<()> {
        let ctx = zvariant::EncodingContext::<byteorder::LE>::new_dbus(0);
        let bytes = to_bytes(ctx, schema).map_err(into_zbus_error)?;
        self.conn
            .new_schema(schema.name().to_string(), bytes)
            .await
            .map_err(into_zbus_error)
    }

    pub async fn update_schema(&self, schema: &Schema) -> zbus::fdo::Result<()> {
        let ctx = zvariant::EncodingContext::<byteorder::LE>::new_dbus(0);
        let bytes = to_bytes(ctx, schema).map_err(into_zbus_error)?;

        self.conn
            .update_schema(schema.name().to_string(), bytes)
            .await
            .map_err(into_zbus_error)
    }

    pub async fn delete_schema(&self, name: String) -> zbus::fdo::Result<()> {
        self.conn.delete_schema(name).await.map_err(into_zbus_error)
    }
}

pub fn into_zbus_error<T: Display>(err: T) -> zbus::fdo::Error {
    zbus::fdo::Error::Failed(format!("{}", err))
}
