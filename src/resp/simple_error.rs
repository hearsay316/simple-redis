use std::ops::Deref;
use bytes::BytesMut;
use super::{RespDecode, RespEncode, RespError, extract_simple_frame_data, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
//error: "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }
        if !buf.starts_with(Self::PREFIX.as_ref()) {
            return Err(RespError::InvalidFrameType(format!("expect: SimpleError(),got {:?}", buf)));
        }
        // 搜索 "\r\n"
        let mut end = 0;
        for i in 1..buf.len() - 1 {
            if buf[i] == b'\r' && buf[i + 1] == b'\n' {
                end = i;
                break;
            }
        }
        if end == 0 {
            return Err(RespError::NotComplete);
        }
        //split the buffer
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleError::new(s.to_string()))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use crate::RespFrame;

    #[test]
    fn test_simple_error_decode()->Result<()>{
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error message\r\n");
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame,SimpleError::new("Error message".to_string()));
        Ok(())
    }
    #[test]
    fn test_simple_error_encode(){
        let frame:RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(),b"-Error message\r\n");
    }
    #[test]
    fn test_error_encode(){
        let frame :RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(),b"-Error message\r\n");

    }

}