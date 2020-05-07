use num_traits::FromPrimitive;
use std::ops::Add;
use std::fmt;
use std::time::Duration;

use std::os::raw::{c_char, c_double, c_int, c_long, c_longlong};
use crate::bindings::*;
use crate::error::Error;

pub struct RetCode(i32);

impl RetCode {
    pub fn check(v: c_int) -> Result<(), Error> {
        if v == REDISMODULE_OK as i32 {
            return Ok(());
        }
        Err(Error::code(v))
    }
}


#[derive(Debug, PartialEq)]
pub enum CommandStrFlags {
    Write,
    Readonly,
    Admin,
    DenyOOM,
    DenyScript,
    AllowLoading,
    Pubsub,
    Random,
    AllowStale,
    NoMonitor,
    Fast,
    GetkeysApi,
    NoCluster,
}

impl fmt::Display for CommandStrFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Write => write!(f, "write"),
            Readonly => write!(f, "readonly"),
            Admin => write!(f, "admin"),
            DenyOOM => write!(f, "deny-oom"),
            DenyScript => write!(f, "deny-script"),
            AllowLoading => write!(f, "allow-loading"),
            Pubsub => write!(f, "pubsub"),
            Random => write!(f, "random"),
            AllowStale => write!(f, "allow-stale"),
            NoMonitor => write!(f, "no-monitor"),
            Fast => write!(f, "fast"),
            GetkeysApi => write!(f, "getkeys-api"),
            NoCluster => write!(f, "no-cluster"),
        }
    }
}

impl Add for CommandStrFlags {
    type Output = String;
    fn add(self, other: Self) -> Self::Output {
        format!("{} {}", self, other)
    }
}

/// wrap RedisModule_Milliseconds
pub fn milliseconds() -> Duration {
    unimplemented!()
}