use std::ops::Deref;
use bytes::{Buf, BytesMut};
use super::{RespDecode, RespEncode, RespError, extract_fixed_data, parse_length, CRLF_LEN};
#[derive(Debug, Clone, PartialEq,Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        // Vec::with_capacity()方法创建一个具有指定容量的Vec<u8>，这样可以避免在push()时重新分配内存
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
        // format!("${}\r\n{}\r\n",self.len(),String::from_utf8(self).unwrap()).into_bytes()
        // format!("${}\r\n{}\r\n",self.len(),String::from_utf8_lossy(&self)).into_bytes()
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}
impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "$-1\r\n";
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(RespNullBulkString)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}
impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}
impl From<String> for BulkString {
    fn from(value: String) -> Self {
        BulkString(value.into_bytes())
    }
}
impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    use crate::{RespFrame};


    #[test]
    fn test_bulk_string_decode()->Result<()>{
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame= BulkString::decode(&mut buf)?;
        assert_eq!(frame,BulkString::new(b"hello"));
        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(),RespError::NotComplete);
        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame,BulkString::new(b"hello"));
        Ok(())
    }
    #[test]
    fn test_null_bulk_string_decode()->Result<()>{
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");
        let frame =RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame,RespNullBulkString);
        Ok(())
    }
    #[test]
    fn test_bulk_string_encode(){
        let frame:RespFrame = BulkString::new("OK".to_string()).into();
        assert_eq!(frame.encode(),b"$2\r\nOK\r\n");
    }
    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

}