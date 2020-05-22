use iredismodule::call_reply::ReplyType;
use iredismodule::io::{Digest, IO};
use iredismodule::key::KeyType;
use iredismodule::key::{ListPosition, ZsetRangeDirection};
use iredismodule::prelude::*;
use iredismodule::rtype::TypeMethod;
use iredismodule_macros::{rcmd, rtypedef};
use std::time::Duration;

/// Generate RString for String or str
#[macro_export]
macro_rules! rstr {
    ($value:expr) => {
        RString::from_str($value)
    };
}

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
#[rcmd("test.clear_keys")]
fn test_clear_keys(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("keys", None, &["test:*"])?;
    let result: RResult = reply.into();
    ctx.notice(format!("{:?}", &result));
    let mut keys: Vec<String> = vec![];
    if let Value::Array(v) = result? {
        v.iter().for_each(|elem| {
            if let Value::BulkString(key) = elem {
                keys.push(std::str::from_utf8(key).unwrap().to_string());
            }
        })
    }
    ctx.notice(format!("{:?}", &keys));
    if keys.len() > 0 {
        ctx.call("del".to_string(), None, &keys)?;
    }
    Ok("OK".into())
}

#[rcmd("test.key")]
fn test_key(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let key_string = ctx.open_write_key(&rstr!("test:key_string"));
    key_string.string_set(&rstr!("abc"))?;
    check!(key_string.get_type() == KeyType::String);
    let key_list = ctx.open_write_key(&rstr!("test:key_list"));
    key_list.list_push(ListPosition::Head, &rstr!("abc"))?;
    key_list.list_push(ListPosition::Tail, &rstr!("def"))?;
    key_list.list_push(ListPosition::Tail, &rstr!("ghi"))?;
    check!(key_list.get_type() == KeyType::List);
    let key_hash = ctx.open_write_key(&rstr!("test:key_hash"));
    key_hash.hash_set(None, &rstr!("field1"), Some(&rstr!("value1")))?;
    check!(key_hash.get_type() == KeyType::Hash);
    ctx.call("SADD", None, &["test:key_set", "abc", "def", "ghi"])?;
    let key_set = ctx.open_read_key(&rstr!("test:key_set"));
    check!(key_set.get_type() == KeyType::Set);
    let key_zset = ctx.open_write_key(&rstr!("test:key_zset"));
    key_zset.zset_add(0.0, &rstr!("abc"), None)?;
    key_zset.zset_add(2.0, &rstr!("ghi"), None)?;
    key_zset.zset_add(3.0, &rstr!("def"), None)?;
    key_zset.zset_incrby(&rstr!("abc"), 1.0, None)?;
    check!(key_zset.zset_score(&rstr!("abc"))? == 1.0);

    let range1 = key_zset.zset_score_range(ZsetRangeDirection::FristIn, 1.0, 3.0, true, false)?;
    check!(range1[0].0.to_string() == "ghi" && range1[0].1 == 2.0);
    check!(range1[1].0.to_string() == "def" && range1[1].1 == 3.0);
    let range2 = key_zset.zset_lex_range(ZsetRangeDirection::LastIn, &rstr!("[a"), &rstr!("[z"))?;
    check!(range2[0].0.to_string() == "def" && range2[0].1 == 3.0);
    check!(range2[1].0.to_string() == "ghi" && range2[1].1 == 2.0);
    key_zset.zset_rem(&rstr!("ghi"))?;
    let length_zset = key_zset.value_length();
    check!(length_zset == 2);

    let key_nonexist = ctx.open_read_key(&rstr!("test:key_nonexist"));
    check!(key_zset.get_type() == KeyType::ZSet);
    check!(key_string.check_type(KeyType::String).is_ok());
    check!(key_string.check_type(KeyType::Hash).is_err());
    check!(key_nonexist.check_type(KeyType::Empty).is_ok());
    check!(key_nonexist.check_type(KeyType::String).ok() == Some(false));

    let value_string = key_string.string_get()?;
    check!(value_string.to_str().unwrap() == "abc");
    let value_list_head = key_list.list_pop(ListPosition::Head)?;
    check!(value_list_head.to_str().unwrap() == "abc");
    let value_hash = key_hash.hash_get(&rstr!("field1"))?;
    check!(value_hash.unwrap().to_str().unwrap() == "value1");
    let exist_hash = key_hash.hash_check(&rstr!("field1"))?;
    check!(exist_hash == true);

    check!(key_string.get_keyname().to_str().unwrap() == "test:key_string");

    let key_expire = ctx.open_write_key(&rstr!("test:expire"));
    key_expire.string_set(&rstr!("abc"))?;
    key_expire.set_expire(Duration::from_secs(30))?;
    let expire_ms = key_expire.get_expire().unwrap();
    check!(expire_ms.as_secs() <= 30 && expire_ms.as_secs() > 0);

    let key_delete = ctx.open_write_key(&rstr!("test:key_delete"));
    key_delete.string_set(&rstr!("abc"))?;
    key_delete.delete()?;
    check!(key_delete.is_empty());

    let key_unlink = ctx.open_write_key(&rstr!("test:key_unlink"));
    key_unlink.string_set(&rstr!("abc"))?;
    key_unlink.unlink()?;

    Ok("OK".into())
}

