mod commands;
mod persistence;
mod resp;
mod server;
mod store;

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let store = Arc::new(Store::new());

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
