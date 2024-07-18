
use crate::{BulkString, RespArray, RespDecode, RespError, RespFrame, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet, SimpleError, SimpleString};
use bytes::{Buf, BytesMut};

/*
- 如何解析 Frame
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
 */
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
impl RespDecode for RespFrame {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                match RespNullBulkString::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = BulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'*') => {
                match RespNullArray::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = RespArray::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            _ => Err(RespError::InvalidFrameType(format!("expect_length: unknown frame type: {:?}", buf)))
        }
    }
}
impl RespDecode for SimpleString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }
        if !buf.starts_with(b"+") {
            return Err(RespError::InvalidFrameType(format!("expect: SimpleString(),got {:?}", buf)));
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
}
impl RespDecode for RespNull {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "_";
        extract_fixed_data(buf, "_\r\n", "Null")?;
        Ok(RespNull)
    }
}
impl RespDecode for RespNullArray {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "*";
        extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
        Ok(RespNullArray)
    }
}
impl RespDecode for RespNullBulkString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "$-1\r\n";
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(RespNullBulkString)
    }
}
impl RespDecode for bool {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(true),
                Err(e) => Err(e)
            }
        }
    }
}
impl RespDecode for BulkString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "$";
        let (end,len) = parse_length(buf,prefix)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }
}

impl RespDecode for RespArray {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "*";
        let (end,len) = parse_length(buf,prefix)?;
        let total_length = calc_total_length(buf,len,prefix)?;
        if buf.len() < total_length {
            return Err(RespError::NotComplete);
        }
        buf.advance(end+CRLF_LEN);
        //with_capacity
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }
        Ok(RespArray::new(frames))
    }
}
//  - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespDecode for i64 {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = ":";
        let end = extract_simple_frame_data(buf, prefix, 1)?;
        // split the buffer
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[prefix.len()..end]);
        Ok(s.parse()?)
    }
}
impl RespDecode for f64 {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = ",";
        let end = extract_simple_frame_data(buf, prefix, 1)?;
        println!("{:?}", buf);
        let data = buf.split_to(end + CRLF_LEN);
        // println!("{:?}",data);
        let s = String::from_utf8_lossy(&data[prefix.len()..end]);
        Ok(s.parse()?)
    }
}
//"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "%";
        let (end,len) = parse_length(buf,prefix)?;
        let total_length = calc_total_length(buf,len,prefix)?;
        if buf.len() < total_length {
            return Err(RespError::NotComplete);
        }
        buf.advance(end+CRLF_LEN);
        let mut frames = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }
        Ok(frames)
    }
}
// set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespDecode for RespSet {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "~";
        let (end,len) = parse_length(buf,prefix)?;
        let total_length = calc_total_length(buf,len,prefix)?;
        if buf.len() < total_length {
            return Err(RespError::NotComplete);
        }
        buf.advance(end+CRLF_LEN);
        let mut frames: Vec<RespFrame> = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }
        Ok(RespSet::new(frames))
    }
}
fn parse_length(buf: &mut BytesMut, prefix: &str) -> Result<(usize,usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix, 1)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end,s.parse()?))
}
/**
 * 计算缓冲区中特定前缀消息的总长度。
 *
 * 此函数用于处理Redis响应缓冲区，根据不同的前缀指示，计算出一个完整的消息应该到哪里结束。
 * 主要用于解析Redis的多值回复（*）和模糊匹配回复（~），以及处理特定的键值对回复（%）。
 *
 * @param buf 一个可变引用到BytesMut对象，代表当前的缓冲区，用于存放Redis的响应数据。
 * @param len 当前处理的消息在缓冲区中的起始位置。
 * @param prefix 当前处理的消息的前缀，用于指示不同类型的回复。
 * @return 返回一个Result，包含计算出的总长度或者一个错误，如果消息不完整则返回错误。
 */
fn calc_total_length(buf: &[u8], len: usize, prefix: &str) -> Result<usize, RespError> {
    // 从给定的起始位置开始切片，获取当前处理的消息数据。
    let data = &buf[len+CRLF_LEN..];

    // 根据前缀的不同，采取不同的处理逻辑。
    match prefix {
        "*" | "~" => {
            // 对于多值回复或模糊匹配回复，查找下一个CRLF的位置，以确定当前消息的结束位置。
            //  find nth CRLF IN  THE buffer 在缓冲区中找到第n个CRLF
            find_crlf(data, len).map(|end| len+CRLF_LEN+end).ok_or(RespError::NotComplete)
        }
        "%" => {
            // 对于键值对回复，需要找到两个CRLF来确定一个键值对的结束，因此长度计算要考虑到这一点。
            //  find nth CRLF IN  THE buffer 在缓冲区中找到第n个CRLF
            // we need to find 2 CRLF for each key-value pair 我们需要为每个键值对找到2个CRLF
            find_crlf(data, len*2).map(|end| len+CRLF_LEN+end).ok_or(RespError::NotComplete)
        }
        _ => Ok(len+CRLF_LEN),
    }
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got: {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());
    Ok(())
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
fn extract_simple_frame_data(buf: &[u8], prefix: &str, nth_crlf: usize) -> Result<usize, RespError> {
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
    let end = find_crlf(buf, nth_crlf).ok_or(RespError::NotComplete)?;

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
mod tests {
    use super::*;
    use anyhow::Result;
    use bytes::BufMut;

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
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",123.456\r\n");
        let frame: f64 = f64::decode(&mut buf)?;
        assert_eq!(frame, 123.456);
        buf.extend_from_slice(b",+1.23456e-9\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-9);
        Ok(())
    }
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
    fn test_set_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n");
        let frame: RespSet = RespSet::decode(&mut buf)?;
        let  set = RespSet::new(vec![
            BulkString::new(b"set".to_vec()).into(),
            BulkString::new(b"hello".to_vec()).into(),
        ]);
        assert_eq!(frame, set);
        Ok(())
    }
}