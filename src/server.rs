use crate::commands;
use crate::resp::{self, RespError, RespValue};
use crate::store::Store;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
                    let args = commands::extract_args(value);

                    let response = commands::execute(args, Arc::clone(&store));

                    socket.write_all(&response.serialize()).await?;

                    consumed += n;
                }

                Err(RespError::Incomplete) => {
                    break;
                }

                Err(e) => {
                    let response = RespValue::err(format!("{}", e));

                    socket.write_all(&response.serialize()).await?;
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