#[rcmd("test.reply_integer")]
fn test_reply_integer(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok(Value::Integer(123))
}

#[rcmd("test.reply_double")]
fn test_reply_float(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok(Value::Double(1.23))
}

#[rcmd("test.reply_string")]
fn test_reply_string(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok(Value::String("abc".into()))
}

#[rcmd("test.reply_bulk_string")]
fn test_reply_bulk_string(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok(Value::BulkString(vec![1u8, 2u8, 3u8]))
}

#[rcmd("test.reply_array")]
fn test_reply_array(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let data: Vec<Value> = (0..10).map(|v| Value::Integer(v)).collect();
    Ok(Value::Array(data))
}

#[rcmd("test.reply_null")]
fn test_reply_null(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Ok(Value::Null)
}
#[rcmd("test.reply_error")]
fn test_reply_error(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    Err(Error::WrongArity)
}

#[rcmd("test.call_reply")]
fn test_call_reply(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let call_reply_string = ctx.call("test.reply_string", None, &[])?;
    check!(call_reply_string.get_type() == ReplyType::String);
    check!(call_reply_string.get_string().unwrap() == "abc".to_string());
    let call_reply_integer = ctx.call("test.reply_integer", None, &[])?;
    check!(call_reply_integer.get_type() == ReplyType::Integer);
    check!(call_reply_integer.get_integer().unwrap() == 123);
    let call_reply_double = ctx.call("test.reply_double", None, &[])?;
    check!(call_reply_double.get_type() == ReplyType::String);
    check!(call_reply_double.get_double().unwrap() == 1.23);
    let call_reply_bulk_string = ctx.call("test.reply_bulk_string", None, &[])?;
    check!(call_reply_bulk_string.get_type() == ReplyType::String);
    check!(call_reply_bulk_string
        .get_bulk_string()
        .unwrap()
        .iter()
        .zip([1u8, 2u8, 3u8].iter())
        .all(|(x, y)| x == y));
    let call_reply_array = ctx.call("test.reply_array", None, &[])?;
    check!(call_reply_array.get_length() == 10);
    check!(
        call_reply_array
            .get_array_element(0)
            .unwrap()
            .get_integer()
            .unwrap()
            == 0
    );
    check!(
        call_reply_array
            .get_array_element(9)
            .unwrap()
            .get_integer()
            .unwrap()
            == 9
    );
    check!(call_reply_array.get_type() == ReplyType::Array);
    let call_reply_null = ctx.call("test.reply_null", None, &[])?;
    check!(call_reply_null.get_type() == ReplyType::Null);
    let call_reply_error = ctx.call("test.reply_error", None, &[])?;
    check!(call_reply_error.get_type() == ReplyType::Error);
    Ok("OK".into())
}

#[rcmd("test.reply_value")]
fn test_reply_value(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let values: Vec<Value> = vec![
        "abc".into(),
        vec![1u8, 2u8, 3u8].into(),
        123i64.into(),
        1.23.into(),
        Value::from(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]),
        ().into(),
    ];
    Ok(values.into())
}

#[rcmd("test.value")]
fn test_value(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let call_reply = ctx.call("test.reply_value", None, &[])?;
    let value0: RResult = call_reply.get_array_element(0).unwrap().into();
    if let Value::String(_) = value0.unwrap() {
    } else {
        check!(false)
    };
    let value1: RResult = call_reply.get_array_element(1).unwrap().into();
    if let Value::BulkString(_) = value1.unwrap() {
    } else {
        check!(false)
    };
    let value2: RResult = call_reply.get_array_element(2).unwrap().into();
    if let Value::Integer(_) = value2.unwrap() {
    } else {
        check!(false)
    };
    let value3: RResult = call_reply.get_array_element(3).unwrap().into();
    if let Value::BulkString(_) = value3.unwrap() {
    } else {
        check!(false)
    };
    let value4: RResult = call_reply.get_array_element(4).unwrap().into();
    if let Value::Array(_) = value4.unwrap() {
    } else {
        check!(false)
    };
    let value5: RResult = call_reply.get_array_element(5).unwrap().into();
    if let Value::Null = value5.unwrap() {
    } else {
        check!(false)
    };
    Ok("OK".into())
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
    if args.len() != 7 {
        return Err(Error::WrongArity);
    }
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

#[rcmd("test.misc")]
fn test_misc(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.select_db(1)?;
    let db = ctx.get_select_db();
    check!(db == 1);
    ctx.select_db(0)?;

    let not_busy = iredismodule::is_module_busy("meaninglessmodule");
    check!(!not_busy);
    let busy = iredismodule::is_module_busy("testmodule");
    check!(busy);

    Ok("OK".into())
}

#[rcmd("test.example_simple")]
fn test_example_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.call("hello.simple", None, &[])?;
    Ok("OK".into())
}

