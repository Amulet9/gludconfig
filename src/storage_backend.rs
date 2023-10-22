use std::future::Future;

use crate::schema::Schema;



#[async_trait::async_trait]

pub trait StorageBackend {
    async fn new_schema(&mut self, name: String, data: Vec<u8>) -> anyhow::Result<()>;
    async fn update_schema(&mut self, name: String, data: Vec<u8>) -> anyhow::Result<()>;
    async fn delete_schema(&mut self, name: String) -> anyhow::Result<()>;
    async fn fetch_all(&mut self) -> anyhow::Result<Vec<(String, Vec<u8>)>>;
    async fn fetch_schema(&mut self, name: String) -> anyhow::Result<Vec<u8>>;
}
