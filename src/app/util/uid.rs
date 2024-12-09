use nanoid::nanoid;
use std::fmt::Display;

/// Unique identifier for statements or branches
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(String);

impl Uid {
    pub fn new() -> Self {
        Self(nanoid!(10))
    }
}

impl From<&str> for Uid {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
