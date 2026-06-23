mod resp;
mod server;
mod store;
mod commands;
mod persistence;

use anyhow::Result;
use tokio::net::TcpListener;
use std::sync::Arc;

use crate::store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let store = Arc::new(Store::new());
    
    println!("redisrs listening on 127.0.0.1:6379");

    loop {
        let (socket, addr) = listener.accept().await?;

        let store = Arc::clone(&store);
        
        let _ = tokio::spawn(async move {
            // Handle the connection here
            println!("Accepted connection from {}", addr);

            let _ = store;
            let _ = socket;
        });
    }
}
