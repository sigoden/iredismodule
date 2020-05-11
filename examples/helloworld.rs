use redismodule::{assert_len, define_module};
use rand::random;
use std::time::Duration;
use redismodule_macros::{rcommand, rcall};

use redismodule::{
    parse_args, raw, Context, Error, ListPosition, RResult, StatusCode, RStr, Value, KeyType, ArgvFlags,
    HashGetFlag, HashSetFlag, ZsetRangeDirection,
};
use std::os::raw::c_int;

#[rcommand(name="hello.simple",flags="readonly",first_key=0,last_key=0,key_step=0)]
fn hello_simple(ctx: &Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

#[rcommand(name="hello.push.native",flags="write deny-oom",first_key=1,last_key=1,key_step=1)]
fn hello_push_native(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let mut key = ctx.open_write_key(&args[1]);
    key.list_push(ListPosition::Tail, &args[2])?;
    let len = key.value_length();
    Ok(len.into())
}

#[rcommand(name="hello.push.call",flags="write deny-oom",first_key=1,last_key=1,key_step=1)]
fn hello_push_call(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let call_args: Vec<&RStr> = args.iter().skip(1).collect();
    ctx.call("RPUSH", ArgvFlags::new(), &call_args).into()
}

#[rcommand(name="hello.push.call2",flags="write deny-oom",first_key=1,last_key=1,key_step=1)]
fn hello_push_call2(ctx: &Context, args: Vec<RStr>) -> RResult {
    hello_push_call(ctx, args)
}

#[rcommand(name="hello.push.sum.len",flags="readonly",first_key=1,last_key=1,key_step=1)]
fn hello_list_sum_len(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let call_args = [&args[1].to_str()?, "0", "-1"];
    let reply = ctx.call_with_str_args("LRANGE", ArgvFlags::new(), &call_args);

    let elem_len = reply.get_length();
    let str_len: usize =  (0..elem_len).map(|v| reply.get_array_element(v).get_length()).sum();
    Ok(Value::from(str_len))
}

#[rcommand(name="hello.list.splice",flags="write deny-oom",first_key=1,last_key=2,key_step=1)]
fn hello_list_splice(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let mut src_key = ctx.open_write_key(&args[1]);
    let mut dest_key = ctx.open_write_key(&args[2]);
    src_key.verify_type(KeyType::List, true)?;
    dest_key.verify_type(KeyType::List, true)?;
    let count = args[3].get_positive_integer().map_err(|_| Error::generic("ERR invalid count"))?;
    for _ in 0..count {
        let ele = src_key.list_pop(ListPosition::Tail);
        match ele {
            Err(_) => break,
            Ok(v) => {
                dest_key.list_push(ListPosition::Head, &v)?;
            }
        }
    }
    let len = src_key.value_length();
    Ok(len.into())
}

#[rcommand(name="hello.list.splice.auto",flags="write deny-oom",first_key=1,last_key=2,key_step=1)]
fn hello_list_splice_auto(ctx: &Context, args: Vec<RStr>) -> RResult {
    hello_list_splice(ctx, args)
}

#[rcommand(name="hello.rand.array",flags="readonly",first_key=0,last_key=0,key_step=0)]
fn hello_rand_array(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let count = args[1].get_positive_integer().map_err(|_| Error::generic("ERR invalid count"))?;
    let value: Vec<Value> = (0..count).map(|_| random::<i64>().into()).collect();
    Ok(Value::Array(value))
}

#[rcommand(name="hello.repl1",flags="readonly",first_key=0,last_key=0,key_step=0)]
fn hello_repl1(ctx: &Context, args: Vec<RStr>) -> RResult {
    ctx.replicate_with_str_args("ECHO", ArgvFlags::new(), &["foo"])?;
    ctx.call_with_str_args("INCR", ArgvFlags::new(), &["foo"]);
    ctx.call_with_str_args("INCR", ArgvFlags::new(), &["bar"]);
    Ok(0i64.into())
}

#[rcommand(name="hello.repl2",flags="write",first_key=1,last_key=1,key_step=1)]
fn hello_repl2(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let mut key = ctx.open_write_key(&args[1]);
    key.verify_type(KeyType::List, false)?;
    let list_len = key.value_length();
    let mut sum = 0;
    for _ in 0..list_len {
        let ele = key.list_pop(ListPosition::Tail)?;
        let mut val = ele.get_long_long().unwrap_or(0);
        val += 1;
        sum += val;
        let new_ele = ctx.create_string(&val.to_string());
        key.list_push(ListPosition::Head, &new_ele)?;
    }
    ctx.replicate_verbatim();
    Ok(sum.into())
}


#[rcommand(name="hello.toggle.case",flags="write",first_key=1,last_key=1,key_step=1)]
fn hello_toggle_case(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let mut key = ctx.open_write_key(&args[1]);
    key.verify_type(KeyType::String, true)?;
    if key.get_type() == KeyType::String {
        let value = key.string_get()?;
        let value = value
            .to_str()?
            .chars()
            .map(|v| {
                if v.is_ascii_uppercase() {
                    v.to_ascii_lowercase()
                } else {
                    v.to_ascii_uppercase()
                }
            })
            .collect::<String>();
        key.string_set(&ctx.create_string(&value))?;
    }
    ctx.replicate_verbatim();
    Ok("OK".into())
}

#[rcommand(name="hello.more.expire",flags="write",first_key=1,last_key=1,key_step=1)]
fn hello_more_expire(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let addms = args[2].get_long_long().map_err(|e| Error::generic("invalid expire time"))?;
    let mut key = ctx.open_write_key(&args[1]);
    let expire = key.get_expire();
    if let Some(d) = expire {
        ctx.log_debug(&format!("current duration {}", d.as_secs()));
        let new_d = d.checked_add(Duration::from_millis(addms as u64)).unwrap();
        key.set_expire(new_d)?;
    } else {
        ctx.log_debug(&format!("current no duration"));
    }
    Ok("OK".into())
}

#[rcommand(name="hello.zsumrange",flags="readonly",first_key=1,last_key=1,key_step=1)]
fn hello_zsumrange(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let mut key = ctx.open_write_key(&args[1]);
    key.verify_type(KeyType::ZSet, false)?;
    let tail_args = args
        .iter()
        .skip(2)
        .map(|v| v.get_long_long()).collect::<Result<Vec<i64>, Error>>()
        .map_err(|e| Error::generic("invalid range"))?;
    let score_start = tail_args[0] as f64;
    let score_end = tail_args[1] as f64;
    let v1 = key.zset_score_range(ZsetRangeDirection::FristIn, score_start, score_end, false, false)?;
    let v2 = key.zset_score_range(ZsetRangeDirection::LastIn, score_start, score_end, false, false)?;
    let score1: f64 = v1.iter().map(|v| v.1).sum();
    let score2: f64 = v2.iter().map(|v| v.1).sum();
    let result: Vec<Value> = vec![score1.into(), score2.into()];
    Ok(Value::Array(result))
}

#[rcommand(name="hello.lexrange",flags="readonly",first_key=1,last_key=1,key_step=1)]
fn hello_lexrange(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.verify_type(KeyType::ZSet, false)?;
    let v = key.zset_lex_range(ZsetRangeDirection::FristIn, &args[2], &args[3])?;
    let result: Vec<Value> = v.iter().map(|v| v.0.to_str().unwrap().to_owned().into()).collect();
    Ok(Value::Array(result))
}

#[rcommand(name="hello.hcopy",flags="write deny-oom",first_key=1,last_key=1,key_step=1)]
fn hello_hcopy(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.verify_type(KeyType::ZSet, true)?;
    let old_val = key.hash_get(HashGetFlag::Normal, &args[2])?;
    if let Some(v) = &old_val {
        ctx.log_debug(&format!("old_val is {}", v.to_str()?));
        key.hash_set(HashSetFlag::Normal, &args[3], v)?;
        ctx.log_debug(&format!("new_val is {}", v.to_str()?));
    }
    let ret: i64 = match &old_val {
        Some(_) => 1,
        None => 0,
    };
    Ok(ret.into())
}

#[rcommand(name="hello.leftpad",flags="",first_key=0,last_key=0,key_step=0)]
fn hello_leftpad(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let pad_len = args[2].get_positive_integer().map_err(|_| Error::generic("ERR invalid padding length"))? as usize;
    let the_str: &str = args[1].to_str()?;
    let the_char: &str = args[3].to_str()?;
    if the_str.len() >= pad_len  {
        return Ok(the_str.into());
    }
    if the_char.len() != 1 {
        return Err(Error::generic("padding must be a single char"));
    }
    let the_char = the_char.chars().nth(0).unwrap();
    let mut pad_str = (0..(pad_len - the_str.len())).map(|_| the_char).collect::<String>();
    pad_str.push_str(the_str);
    Ok(pad_str.into())
}

#[rcall]
fn init(ctx: &mut Context, args: Vec<RStr>) -> Result<(), Error> {
    ctx.log_debug(&format!(
        "Module loaded with ARGV[{}] = {:?}\n",
        args.len(),
        args.iter().map(|v| v.to_str().unwrap().to_owned()).collect::<Vec<String>>()
    ));
    Ok(())
}


define_module! {
    name: "hello",
    version: 1,
    data_types: [],
    init_funcs: [
        init_c,
    ],
    commands: [
        create_hello_simple,
        create_hello_push_native,
        create_hello_push_call,
        create_hello_push_call2,
        create_hello_list_sum_len,
        create_hello_list_splice,
        create_hello_list_splice_auto,
        create_hello_rand_array,
        create_hello_repl1,
        create_hello_repl2,
        create_hello_toggle_case,
        create_hello_more_expire,
        create_hello_zsumrange,
        create_hello_lexrange,
        create_hello_hcopy,
        create_hello_leftpad,
    ],
}
