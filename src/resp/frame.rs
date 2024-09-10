use enum_dispatch::enum_dispatch;
use crate::{BulkString, RespArray, RespMap, RespNullArray, RespNullBulkString, RespSet, SimpleError, SimpleString};

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;










impl RespNull { pub fn new() -> Self { RespNull } }


impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}


impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}


impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}



