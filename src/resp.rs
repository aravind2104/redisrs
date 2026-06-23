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
        b'+' => parse_simple_string(buf),
        b'-' => parse_error(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
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

fn parse_simple_string(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    let (line, consumed) = read_line(buf)?;
    Ok((RespValue::SimpleString(line.to_string()), consumed))
}

fn parse_error(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    let (line, consumed) = read_line(buf)?;
    Ok((RespValue::Error(line.to_string()), consumed))
}

fn parse_integer(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    let (line, consumed) = read_line(buf)?;
    let value = line
        .parse::<i64>()
        .map_err(|e| RespError::Protocol(format!("Invalid integer: {}", e)))?;
    Ok((RespValue::Integer(value), consumed))
}

fn parse_bulk_string(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    let (line, header_len) = read_line(buf)?;
    
    let data_len = line
        .parse::<i64>()
        .map_err(|e| RespError::Protocol(format!("Invalid bulk string length: {}", e)))?;
    
    if data_len == -1 {
        return Ok((RespValue::BulkString(None), header_len));
    }

    if data_len < 0 {
        return Err(RespError::Protocol(
            "Negative bulk string length".to_string(),
        ));
    }

    let data_len = data_len as usize;
    let total_len = header_len + data_len + 2;

    if buf.len() < total_len {
        return Err(RespError::Incomplete);
    }

    let data = std::str::from_utf8(&buf[header_len..header_len+data_len])
        .map_err(|e| RespError::Protocol(format!("Invalid UTF-8 in bulk string: {}", e)))?;

    Ok((RespValue::BulkString(Some(data.to_string())), total_len))
}


fn parse_array(buf: &[u8]) -> Result<(RespValue, usize), RespError> {
    let (line, header_len) = read_line(buf)?;

    let array_len = line
        .parse::<i64>()
        .map_err(|e| RespError::Protocol(format!("Invalid array length: {}", e)))?;

    if array_len == -1 {
        return Ok((RespValue::Array(None), header_len));
    }

    if array_len < 0 {
        return Err(RespError::Protocol(
            "Negative array length".to_string(),
        ));
    }

    let mut offset = header_len;
    let mut values = Vec::with_capacity(array_len as usize);

    for _ in 0..array_len {
        let (value, consumed) = parse(&buf[offset..])?;
        values.push(value);
        offset += consumed;
    }

    Ok((RespValue::Array(Some(values)), offset))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let buf = b"+OK\r\n";
        let (value, consumed) = parse(buf).unwrap();
        assert_eq!(value, RespValue::SimpleString("OK".into()));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_parse_error() {
        let buf = b"-ERR unknown command 'foo'\r\n";
        let (value, consumed) = parse(buf).unwrap();
        assert_eq!(value, RespValue::Error("ERR unknown command 'foo'".into()));
        assert_eq!(consumed, 28);
    }

    #[test]
    fn test_parse_integer() {
        let buf = b":42\r\n";
        let (value, consumed) = parse(buf).unwrap();
        assert_eq!(value, RespValue::Integer(42));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_parse_bulk_string() {
        let buf = b"$5\r\nhello\r\n";
        let (value, consumed) = parse(buf).unwrap();
        assert_eq!(value, RespValue::BulkString(Some("hello".into())));
        assert_eq!(consumed, 11);
    }

    #[test]
    fn test_parse_array() {
        let buf = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let (value, consumed) = parse(buf).unwrap();
        assert_eq!(value, RespValue::Array(Some(vec![
            RespValue::BulkString(Some("hello".into())),
            RespValue::BulkString(Some("world".into())),
        ])));
        assert_eq!(consumed, 26);
    }

    #[test]
    fn test_parse_incomplete() {
        let buf = b"$5\r\nhel";
        let err = parse(buf).unwrap_err();
        assert!(matches!(err, RespError::Incomplete));
    }
}