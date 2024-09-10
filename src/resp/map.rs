use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use bytes::{Buf, BytesMut};
use super::{RespDecode, RespEncode, RespError, RespFrame, SimpleString, calc_total_length, parse_length, CRLF_LEN, BUF_CAP};


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

//map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

//"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_length = calc_total_length(buf,end, len, Self::PREFIX)?;
        if buf.len() < total_length {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }
        Ok(frames)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf,end,len,Self::PREFIX)
    }
}
impl Default for RespMap {
    fn default() -> Self {
        RespMap::new()
    }
}
impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use crate::BulkString;

    #[test]
    fn test_map_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+set\r\n$5\r\nhello\r\n+set\r\n$5\r\nhello\r\n");
        // buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        //
        let frame: RespMap = RespMap::decode(&mut buf)?;
        let mut map = RespMap::new();
        map.insert("set".to_string(), BulkString::new(b"hello".to_vec()).into());
        map.insert("set".to_string(), BulkString::new(b"hello".to_vec()).into());
        assert_eq!(frame, map);
        Ok(())

        // let mut buf = BytesMut::new();
        //
        // let frame = RespMap::decode(&mut buf)?;
        // let mut map = RespMap::new();
        // map.insert(
        //     "hello".to_string(),
        //     BulkString::new(b"world".to_vec()).into(),
        // );
        // map.insert("foo".to_string(), BulkString::new(b"bar".to_vec()).into());
        // assert_eq!(frame, map);
        //
        // Ok(())
    }
    #[test]
    fn test_map_encode(){
        let mut map: RespMap = RespMap::new();
        map.insert("key".to_string(),SimpleString::new("value".to_string()).into());
        let frame:RespFrame = map.into();
        assert_eq!(frame.encode(),b"%1\r\n+key\r\n+value\r\n");
    }
}