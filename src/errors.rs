use std::fmt::{self, Formatter};

pub type Result<T> = std::result::Result<T, self::Error>;

pub(crate) fn runtime_err(message: impl ToString) -> self::Error {
    Error {
        message: message.to_string(),
        kind: ErrorKind::Runtime,
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Runtime,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self { message, .. } = self;
        write!(f, "{message}")
    }
}

impl std::error::Error for self::Error {}

impl<T> From<self::Error> for self::Result<T> {
    fn from(err: self::Error) -> Self {
        Err(err)
    }
}
