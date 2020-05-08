use std;
use std::error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    WrongArity,
    Generic(GenericError),
}

impl Error {
    pub fn generic(message: &str) -> Error {
        Error::Generic(GenericError::new(message))
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::generic(&err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Error {
        Error::generic("Expect int value")
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Error {
        Error::generic("Expect float value")
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Error::generic("Expect utf8 string")
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Error {
        Error::generic("Expect utf8 string")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::WrongArity => write!(f, "Wrong Arity"),
            Error::Generic(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::WrongArity => None,
            Error::Generic(ref err) => Some(err),
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
