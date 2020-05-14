use iredismodule::prelude::*;
use iredismodule_macros::rcmd;
use iredismodule::key::KeyType;
use iredismodule::key::{HashSetFlag, ListPosition};

macro_rules! check {
    ($cond:expr, $msg:expr) => {
        if $cond { } else { return Err(Error::new($msg)) }
    };
}

#[rcmd("test.select_db")]
fn test_change_db(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.select_db(1)?;
    let db = ctx.get_select_db();
    check!(db == 1, "select_db failed");
    ctx.select_db(0)?;
    Ok("OK".into())
}

#[rcmd("test.is_module_busy")]
fn test_is_module_busy(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let not_busy_module = iredismodule::is_module_busy("meaninglessmodule");
    check!(!not_busy_module, "is_module_busy failed");
    let busy_module = iredismodule::is_module_busy("test");
    check!(busy_module, "is_module_busy failed");
    Ok("OK".into())
}

#[rcmd("test.key_type")]
fn test_key_type(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let key_string = ctx.open_write_key(rstr!("key_string"));
    // let key_string = ctx.open_write_key(rstr!("key_string"));
    key_string.hash_set(HashSetFlag::None, rstr!("abc"), Some(rstr!("123")))?;
    check!(key_string.get_type() == KeyType::String, "get_type failed".into());
    let key_list = ctx.open_write_key(rstr!("key_list"));
    key_list.list_push(ListPosition::Head, rstr!("abc"))?;
    key_list.list_push(ListPosition::Tail, rstr!("def"))?;
    check!(key_list.get_type() == KeyType::List, "get_type failed");
    let key_hash = ctx.open_write_key(rstr!("key_hash"));
    key_hash.hash_set(HashSetFlag::None, rstr!("field1"), Some(rstr!("value1")))?;
    ctx.call_str("set", CallFlags::None, &["key_string", "abc"])?;
    ctx.call_str("", CallFlags::None, &["key_list", "abc"])?;
    
    Ok("OK".into())
}

define_module! {
    name: "test",
    version: 1,
    data_types: [],
    init_funcs: [],
    commands: [
        test_change_db_cmd,
        test_is_module_busy_cmd,
        test_key_type_cmd,
    ]
}
