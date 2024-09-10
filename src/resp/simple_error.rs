use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}