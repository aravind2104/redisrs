use crate::resp::RespValue;
use crate::store::Store;
use std::sync::Arc;

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
                Some(value) => RespValue::bulk(value),
                None => RespValue::null(),
            }
        }

        "SET" => {
            if args.len() != 3 {
                return RespValue::err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            store.set(args[1].clone(), args[2].clone());
            RespValue::ok()
        }

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

        cmd => RespValue::err(format!("ERR unknown command '{}'", cmd)),
    }
}
