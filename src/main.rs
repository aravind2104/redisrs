mod commands;
mod config;
mod persistence;
mod resp;
mod server;
mod store;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::{persistence::load_aof, persistence::load_rdb, store::Store};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load("config.toml");
    let port = config.port.unwrap_or(6379);
    let addr = format!("127.0.0.1:{}", port);
    let aof_path = config
        .aof_path
        .clone()
        .unwrap_or_else(|| "appendonly.aof".to_string());
    let snapshot_interval = config.snapshot_interval.unwrap_or(300);
    let listener = TcpListener::bind(&addr).await?;
    let store = Arc::new(Store::new());
    let config = Arc::new(config);
    if Path::new(&aof_path).exists() {
        load_aof(&store, &aof_path)?;
    } else {
        load_rdb(&store, "dump.rdb")?;
    }

    tokio::spawn(store::Store::active_expiry_task(Arc::clone(&store)));
    tokio::spawn(persistence::rdb_snapshot_task(
        Arc::clone(&store),
        "dump.rdb",
        snapshot_interval,
    ));

    println!("redisrs listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;

        let store = Arc::clone(&store);
        let config = Arc::clone(&config);

        println!("Accepted connection from {}", addr);

        let _ = tokio::spawn(async move {
            if let Err(e) = server::handle_client(socket, store, config).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}
