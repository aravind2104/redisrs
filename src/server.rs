use crate::commands;
use crate::config::Config;
use crate::resp::{self, RespError, RespValue};
use crate::store::Store;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_client(
    mut socket: TcpStream,
    store: Arc<Store>,
    config: Arc<Config>,
) -> Result<()> {
    let requires_auth = config.password.is_some();
    let mut authenticated = !requires_auth;
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
                    consumed += n;
                    let args = commands::extract_args(value);

                    let cmd = args[0].to_uppercase();

                    if cmd == "AUTH" {
                        if args.len() < 2 {
                            let err =
                                RespValue::err("ERR wrong number of arguments for 'auth' command");
                            socket.write_all(&err.serialize()).await?;
                        } else {
                            let password = args[1].clone();
                            if let Some(expected_password) = &config.password {
                                if &password == expected_password {
                                    authenticated = true;
                                    let response = RespValue::ok();
                                    socket.write_all(&response.serialize()).await?;
                                } else {
                                    let err = RespValue::err("ERR invalid password");
                                    socket.write_all(&err.serialize()).await?;
                                }
                            } else {
                                let err = RespValue::err("ERR no password is set");
                                socket.write_all(&err.serialize()).await?;
                            }
                        }
                        continue;
                    }

                    if requires_auth && !authenticated {
                        let err = RespValue::err("NOAUTH Authentication required.");
                        socket.write_all(&err.serialize()).await?;
                        continue;
                    }

                    if cmd == "SUBSCRIBE" {
                        if args.len() < 2 {
                            let err = RespValue::err(
                                "ERR wrong number of arguments for 'subscribe' command",
                            );
                            socket.write_all(&err.serialize()).await?;
                        } else {
                            handle_subscribe(&mut socket, Arc::clone(&store), args[1].clone())
                                .await?;
                        }
                        continue;
                    }

                    let response = commands::execute(
                        args,
                        Arc::clone(&store),
                        &config.aof_path.as_deref().unwrap_or("appendonly.aof"),
                    );

                    socket.write_all(&response.serialize()).await?;
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

async fn handle_subscribe(
    socket: &mut TcpStream,
    store: Arc<Store>,
    channel: String,
) -> Result<()> {
    let mut receiver = store.subscribe(&channel);
    let mut tmp = [0u8; 1024];

    let response = RespValue::Array(Some(vec![
        RespValue::bulk("subscribe"),
        RespValue::bulk(channel.clone()),
        RespValue::Integer(1),
    ]));

    socket.write_all(&response.serialize()).await?;

    loop {
        tokio::select! {
            result = receiver.recv() => {
                match result {
                    Ok(message) => {
                        let response = RespValue::Array(Some(vec![
                            RespValue::bulk("message"),
                            RespValue::bulk(channel.clone()),
                            RespValue::bulk(message),
                        ]));
                        socket.write_all(&response.serialize()).await?;
                    }
                    Err(_) => {
                        // The sender has been dropped, which means the channel is closed.
                        break;
                    }
                }
            }
            result = socket.read(&mut tmp) => {
                match result {
                    Ok(0) => return Ok(()), // Client disconnected
                    Ok(n) => {
                       if let Ok((RespValue::Array(Some(value)), _)) = resp::parse(&tmp[..n]) {
                            let args = commands::extract_args(RespValue::Array(Some(value)));
                            let cmd = args[0].to_uppercase();
                            if cmd == "UNSUBSCRIBE" {
                                let response = RespValue::Array(Some(vec![
                                    RespValue::bulk("unsubscribe"),
                                    RespValue::bulk(channel.clone()),
                                    RespValue::Integer(0),
                                ]));
                                socket.write_all(&response.serialize()).await?;
                                return Ok(());
                            } else {
                                let err = RespValue::err("ERR only UNSUBSCRIBE command is allowed in subscription mode");
                                socket.write_all(&err.serialize()).await?;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
