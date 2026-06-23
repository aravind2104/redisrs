use crate::resp::RespValue;
use crate::store::Store;
use std::sync::Arc;

pub fn execute(args: Vec<String>, store: Arc<Store>) -> RespValue {
    let _ = store; //Unused for now

    if args.is_empty() {
        return RespValue::err("ERR empty command");
    }

    match args[0].to_uppercase().as_str() {
        "PING" => {
            if args.len() == 1 {
                RespValue::SimpleString("PONG".into())
            } else if args.len() == 2 {
                RespValue::bulk(args[1].clone())
            } else {
                RespValue::err("ERR wrong number of arguments for 'ping' command")
            }
        }

        cmd => RespValue::err(format!("ERR unknown command '{}'", cmd)),
    }
}