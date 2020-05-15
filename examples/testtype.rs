use iredismodule::io::{Digest, IO};
use iredismodule::key::KeyType;
use iredismodule::prelude::*;
use iredismodule::rtype::TypeMethod;
use iredismodule_macros::{rcmd, rtypedef};

macro_rules! check {
    ($cond:expr) => {
        if $cond {
        } else {
            return Err(Error::new(format!("failed at line {}", line!())));
        }
    };
    ($cond:expr, $desc:expr) => {
        if $cond {
        } else {
            return Err(Error::new(format!("{} at line {}", $desc, line!())));
        }
    };
}

#[derive(Debug, PartialEq)]
pub struct MyType {
    pub v1: u64,
    pub v2: i64,
    pub v3: String,
    pub v4: f64,
    pub v5: f32,
}

#[rtypedef("mytype123", 0)]
impl TypeMethod for MyType {
    fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
        println!("mytype123 load rdb");
        if encver != 0 {
            return None;
        }
        let v1 = io.load_unsigned();
        let v2 = io.load_signed();
        let v3 = io.load_string();
        let v4 = io.load_double();
        let v5 = io.load_float();

        Some(Box::new(MyType { v1, v2, v3, v4, v5 }))
    }
    fn rdb_save(&self, io: &mut IO) {
        println!("mytype123 save rdb");
        io.save_unsigned(self.v1);
        io.save_signed(self.v2);
        io.save_string(self.v3.as_str());
        io.save_double(self.v4);
        io.save_float(self.v5);
    }
    fn free(_: Box<Self>) {
        println!("mytype123 free")
    }
    fn mem_usage(&self) -> usize {
        println!("mytype123 check mem usage");
        std::mem::size_of::<Self>()
    }
    fn digest(&self, digest: &mut Digest) {
        println!("mytype123 digest");
        digest.add_integer(self.v1 as i64);
        digest.add_integer(self.v2);
        digest.add_string(self.v3.as_str());
        digest.add_string(self.v4.to_string());
        digest.add_string(self.v5.to_string());
        digest.end_sequeue()
    }
    fn aof_rewrite<T: AsRef<str>>(&self, io: &mut IO, key: T) {
        println!("mytype123 aof rewrite");
        io.emit_aof(
            "test.set_type".to_owned(),
            &[
                key.as_ref().to_string(),
                self.v1.to_string(),
                self.v2.to_string(),
                self.v3.to_string(),
                self.v4.to_string(),
                self.v5.to_string(),
            ],
        )
    }
}

#[rcmd("test.set_type", "write deny-oom", 1, 1, 1)]
fn test_set_type(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 7);
    let key = ctx.open_write_key(&args[1]);
    key.check_module_type(&MYTYPE123)?;
    let value = MyType {
        v1: args[2].get_integer()? as u64,
        v2: args[3].get_integer()?,
        v3: args[4].to_string(),
        v4: args[5].to_string().parse::<f64>()?,
        v5: args[6].to_string().parse::<f32>()?,
    };
    key.set_value(&MYTYPE123, value)?;
    Ok("OK".into())
}

#[rcmd("test.get_type", "readonly")]
fn test_get_type(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    let key = ctx.open_read_key(&args[1]);
    check!(key.get_type() == KeyType::Module);
    let exist = key.check_module_type(&MYTYPE123)?;
    let value: &mut MyType = key.get_value(&MYTYPE123)?.unwrap();
    check!(exist);
    Ok(Value::Array(vec![
        (value.v1 as i64).into(),
        value.v2.into(),
        value.v3.as_str().into(),
        value.v4.into(),
        (value.v5 as f64).into(),
    ]))
}

#[rcmd("test.type", "readonly")]
fn test_type(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.call(
        "test.set_type",
        None,
        &["test:type", "123", "-321", "abc", "1.23", "3.21"],
    )?;
    let reply = ctx.call("test.get_type", None, &["test:type"])?;
    check!(reply.get_array_element(0).unwrap().get_integer().unwrap() as u64 == 123);
    check!(reply.get_array_element(1).unwrap().get_integer().unwrap() == -321);
    check!(reply.get_array_element(2).unwrap().get_string().unwrap() == "abc".to_string());
    check!(reply.get_array_element(3).unwrap().get_double().unwrap() == 1.23);
    check!(reply.get_array_element(4).unwrap().get_double().unwrap() as f32 == 3.21f32);
    Ok("OK".into())
}

define_module! {
    name: "testtype",
    version: 1,
    data_types: [
        MYTYPE123,
    ],
    init_funcs: [],
    commands: [
        test_set_type_cmd,
        test_get_type_cmd,

        test_type_cmd,
    ]
}
