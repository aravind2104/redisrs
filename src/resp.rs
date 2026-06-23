use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespValue>>),
}

#[derive(Error, Debug)]
pub enum RespError {
    #[error("Incomplete data - need more bytes")]
    Incomplete,
    #[error("Protocol error: {0}")]
    Protocol(String),
}


pub fn parse(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    if buf.is_empty() {
        return Err(RespError::Incomplete);
    }

    match buf[0] {
        b'+' => todo!(), // Simple String
        b'-' => todo!(), // Error
        b':' => todo!(), // Integer
        b'$' => todo!(), // Bulk String
        b'*' => todo!(), // Array
        _ => Err(RespError::Protocol(format!("Unknown type byte: {}", buf[0] as char))),
    }
}

fn find_crlf(buf: &[u8], start: usize) -> Option<usize> {
    let mut i = start;

    while i + 1 < buf.len() {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            return Some(i);
        }
        i += 1;
    }

    None
}

fn read_line(buf: &[u8]) -> Result<(&str, usize) , RespError> {
    let Some(pos) =find_crlf(buf, 1) else {
        return Err(RespError::Incomplete);
    };
    let line = std::str::from_utf8(&buf[1..pos])
        .map_err(|e| RespError::Protocol(format!("Invalid UTF-8 in line: {}", e)))?;
    Ok((line, pos + 2))
}