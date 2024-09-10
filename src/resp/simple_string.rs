use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}