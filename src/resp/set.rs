use std::ops::Deref;
use crate::RespFrame;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}