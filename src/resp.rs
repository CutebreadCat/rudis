use crate::resp_result::RESPError;
use crate::resp_result::RESPLenth;
use crate::resp_result::RESPResult;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RESP {
    SimpleString(String),
    Null,
    BulkString(String),
    RDBPrefix(usize),
    Array(Vec<RESP>),
    Integer(i64),
    Error(String),
}

impl fmt::Display for RESP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            RESP::SimpleString(value) => format!("+{}\r\n", value),
            RESP::Null => "$-1\r\n".to_string(),
            RESP::BulkString(value) => format!("${}\r\n{}\r\n", value.len(), value),
            RESP::RDBPrefix(value) => format!("*RDBPrefix {}\r\n", value),
            RESP::Array(values) => {
                let mut output = format!("*{}\r\n", values.len());
                for value in values {
                    output.push_str(&value.to_string());
                }
                output
            }
            RESP::Integer(value) => format!(":{}\r\n", value),
            RESP::Error(value) => format!("-{}\r\n", value),
        };
        write!(f, "{}", data)
    }
}
#[cfg(test)]
mod tests {
    use crate::resp_result::{RESPError, RESPResult};

    use super::*;

    #[test]
    fn test_binary_extract_line() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_line(buffer, &mut index).unwrap();
        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index, 4);
    }
    #[test]
    fn test_binary_extract_line_loger_string() {
        let buffer = "ECHO\r\n".as_bytes();
        let mut index: usize = 0;
        let Output = binary_extract_line(buffer, &mut index).unwrap();
        assert_eq!(Output, "ECHO".as_bytes());
        assert!(index == 6);
    }
    #[test]
    fn test_binary_extract_line_empty_buffer() {
        let buffer = "".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 0);
            }
            _ => panic!("unexpected error"),
        }
    }
    #[test]
    fn test_binary_extract_line_no_separator() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 0;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2)
            }
            _ => panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line_index_too_advanced() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 1;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line_as_string() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_line_as_string(buffer, &mut index).unwrap();
        assert_eq!(output, String::from("OK"));
        assert_eq!(index, 4);
    }
    #[test]
    fn test_resp_remove_type() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        resp_remove_type('+', buffer, &mut index).unwrap();
        assert_eq!(index, 1);
    }
    #[test]
    fn test_resp_remove_type_error() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = resp_remove_type('-', buffer, &mut index).unwrap_err();
        assert_eq!(index, 0);
        assert_eq!(error, RESPError::WrongType);
    }
    #[test]
    fn test_parse_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_simple_string(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }
    #[test]
    fn test_to_resp_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }
    #[test]
    fn test_to_resp_error() {
        let buffer = "?OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap_err();
        assert_eq!(output, RESPError::Unkonwn);
        assert_eq!(index, 0);
    }
    #[test]
    fn test_binary_extract_bytes() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_bytes(buffer, &mut index, 2).unwrap();
        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index, 2);
    }
    #[test]
    fn test_binary_extract_bytes_error() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = binary_extract_bytes(buffer, &mut index, 10).unwrap_err();
        assert_eq!(output, RESPError::OutOfBounds(0));
        assert_eq!(index, 0);
    }
    #[test]
    fn test_parse_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_bulk_string(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("OK")));
        assert_eq!(index, 8);
    }
    #[test]
    fn test_parse_bulk_string_empty() {
        let buffer = "$-1\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_bulk_string(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::Null);
        assert_eq!(index, 5);
    }
    #[test]
    fn test_parse_bulk_string_unparsable_length() {
        let buffer = "$wrong\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::ParseInt);
        assert_eq!(index, 8);
    }
    #[test]
    fn test_parse_bulk_string_negative_length() {
        let buffer = "$-7\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::IncorrectLength(-7));
        assert_eq!(index, 5);
    }
    #[test]
    fn test_parse_bulk_string_missing_crlf() {
        let buffer = "$2\r\nOK".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::OutOfBounds(6));
    }
    #[test]
    fn test_to_resp_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("OK")));
        assert_eq!(index, 8);
    }
}

