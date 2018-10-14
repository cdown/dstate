use std::{str, string, io};

#[derive(Debug)]
pub enum DStateError {
    StringUtf8(string::FromUtf8Error),
    StrUtf8(str::Utf8Error),
    Io(io::Error),
    InvalidStatFile,
}

impl From<io::Error> for DStateError {
    fn from(err: io::Error) -> DStateError {
        DStateError::Io(err)
    }
}

impl From<string::FromUtf8Error> for DStateError {
    fn from(err: string::FromUtf8Error) -> DStateError {
        DStateError::StringUtf8(err)
    }
}

impl From<str::Utf8Error> for DStateError {
    fn from(err: str::Utf8Error) -> DStateError {
        DStateError::StrUtf8(err)
    }
}
