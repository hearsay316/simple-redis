mod decode;
mod encode;

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


use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

pub use self::{array::{RespArray, RespNullArray}, bulk_string::{BulkString, RespNullBulkString},frame::{RespFrame,RespNull}, map::RespMap, simple_error::SimpleError, simple_string::SimpleString,set::RespSet};


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
