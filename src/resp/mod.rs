use std::collections::{BTreeMap};
use thiserror::Error;
use bytes::BytesMut;
use std::ops::{Deref, DerefMut};

use enum_dispatch::enum_dispatch;
mod encode;
mod decode;
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

pub trait RespDecode : Sized {
    const PREFIX:&'static str;
    fn decode(buf:&mut BytesMut) -> Result<Self,RespError>;
    fn expect_length(buf:&[u8])->Result<usize,RespError>;
}

#[derive(Debug,Error,PartialEq, Eq,)]
pub enum RespError{
    #[error("Invalid frame : {0}")]
    InvalidFrame(String),
    #[error("invalid Frame Type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length : {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,
    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError)
}


// impl RespDecode for BytesMut {
//     fn decode(buf:&mut  BytesMut) -> Result<Self, RespError> {
//         todo!()
//     }
// }
#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString( SimpleString),
    Error(SimpleError),

    Integer(i64),
    BulkString(BulkString),

    NullBulkString(RespNullBulkString),
    Array(RespArray),

    Null(RespNull),
    NullArray(RespNullArray),

    Boolean(bool),
    Double(f64),

    Map(RespMap),
    Set(RespSet),
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String);
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNull;
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
#[derive(Debug, PartialEq,  PartialOrd)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
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

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}
impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}
impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}
impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
impl RespNull { pub fn new() -> Self { RespNull } }

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

// impl From<&str> for RespFrame {
//     fn from(s: &str) -> Self {
//         SimpleString(s.to_string()).into()
//     }
// }

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl From<Vec<u8>> for BulkString{
    fn from(value: Vec<u8>) -> Self {
        BulkString(value)
    }
}
impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}
impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