fn binary_extract_line(buffer: &[u8], index: &mut usize) -> RESPResult<Vec<u8>> {
    let mut output = Vec::new();
    if *index >= buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }
    let mut previous_elem: u8 = buffer[*index].clone();
    let mut final_index: usize = *index;
    let mut separator_found: bool = false;
    for &elem in buffer[*index..].iter() {
        final_index += 1;
        if elem == b'\n' && previous_elem == b'\r' {
            separator_found = true;
            break;
        }
        previous_elem = elem;
    }
    if !separator_found {
        *index = final_index;
        return Err(RESPError::OutOfBounds(*index));
    }
    output.extend_from_slice(&buffer[*index..final_index - 2]);
    *index = final_index;
    Ok(output)
}
fn binary_extract_line_as_string(buffer: &[u8], index: &mut usize) -> RESPResult<String> {
    let line = binary_extract_line(buffer, index)?;
    Ok(String::from_utf8(line)?)
}
pub fn resp_remove_type(value: char, buffer: &[u8], index: &mut usize) -> RESPResult<()> {
    if *index >= buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }
    if buffer[*index] != value as u8 {
        return Err(RESPError::WrongType);
    }
    *index += 1;
    Ok(())
}

pub fn parse_simple_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('+', buffer, index)?;
    let value = binary_extract_line_as_string(buffer, index)?;
    Ok(RESP::SimpleString(value))
}

fn parser_router(
    buffer: &[u8],
    index: &mut usize,
) -> Option<fn(&[u8], &mut usize) -> RESPResult<RESP>> {
    match buffer[*index] {
        b'+' => Some(parse_simple_string),
        b'$' => Some(parse_bulk_string),
        b'*' => Some(parse_array),
        _ => None,
    }
}

pub fn bytes_to_resp(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    match parser_router(buffer, index) {
        Some(parse_func) => {
            let result: RESP = parse_func(buffer, index)?;
            return Ok(result);
        }
        None => return Err(RESPError::Unkonwn),
    }
}

fn binary_extract_bytes(buffer: &[u8], index: &mut usize, length: usize) -> RESPResult<Vec<u8>> {
    let mut output = Vec::new();
    if *index + length > buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }
    output.extend_from_slice(&buffer[*index..*index + length]);
    *index += length;
    Ok(output)
}

pub fn resp_extract_length(buffer: &[u8], index: &mut usize) -> RESPResult<RESPLenth> {
    let length_str = binary_extract_line_as_string(buffer, index)?;
    let length: RESPLenth = length_str.parse()?;
    Ok(length)
}

fn parse_bulk_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('$', buffer, index)?;
    let length = resp_extract_length(buffer, index)?;
    if length == -1 {
        return Ok(RESP::Null);
    }
    if length < -1 {
        return Err(RESPError::IncorrectLength(length));
    }
    let bytes = binary_extract_bytes(buffer, index, length as usize)?;
    if *index + 2 > buffer.len() || buffer[*index] != b'\r' || buffer[*index + 1] != b'\n' {
        return Err(RESPError::OutOfBounds(*index));
    }
    *index += 2;
    let value = String::from_utf8(bytes)?;
    Ok(RESP::BulkString(value))
}
fn parse_array(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    resp_remove_type('*', buffer, index)?;
    let length = resp_extract_length(buffer, index)?;
    if length < 0 {
        return Err(RESPError::IncorrectLength(length));
    }
    let mut output = Vec::new();
    for _ in 0..length {
        match parser_router(buffer, index) {
            Some(parse_func) => {
                let result: RESP = parse_func(buffer, index)?;
                output.push(result);
            }
            None => return Err(RESPError::Unkonwn),
        }
    }
    Ok(RESP::Array(output))
}

pub fn bulk_string_from_vec(string: Vec<String>) -> RESP {
    RESP::BulkString(string.join("\r\n"))
}
