use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use crate::RespFrame;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}
impl Default for RespMap {
    fn default() -> Self {
        RespMap::new()
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