
use crate::{RespArray, RespDecode, RespError, RespFrame, SimpleError, SimpleString};
use bytes::BytesMut;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
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
impl RespDecode for RespArray{
    fn decode(buf:&mut BytesMut)->Result<Self,RespError>{
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }
    }

}
/**
 * 从给定的字节缓冲区中提取简单帧数据。
 *
 * 该函数旨在处理Redis响应协议中的简单字符串类型帧。它首先检查缓冲区是否足够长，然后确认缓冲区的前缀是否符合预期，
 * 最后找到并返回简单字符串的结束位置。
 *
 * @param buf 待处理的字节缓冲区，应包含一个Redis简单字符串帧。
 * @param prefix 预期的帧前缀，用于识别帧类型。
 * @return 返回简单字符串帧的结束位置索引，如果帧不完整或类型不匹配，则返回错误。
 */
fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    // 检查缓冲区长度是否小于3，即至少应包含前缀和CRLF
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }

    // 检查缓冲区是否以预期的前缀开头
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: SimpleString({}), got: {:?}",
            prefix, buf
        )));
    }

    // 寻找CRLF的位置，即简单字符串的结束位置
    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;

    // 返回找到的结束位置
    Ok(end)
}

// find nth CRLF in the buffer 在缓冲区中找到第 n 个 CRLF
fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }

    None
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