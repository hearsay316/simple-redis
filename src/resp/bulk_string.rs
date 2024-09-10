use std::ops::Deref;

#[derive(Debug, Clone, PartialEq,Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}
impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
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
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}