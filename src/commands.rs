use crate::resp::RespValue;
use crate::store::Store;
use std::sync::Arc;
use std::time::Duration;

pub fn execute(args: Vec<String>, store: Arc<Store>) -> RespValue {
    if args.is_empty() {
        return RespValue::err("ERR empty command");
    }

    let cmd = args[0].to_uppercase();

    match cmd.as_str() {
        "PING" => {
            if args.len() == 1 {
                RespValue::SimpleString("PONG".into())
            } else if args.len() == 2 {
                RespValue::bulk(args[1].clone())
            } else {
                RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ))
            }
        }

        "GET" => {
            if args.len() != 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            match store.get(&args[1]) {
                Ok(Some(value)) => RespValue::bulk(value),
                Ok(None) => RespValue::null(),
                Err(_) => RespValue::err("ERR wrong kind of value"),
            }
        }

        "SET" => match args.len() {
            3 => {
                store.set(args[1].clone(), args[2].clone());
                RespValue::ok()
            }
            5 => {
                let option = args[3].to_uppercase();
                let amount = match args[4].parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => {
                        return RespValue::err(
                            format!("ERR value is not an integer or out of range for '{}' command", cmd),
                        );
                    }
                };
                let duration = match option.as_str() {
                    "EX" => Duration::from_secs(amount),
                    "PX" => Duration::from_millis(amount),
                    _ => {
                        return RespValue::err(format!(
                            "ERR syntax error for '{}' command",
                            cmd
                        ));
                    }
                };

                store.set_with_expiry(
                    args[1].clone(),
                    args[2].clone(),
                    duration
                );
                RespValue::ok()
            }
            _ => RespValue::err(format!(
                "ERR wrong number of arguments for '{}' command",
                cmd
            )),
        },

        "DEL" => {
            if args.len() < 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let count = store.del(&args[1..]);
            RespValue::Integer(count)
        }

        "EXISTS" => {
            if args.len() < 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let count = store.exists(&args[1..]);
            RespValue::Integer(count)
        }

        "KEYS" => {
            if args.len() > 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            if args.len() == 2 && args[1] != "*" {
                return RespValue::err("ERR only '*' pattern is supported for KEYS command");
            }

            let keys = store.keys();

            RespValue::Array(Some(
                keys.into_iter().map(|key| RespValue::bulk(key)).collect(),
            ))
        }

        "EXPIRE" => {
            if args.len() != 3 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let seconds = match args[2].parse::<u64>() {
                Ok(v) => v,
                Err(_) => {
                    return RespValue::err(format!(
                        "ERR value is not an integer or out of range for '{}' command",
                        cmd
                    ));
                }
            };

            let success = store.expire(&args[1], seconds);

            RespValue::Integer(if success { 1 } else { 0 })
        }

        "PERSIST" => {
            if args.len() != 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let success = store.persist(&args[1]);

            RespValue::Integer(if success { 1 } else { 0 })
        }

        "TTL" => {
            if args.len() != 2 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let ttl = store.ttl(&args[1]);

            RespValue::Integer(ttl)
        }

        cmd => RespValue::err(format!("ERR unknown command '{}'", cmd)),
    }
}
