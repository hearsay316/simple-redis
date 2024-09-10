use std::ops::Deref;
use bytes::{Buf, BytesMut};
use super::{RespDecode, RespEncode, RespError, RespFrame, extract_fixed_data, BUF_CAP, parse_length, calc_total_length, CRLF_LEN};
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;


// array: "*<number-of-elements>\r\n<element-1>...<element-n>"


impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_length = calc_total_length(buf,end, len, Self::PREFIX)?;
        if buf.len() < total_length {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        //with_capacity
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }
        Ok(RespArray::new(frames))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf,end,len,Self::PREFIX)
    }
}
impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "*";
        extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
        Ok(RespNullArray)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

#[cfg(test)]
mod test{
   use super::*;
    use anyhow::Result;
    use crate::BulkString;

    #[test]
    fn test_null_array_decode()->Result<()>{
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");
        let frame = RespNullArray::decode(&mut buf)?;
        assert_eq!(frame,RespNullArray);
        Ok(())
    }
    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }
    #[test]
    fn test_array_encode(){
        let frame:RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ]).into();
        // println!("{:?}",frame.encode().);
        // assert_eq!(frame.encode(),b"*3\r\n+set\r\n+hello\r\n+world\r\n");
        assert_eq!(frame.encode(),b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
    }
    #[test]
    fn test_calc_array_length()->Result<()>{
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end,len) = parse_length(buf,"*")?;
        let total_len = calc_total_length(buf,end,len,"*")?;
        assert_eq!(total_len,buf.len());

        let buf = b"*2\r\n$3\r\nset\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let ret = calc_total_length(buf, end, len, "*");
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);
        Ok(())
    }
}