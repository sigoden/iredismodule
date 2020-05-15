use iredismodule::prelude::*;
use iredismodule_macros::rcmd;
use iredismodule::key::KeyType;
use iredismodule::key::{ListPosition, ZsetRangeDirection};
use iredismodule::call_reply::ReplyType;
use std::time::Duration;


macro_rules! check {
    ($cond:expr) => {
        if $cond { } else { return Err(Error::new(format!("failed at line {}", line!()))) }
    };
    ($cond:expr, $desc:expr) => {
        if $cond { } else { return Err(Error::new(format!("{} at line {}", $desc, line!()))) }
    };
}

#[rcmd("test.misc")]
fn test_misc(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.select_db(1)?;
    let db = ctx.get_select_db();
    check!(db == 1);
    ctx.select_db(0)?;

    let not_busy = iredismodule::is_module_busy("meaninglessmodule");
    check!(!not_busy);
    let busy = iredismodule::is_module_busy("testbase");
    check!(busy);

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
    check!(key_string.assert_type(KeyType::String, false).is_ok());
    check!(key_string.assert_type(KeyType::Hash, false).is_err());
    check!(key_nonexist.assert_type(KeyType::Empty, false).is_ok());
    check!(key_nonexist.assert_type(KeyType::String, false).is_err());
    check!(key_nonexist.assert_type(KeyType::String, true).is_ok());

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
    check!(call_reply_string.get_string() == "abc".to_string());
    let call_reply_integer = ctx.call("test.reply_integer", None, &[])?;
    check!(call_reply_integer.get_type() == ReplyType::Integer);
    check!(call_reply_integer.get_integer() == 123);
    let call_reply_double = ctx.call("test.reply_double", None, &[])?;
    check!(call_reply_double.get_type() == ReplyType::String);
    check!(call_reply_double.get_double() == 1.23);
    let call_reply_bulk_string = ctx.call("test.reply_bulk_string", None, &[])?;
    check!(call_reply_bulk_string.get_type() == ReplyType::String);
    check!(call_reply_bulk_string.get_bulk_string().iter().zip([1u8,2u8,3u8].iter()).all(|(x, y)| x == y));
    let call_reply_array = ctx.call("test.reply_array", None, &[])?;
    check!(call_reply_array.get_length() == 10);
    check!(call_reply_array.get_array_element(0).unwrap().get_integer() == 0);
    check!(call_reply_array.get_array_element(9).unwrap().get_integer() == 9);
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
        Value::from(vec![Value::from(1i64),Value::from(2i64), Value::from(3i64)]),
        ().into(),
    ];
    Ok(values.into())
}

#[rcmd("test.value")]
fn test_value(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let call_reply = ctx.call("test.reply_value", None, &[])?;
    let value0: RResult = call_reply.get_array_element(0).unwrap().into();
    if let Value::String(_) = value0.unwrap() {} else { check!(false) };
    let value1: RResult = call_reply.get_array_element(1).unwrap().into();
    if let Value::BulkString(_) = value1.unwrap() {} else { check!(false) };
    let value2: RResult = call_reply.get_array_element(2).unwrap().into();
    if let Value::Integer(_) = value2.unwrap() {} else { check!(false) };
    let value3: RResult = call_reply.get_array_element(3).unwrap().into();
    if let Value::BulkString(_) = value3.unwrap() {} else { check!(false) };
    let value4: RResult = call_reply.get_array_element(4).unwrap().into();
    if let Value::Array(_) = value4.unwrap() {} else { check!(false) };
    let value5: RResult = call_reply.get_array_element(5).unwrap().into();
    if let Value::Null = value5.unwrap() {} else { check!(false) };
    Ok("OK".into())
    
}

define_module! {
    name: "testbase",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        test_misc_cmd,
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
    ]
}