#[rcmd("test.example_helloworld")]
fn test_example_helloworld(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hello.simple", None, &[])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.native", None, &["test:helloword:key1", "1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.call", None, &["test:helloword:key1", "2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.call2", None, &["test:helloword:key1", "3"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.push.sum.len", None, &["test:helloword:key1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call(
        "hello.list.splice",
        None,
        &["test:helloword:key1", "test:helloword:key2", "2"],
    )?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.rand.array", None, &["5"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hello.repl1", None, &[])?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.repl2", None, &["test:helloword:key2"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call("set", None, &["test:helloworld:key3", "abc"])?;
    let reply = ctx.call("hello.toggle.case", None, &["test:helloworld:key3"])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call(
        "hello.more.expire",
        None,
        &["test:helloworld:key3", "10000"],
    )?;
    check!(reply.get_type() == ReplyType::String);
    let value = [
        "test:helloworld:key4",
        "1",
        "a",
        "2",
        "b",
        "3",
        "c",
        "4",
        "d",
    ];
    ctx.call("zadd", None, &value)?;
    let reply = ctx.call("hello.zsumrange", None, &["test:helloworld:key4", "1", "4"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hello.lexrange", None, &["test:helloworld:key4", "-", "[c"])?;
    check!(reply.get_type() == ReplyType::Array);
    ctx.call("hset", None, &["test:helloworld:key5", "field1", "abc"])?;
    let reply = ctx.call(
        "hello.hcopy",
        None,
        &["test:helloworld:key5", "field1", "field2"],
    )?;
    check!(reply.get_type() == ReplyType::Integer);
    let reply = ctx.call("hello.leftpad", None, &["123", "8", "0"])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

#[rcmd("test.example_hellotype")]
fn test_example_hellotype(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hellotype.insert", None, &["test:hellotype:key1", "1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "2"])?;
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "3"])?;
    ctx.call("hellotype.insert", None, &["test:hellotype:key1", "4"])?;
    let reply = ctx.call("hellotype.range", None, &["test:hellotype:key1", "1", "2"])?;
    check!(reply.get_type() == ReplyType::Array);
    let reply = ctx.call("hellotype.len", None, &["test:hellotype:key1"])?;
    check!(reply.get_type() == ReplyType::Integer);
    ctx.call(
        "hellotype.brange",
        None,
        &["test:hellotype:key1", "1", "2", "5"],
    )?;
    Ok("OK".into())
}

#[rcmd("test.example_hellotimer")]
fn test_example_hellotimer(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("hellotimer.timer", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

#[rcmd("test.example_helloblock")]
fn test_example_helloblock(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.call("hello.block", None, &["1", "2"])?;
    ctx.call("hello.block", None, &["2", "1"])?;
    ctx.call("hello.keys", None, &["2", "1"])?;
    Ok("OK".into())
}

#[rcmd("test.all")]
fn test_all(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let reply = ctx.call("test.clear_keys", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.key", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.call_reply", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.value", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.type", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.misc", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_simple", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_helloworld", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_hellotype", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_hellotimer", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    let reply = ctx.call("test.example_helloblock", None, &[])?;
    check!(reply.get_type() == ReplyType::String);
    Ok("OK".into())
}

define_module! {
    name: "testmodule",
    version: 1,
    data_types: [
        MYTYPE123,
    ],
    init_funcs: [],
    commands: [
        test_clear_keys_cmd,
        test_key_cmd,
        test_reply_integer_cmd,
        test_reply_float_cmd,
        test_reply_string_cmd,
        test_reply_bulk_string_cmd,
        test_reply_array_cmd,
        test_reply_null_cmd,
        test_reply_error_cmd,
        test_reply_value_cmd,
        test_call_reply_cmd,
        test_value_cmd,
        test_set_type_cmd,
        test_get_type_cmd,
        test_type_cmd,
        test_misc_cmd,
        test_example_simple_cmd,
        test_example_helloworld_cmd,
        test_example_hellotype_cmd,
        test_example_hellotimer_cmd,
        test_example_helloblock_cmd,
        test_all_cmd,
    ]
}
