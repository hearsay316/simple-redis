

mod array;
mod frame;
mod bool;
mod bulk_string;
mod double;

mod integer;
mod map;
mod set;
mod simple_error;
mod simple_string;
mod null;

use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

pub use self::{array::{RespArray, RespNullArray}, bulk_string::{BulkString, RespNullBulkString},frame::{RespFrame}, map::RespMap, simple_error::SimpleError, simple_string::SimpleString,set::RespSet,null::RespNull};

pub const BUF_CAP: usize = 4096;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
// use encode::*;
// use decode::*;
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
#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Error, Debug, PartialEq, Eq, )]
pub enum RespError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length： {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,

    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}
// 公共函数
fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end, s.parse()?))
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
fn calc_total_length(buf: &[u8],end:usize, len: usize, prefix: &str) -> Result<usize, RespError> {
    // 从给定的起始位置开始切片，获取当前处理的消息数据。
    let mut total = end+CRLF_LEN;
    let mut data = &buf[total..];

    // 根据前缀的不同，采取不同的处理逻辑。
    match prefix {
        "*" | "~" => {
            // 对于多值回复或模糊匹配回复，查找下一个CRLF的位置，以确定当前消息的结束位置。
            //  find nth CRLF IN  THE buffer 在缓冲区中找到第n个CRLF
            for _ in 0..len {
                let  len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total+=len;
            }
            Ok(total)
            // find_crlf(data, len).map(|end| len + CRLF_LEN + end).ok_or(RespError::NotComplete)
        }
        "%" => {
            // 对于键值对回复，需要找到两个CRLF来确定一个键值对的结束，因此长度计算要考虑到这一点。
            //  find nth CRLF IN  THE buffer 在缓冲区中找到第n个CRLF
            // we need to find 2 CRLF for each key-value pair 我们需要为每个键值对找到2个CRLF
            // find_crlf(data, len * 2).map(|end| len + CRLF_LEN + end).ok_or(RespError::NotComplete)
            for _ in 0..len {
                let  len = SimpleString::expect_length(data)?;
                data = &data[len..];
                total+=len;

                let  len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total+=len;

            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
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