mod commands;
mod persistence;
mod resp;
mod server;
mod store;

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::{persistence::load_rdb,persistence::load_aof, store::Store};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let store = Arc::new(Store::new());
    load_rdb(&store, "dump.rdb")?;
    load_aof(&store, "appendonly.aof")?;

    tokio::spawn(store::Store::active_expiry_task(Arc::clone(&store)));
    tokio::spawn(persistence::rdb_snapshot_task(Arc::clone(&store), "dump.rdb"));

    println!("redisrs listening on 127.0.0.1:6379");

    loop {
        let (socket, addr) = listener.accept().await?;

        let store = Arc::clone(&store);

        println!("Accepted connection from {}", addr);

        let _ = tokio::spawn(async move {
            if let Err(e) = server::handle_client(socket, store).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}
