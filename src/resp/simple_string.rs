use std::ops::Deref;
use bytes::BytesMut;
use crate::{RespDecode, RespEncode, RespError};
use crate::resp::{extract_simple_frame_data, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
//- simple string:"+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // if buf.len() < 3 {
        //     return Err(RespError::NotComplete);
        // }
        // if !buf.starts_with(Self::PREFIX) {
        //     return Err(RespError::InvalidFrameType(format!("expect: SimpleString(),got {:?}", buf)));
        // }
        // // 搜索 "\r\n"
        // let mut end = 0;
        // for i in 1..buf.len() - 1 {
        //     if buf[i] == b'\r' && buf[i + 1] == b'\n' {
        //         end = i;
        //         break;
        //     }
        // }
        // if end == 0 {
        //     return Err(RespError::NotComplete);
        // }
        // //split the buffer
        // let data = buf.split_to(end + 2);
        // let s = String::from_utf8_lossy(&data[1..end]);
        // Ok(SimpleString::new(s.to_string()))
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(SimpleString::new(s.to_string()))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}


impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use bytes::BufMut;
    use crate::RespFrame;

    #[test]
    fn test_decode_simple_string() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");
        let frame: SimpleString = SimpleString::decode(&mut buf)?.into();
        assert_eq!(frame, SimpleString::new("OK".to_string()));
        buf.extend_from_slice(b"+hello\r");
        //
        let ret = SimpleString::decode(&mut buf);
        // println!("{:?}", ret.unwrap_err());
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);
        buf.put_u8(b'\n');
        let frame: SimpleString = SimpleString::decode(&mut buf)?.into();
        assert_eq!(frame, SimpleString::new("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_simple_string_encode() {
        let frame:RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

}