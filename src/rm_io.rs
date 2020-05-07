use crate::raw;
use crate::{CmdFmtFlags, Str};
pub struct IO {

}

impl IO {
    pub fn save_unsigned(&self, value: u64) {
        unimplemented!()
    }
    pub fn load_unsigned(&self) -> u64 {
        unimplemented!()
    }
    pub fn save_signed(&self, value: i64) {
        unimplemented!()
    }
    pub fn load_signed(&self) -> i64 {
        unimplemented!()
    }
    pub fn save_string(&self, value: &Str) {
        unimplemented!()
    }
    pub fn load_string(&self) -> Str {
        unimplemented!()
    }
    pub fn save_string_buffer(&self, value: &str) {
        unimplemented!()
    }
    pub fn load_string_buffer(&self) -> String {
        unimplemented!()
    }
    pub fn save_double(&self, value: f64) {
        unimplemented!()
    }
    pub fn load_double(&self) -> f64 {
        unimplemented!()
    }
    pub fn save_float(&self, value: f32) {
        unimplemented!()
    }
    pub fn load_float(&self) -> f32  {
        unimplemented!()
    }
    pub fn emit_aof(&self, command: &str, args: &[&str], flags: &[CmdFmtFlags]) {
        unimplemented!()
    }
    pub fn log(&self, level: LogLevel, message: &str) {
        unimplemented!()
    }
    pub fn log_io_error(&self, level: LogLevel, message: &str) {
        unimplemented!()
    }
}

pub enum LogLevel {

}


pub struct Digest {
    pub inner: raw::RedisModuleDigest,
}

impl Digest {
    pub fn add_string_buffer(&mut self, ele: &str) {
        unimplemented!()
    }
    pub fn add_long_long(&mut self, ll: i128) {
        unimplemented!()
    }
    pub fn end_sequeue(&mut self) {
        unimplemented!()
    }
}