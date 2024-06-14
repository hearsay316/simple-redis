use std::collections::{HashMap, HashSet};
use anyhow::Result;

pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespFrame>),
    Null,
    NullArray,
    Boolean(bool),
    Double(f64),
    Map(HashMap<String,RespFrame>),
    Set(HashSet<RespFrame>),
}
pub struct SimpleString(String);
pub struct SimpleError(String);