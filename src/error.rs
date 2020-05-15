//! Error management

use crate::raw;
use std;
use std::error;
use std::fmt;
use std::fmt::Display;

/// The core error component
#[derive(Debug)]
pub enum Error {
    WrongArity,
    WrongType,
    Custom(CustomError),
}

impl Error {
    pub fn new<T: AsRef<str>>(message: T) -> Error {
        Error::Custom(CustomError::new(message.as_ref()))
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::new(&err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Error {
        Error::new("value is not int")
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Error {
        Error::new("value is not float")
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Error::new("value is not utf8")
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Error {
        Error::new("value is not utf8")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::WrongType => write!(
                f,
                "{}",
                std::str::from_utf8(raw::REDISMODULE_ERRORMSG_WRONGTYPE).unwrap()
            ),
            Error::WrongArity => write!(f, "ERR wrong number of arguments"),
            Error::Custom(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::WrongType => None,
            Error::WrongArity => None,
            Error::Custom(ref err) => Some(err),
        }
    }
}
/// A custom eror
#[derive(Debug)]
pub struct CustomError {
    message: String,
}

impl CustomError {
    pub fn new(message: &str) -> CustomError {
        CustomError {
            message: String::from(message),
        }
    }
}

impl<'a> Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<'a> error::Error for CustomError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
