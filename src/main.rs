mod resp;
mod server;
mod store;
mod commands;
mod persistence;

use anyhow::Result;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    println!("redisrs listening on 127.0.0.1:6379");

    loop {
        let (socket, addr) = listener.accept().await?;

        let _ = tokio::spawn(async move {
            // Handle the connection here
            println!("Accepted connection from {}", addr);

            let _ = socket;
        });
    }
}
