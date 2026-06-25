use crate::store::{Store, StoreEntry};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub fn save_rdb(store: &Store, path: &str) -> Result<()> {
    let data = store.data.lock().unwrap();
    let bytes= bincode::serialize(&*data)?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn load_rdb(store: &Store, path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        return Ok(());
    }

    let bytes = fs::read(path)?;

    let snapshot: HashMap<String, StoreEntry> =
        bincode::deserialize(&bytes)?;

    let mut data = store.data.lock().unwrap();

    *data = snapshot;

    Ok(())
}

pub async fn rdb_snapshot_task(store: Arc<Store>, path: &'static str) {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;
        if let Err(e) = save_rdb(&store, path) {
            eprintln!("Error saving RDB snapshot: {:?}", e);
        } else {
            println!("RDB snapshot saved successfully to {}.", path);
        }
    }
}