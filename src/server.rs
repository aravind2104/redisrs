use tokio::net::TcpStream;
use std::sync::Arc;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::store::Store;
use crate::commands;
use crate::resp::{self, RespValue, RespError};

pub async fn handle_client(mut socket: TcpStream, store: Arc<Store>) -> Result<()> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];

    loop {
        let n = socket.read(&mut tmp).await?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&tmp[..n]);

        let mut consumed = 0;

        loop {
            match resp::parse(&buf[consumed..]) {
                Ok((value, n)) => {
                    let args = extract_args(value);

                    let response = commands::execute(args, Arc::clone(&store));

                    socket
                        .write_all(&response.serialize())
                        .await?;

                    consumed += n;
                }

                Err(RespError::Incomplete) => {
                    break;
                }

                Err(e) => {
                    let response = RespValue::err(format!("{}", e));

                    socket
                        .write_all(&response.serialize())
                        .await?;
                    buf.clear();
                    break;
                }
            }
        }
        if consumed > 0 {
            buf.drain(..consumed);
        }
    }
}

fn extract_args(value: RespValue) -> Vec<String> {
    match value {
        RespValue::Array(Some(items)) => items
            .into_iter()
            .filter_map(|item| match item {
                RespValue::BulkString(Some(s)) => Some(s),
                RespValue::SimpleString(s) => Some(s),
                _ => None,
            })
            .collect(),

        _ => vec![],
    }
}