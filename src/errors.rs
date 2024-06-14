use std::fmt::{self, Formatter};

pub type Result<T> = std::result::Result<T, self::Error>;

pub(crate) fn runtime_err(message: impl ToString) -> self::Error {
    Error {
        message: message.to_string(),
        kind: ErrorKind::Runtime,
    }
}

pub(crate) fn typecheck_err(message: impl ToString) -> self::Error {
    Error {
        message: message.to_string(),
        kind: ErrorKind::Type,
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Runtime,
    Type,
}

impl Error {
    pub fn is_typecheck_err(&self) -> bool {
        matches!(self.kind, ErrorKind::Type)
    }
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
