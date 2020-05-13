//! Implement a redis module value

/// Represents the data which will be replied to client
#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Buffer(Vec<u8>),
    Integer(i64),
    Float(f64),
    Array(Vec<Value>),
    Null,
    NoReply,
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self {
        (i as i64).into()
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        s.to_owned().into()
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        s.to_owned().into()
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Buffer(b)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(s: Option<T>) -> Self {
        match s {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Value::Array(items.into_iter().map(|item| item.into()).collect())
    }
}
