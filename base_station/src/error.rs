use std::{io, fmt};

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    Other(String)
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Other(error)
    }
}
