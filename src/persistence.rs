use crate::resp::RespValue;
use crate::store::{Store, StoreEntry};
use crate::{commands, resp};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub fn save_rdb(store: &Store, path: &str) -> Result<()> {
    let data = store.data.lock().unwrap();
    let bytes = bincode::serialize(&*data)?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn load_rdb(store: &Store, path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        return Ok(());
    }

    let bytes = fs::read(path)?;

    let snapshot: HashMap<String, StoreEntry> = bincode::deserialize(&bytes)?;

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

pub fn append_aof(path: &str, args: &[String]) -> Result<()> {
    let value = RespValue::Array(Some(args.iter().cloned().map(RespValue::bulk).collect()));
    let bytes = value.serialize();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    file.write_all(&bytes)?;
    Ok(())
}

pub fn load_aof(store: &Arc<Store>, path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        return Ok(());
    }

    let bytes = fs::read(path)?;
    let mut offset = 0;

    while offset < bytes.len() {
        match resp::parse(&bytes[offset..]) {
            Ok((value, consumed)) => {
                let args = commands::extract_args(value);

                commands::execute(args, Arc::clone(store));

                offset += consumed;
            }

            Err(_) => {
                eprintln!("Error parsing AOF file at offset {}", offset);
                break;
            }
        }
    }

    Ok(())
}
