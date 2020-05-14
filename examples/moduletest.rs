use iredismodule::prelude::*;
use iredismodule_macros::rcmd;
use iredismodule::key::KeyType;
use iredismodule::key::{HashSetFlag, ListPosition, ZaddInputFlag};

macro_rules! check {
    ($cond:expr) => {
        if $cond { } else { return Err(Error::new(format!("failed at line {}", line!()))) }
    };
    ($cond:expr, $desc:expr) => {
        if $cond { } else { return Err(Error::new(format!("{} at line {}", $desc, line!()))) }
    };
}

#[rcmd("test.select_db")]
fn test_change_db(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.select_db(1)?;
    let db = ctx.get_select_db();
    check!(db == 1);
    ctx.select_db(0)?;
    Ok("OK".into())
}

#[rcmd("test.is_module_busy")]
fn test_is_module_busy(_ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let not_busy = iredismodule::is_module_busy("meaninglessmodule");
    check!(!not_busy);
    let busy = iredismodule::is_module_busy("test");
    check!(busy);
    Ok("OK".into())
}

#[rcmd("test.key_type")]
fn test_key_type(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let key_string = ctx.open_write_key(&rstr!("key_string"));
    key_string.string_set(&rstr!("abc"))?;
    check!(key_string.get_type() == KeyType::String);
    let key_list = ctx.open_write_key(&rstr!("key_list"));
    key_list.list_push(ListPosition::Head, &rstr!("abc"))?;
    key_list.list_push(ListPosition::Tail, &rstr!("def"))?;
    check!(key_list.get_type() == KeyType::List);
    let key_hash = ctx.open_write_key(&rstr!("key_hash"));
    key_hash.hash_set(HashSetFlag::None, &rstr!("field1"), Some(&rstr!("value1")))?;
    check!(key_hash.get_type() == KeyType::Hash);
    ctx.call_str("SADD", CallFlags::None, &["key_set", "abc"])?;
    let key_set = ctx.open_read_key(&rstr!("key_set"));
    check!(key_set.get_type() == KeyType::Set);
    let key_zset = ctx.open_write_key(&rstr!("key_zset"));
    key_zset.zset_add(1.0, &rstr!("field1"), ZaddInputFlag::None)?;
    check!(key_zset.get_type() == KeyType::ZSet);
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
