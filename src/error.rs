use std;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::str;
use crate::raw;

#[derive(Debug)]
pub enum RedisError {
    WrongArity,
    WrongType,
    Generic(GenericError),
}

impl RedisError {
    pub fn generic(message: &str) -> RedisError {
        RedisError::Generic(GenericError::new(message))
    }
}

impl From<String> for RedisError {
    fn from(err: String) -> RedisError {
        RedisError::generic(&err)
    }
}

impl From<std::num::ParseIntError> for RedisError {
    fn from(_: std::num::ParseIntError) -> RedisError {
        RedisError::generic("value is not int")
    }
}

impl From<std::num::ParseFloatError> for RedisError {
    fn from(_: std::num::ParseFloatError) -> RedisError {
        RedisError::generic("value is not float")
    }
}

impl From<std::str::Utf8Error> for RedisError {
    fn from(_: std::str::Utf8Error) -> Self {
        RedisError::generic("value is not utf8")
    }
}

impl From<std::string::FromUtf8Error> for RedisError {
    fn from(_: std::string::FromUtf8Error) -> RedisError {
        RedisError::generic("value is not utf8")
    }
}

impl Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RedisError::WrongType => write!(f, "{}", str::from_utf8(raw::REDISMODULE_ERRORMSG_WRONGTYPE).unwrap()),
            RedisError::WrongArity => write!(f, "ERR wrong number of arguments"),
            RedisError::Generic(ref err) => write!(f, "ERR {}", err),
        }
    }
}

impl error::Error for RedisError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            RedisError::WrongType => None,
            RedisError::WrongArity => None,
            RedisError::Generic(ref err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub struct GenericError {
    message: String,
}

impl GenericError {
    pub fn new(message: &str) -> GenericError {
        GenericError {
            message: String::from(message),
        }
    }
}

impl<'a> Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<'a> error::Error for GenericError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
