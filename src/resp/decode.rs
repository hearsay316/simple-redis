
use crate::{RespDecode, RespError, RespFrame, SimpleError, SimpleString};
use bytes::BytesMut;
// impl RespDecode for RespFrame {
//     fn decode(buf: &mut BytesMut) -> Result<RespFrame, RespError> {
//         let mut item = buf.iter().peekable();
//         match item.peek() {
//             Some(b'+') => {
//                 todo!()
//             }
//             _ => todo!()
//         }
//         todo!()
//     }
// }
impl RespDecode for SimpleString {
    fn decode( buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }
        if !buf.starts_with(b"+") {
            return Err(RespError::InvalidFrameType(format!("expect: SimpleString(),got {:?}", buf)));
        }
        // 搜索 "\r\n"
        let mut end = 0;
         for i in 1..buf.len() - 1 {
             if buf[i] ==b'\r'&&buf[i+1] == b'\n' {
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
        Ok(SimpleString::new(s.to_string()))
    }
}
impl RespDecode for SimpleError {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }
        if !buf.starts_with(b"-") {
            return Err(RespError::InvalidFrameType(format!("expect: SimpleError(),got {:?}", buf)));
        }
        // 搜索 "\r\n"
        let mut end = 0;
         for i in 1..buf.len() - 1 {
             if buf[i] ==b'\r'&&buf[i+1] == b'\n' {
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
}
#[cfg(test)]
mod tests{
    use super::*;
    use anyhow::Result;
    use bytes::BufMut;

    #[test]
    fn test_decode_simple_string()->Result<()>{
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");
        let frame : SimpleString= SimpleString::decode(&mut buf)?.into();
        assert_eq!(frame, SimpleString::new("OK".to_string()));
        buf.extend_from_slice(b"+hello\r");
        //
        let ret = SimpleString::decode(&mut buf);
            // println!("{:?}", ret.unwrap_err());
        assert_eq!(ret.unwrap_err(),RespError::NotComplete);
        buf.put_u8(b'\n');
        let frame : SimpleString= SimpleString::decode(&mut buf)?.into();
        assert_eq!(frame, SimpleString::new("hello".to_string()));
        Ok(())
    }
}