use iredismodule::key::{KeyType, ListPosition, ZsetRangeDirection};
use iredismodule::prelude::*;
use iredismodule_macros::{rcmd, rwrap};
use rand::random;
use std::time::Duration;

/// HELLO.SIMPLE is among the simplest commands you can implement.
/// It just returns the currently selected DB id, a functionality which is
/// missing in Redis. The command uses two important API calls: one to
/// fetch the currently selected DB, the other in order to send the client
/// an integer reply as response.
#[rcmd("hello.simple")]
fn hello_simple(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    let db = ctx.get_select_db();
    Ok(db.into())
}

/// HELLO.PUSH.NATIVE re-implements RPUSH, and shows the low level modules API
/// where you can "open" keys, make low level operations, create new keys by
/// pushing elements into non-existing keys, and so forth.
///
/// You'll find this command to be roughly as fast as the actual RPUSH
/// command.
#[rcmd("hello.push.native", "write deny-oom", 1, 1, 1)]
fn hello_push_native(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let key = ctx.open_write_key(&args[1]);
    key.list_push(ListPosition::Tail, &args[2])?;
    let len = key.value_length();
    Ok(len.into())
}

/// HELLO.PUSH.CALL implements RPUSH using an higher level approach, calling
/// a Redis command instead of working with the key in a low level way. This
/// approach is useful when you need to call Redis commands that are not
/// available as low level APIs, or when you don't need the maximum speed
/// possible but instead prefer implementation simplicity.
#[rcmd("hello.push.call", "write deny-oom", 1, 1, 1)]
fn hello_push_call(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let call_args: Vec<&RStr> = args.iter().skip(1).collect();
    let args = call_args
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect::<Vec<&str>>();
    ctx.call("RPUSH", None, &args).unwrap().into()
}

/// HELLO.PUSH.CALL2
/// This is exaxctly as HELLO.PUSH.CALL, but shows how we can reply to the
/// client using directly a reply object that Call() returned.
#[rcmd("hello.push.call2", "write deny-oom", 1, 1, 1)]
fn hello_push_call2(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    hello_push_call(ctx, args)
}

/// HELLO.LIST.SUM.LEN returns the total length of all the items inside
/// a Redis list, by using the high level Call() API.
/// This command is an example of the array reply access.
#[rcmd("hello.push.sum.len", "readonly", 1, 1, 1)]
fn hello_list_sum_len(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let call_args = [&args[1].to_str()?, "0", "-1"];
    let reply = ctx.call("LRANGE", None, &call_args)?;

    let elem_len = reply.get_length();
    let str_len: usize = (0..elem_len)
        .map(|v| reply.get_array_element(v).unwrap().get_length())
        .sum();
    Ok(Value::from(str_len))
}

/// HELLO.LIST.SPLICE srclist dstlist count
/// Moves 'count' elements from the tail of 'srclist' to the head of
/// 'dstlist'. If less than count elements are available, it moves as much
/// elements as possible.
#[rcmd("hello.list.splice", "write deny-oom", 1, 2, 1)]
fn hello_list_splice(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let src_key = ctx.open_write_key(&args[1]);
    let dest_key = ctx.open_write_key(&args[2]);
    src_key.assert_type(KeyType::List, true)?;
    dest_key.assert_type(KeyType::List, true)?;
    let count = args[3]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid count"))?;
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

/// HELLO.RAND.ARRAY <count>
/// Shows how to generate arrays as commands replies.
/// It just outputs <count> random numbers.
#[rcmd("hello.rand.array")]
fn hello_rand_array(_ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let count = args[1]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid count"))?;
    let value: Vec<Value> = (0..count).map(|_| random::<i64>().into()).collect();
    Ok(Value::Array(value))
}

/// This is a simple command to test replication. Because of AofAndReplicas flag,
/// the two INCRs get replicated.
/// Also note how the ECHO is replicated in an unexpected position (check
/// comments the function implementation).
#[rcmd("hello.repl1")]
fn hello_repl1(ctx: &mut Context, _args: Vec<RStr>) -> RResult {
    ctx.replicate("ECHO", None, &["test:foo"])?;
    ctx.call("INCR", Some(CallFlag::AofAndReplicas), &["test:foo"])?;
    ctx.call("INCR", Some(CallFlag::AofAndReplicas), &["test:bar"])?;
    Ok(0i64.into())
}

/// Another command to show replication. In this case, we call
/// Context::replicate_verbatim to mean we want just the command to be
/// propagated to slaves / AOF exactly as it was called by the user.
///
/// This command also shows how to work with string objects.
/// It takes a list, and increments all the elements (that must have
/// a numerical value) by 1, returning the sum of all the elements
/// as reply.
///
/// Usage: HELLO.REPL2 <list-key>
#[rcmd("hello.repl2", "write", 1, 1, 1)]
fn hello_repl2(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let key = ctx.open_write_key(&args[1]);
    key.assert_type(KeyType::List, false)?;
    let list_len = key.value_length();
    let mut sum = 0;
    for _ in 0..list_len {
        let ele = key.list_pop(ListPosition::Tail)?;
        let mut val = ele.get_integer().unwrap_or(0);
        val += 1;
        sum += val;
        let new_ele = RString::from_str(&val.to_string());
        key.list_push(ListPosition::Head, &new_ele)?;
    }
    ctx.replicate_verbatim();
    Ok(sum.into())
}

/// This is an example of strings DMA access. Given a key containing a string
/// it toggles the case of each character from lower to upper case or the
/// other way around.
///
/// HELLO.TOGGLE.CASE key
#[rcmd("hello.toggle.case", "write", 1, 1, 1)]
fn hello_toggle_case(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let key = ctx.open_write_key(&args[1]);
    key.assert_type(KeyType::String, true)?;
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
        key.string_set(&RString::from_str(&value))?;
    }
    ctx.replicate_verbatim();
    Ok("OK".into())
}

/// HELLO.MORE.EXPIRE key milliseconds.
///
/// If they key has already an associated TTL, extends it by "milliseconds"
/// milliseconds. Otherwise no operation is performed.
#[rcmd("hello.more.expire", "write", 1, 1, 1)]
fn hello_more_expire(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let addms = args[2]
        .get_integer()
        .map_err(|_e| Error::new("ERR invalid expire time"))?;
    let key = ctx.open_write_key(&args[1]);
    let expire = key.get_expire();
    if let Some(d) = expire {
        ctx.debug(&format!("current duration {}", d.as_secs()));
        let new_d = d.checked_add(Duration::from_millis(addms as u64)).unwrap();
        key.set_expire(new_d)?;
    } else {
        ctx.debug(&format!("current no duration"));
    }
    Ok("OK".into())
}

/// HELLO.ZSUMRANGE key startscore endscore
/// Return the sum of all the scores elements between startscore and endscore.
///
/// The computation is performed two times, one time from start to end and
/// another time backward. The two scores, returned as a two element array,
/// should match.
#[rcmd("hello.zsumrange", "readonly", 1, 1, 1)]
fn hello_zsumrange(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.assert_type(KeyType::ZSet, false)?;
    let tail_args = args
        .iter()
        .skip(2)
        .map(|v| v.get_integer())
        .collect::<Result<Vec<i64>, Error>>()
        .map_err(|_e| Error::new("invalid range"))?;
    let score_start = tail_args[0] as f64;
    let score_end = tail_args[1] as f64;
    let v1 = key.zset_score_range(
        ZsetRangeDirection::FristIn,
        score_start,
        score_end,
        false,
        false,
    )?;
    let v2 = key.zset_score_range(
        ZsetRangeDirection::LastIn,
        score_start,
        score_end,
        false,
        false,
    )?;
    let score1: f64 = v1.iter().map(|v| v.1).sum();
    let score2: f64 = v2.iter().map(|v| v.1).sum();
    let result: Vec<Value> = vec![score1.into(), score2.into()];
    Ok(Value::Array(result))
}

/// HELLO.LEXRANGE key min_lex max_lex min_age max_age
/// This command expects a sorted set stored at key in the following form:
/// - All the elements have score 0.
/// - Elements are pairs of "<name>:<age>", for example "Anna:52".
/// The command will return all the sorted set items that are lexicographically
/// between the specified range (using the same format as ZRANGEBYLEX)
/// and having an age between min_age and max_age.
#[rcmd("hello.lexrange", "readonly", 1, 1, 1)]
fn hello_lexrange(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.assert_type(KeyType::ZSet, false)?;
    let v = key.zset_lex_range(ZsetRangeDirection::FristIn, &args[2], &args[3])?;
    let result: Vec<Value> = v
        .iter()
        .map(|v| v.0.to_str().unwrap().to_owned().into())
        .collect();
    Ok(Value::Array(result))
}

/// HELLO.HCOPY key srcfield dstfield
/// This is just an example command that sets the hash field dstfield to the
/// same value of srcfield. If srcfield does not exist no operation is
/// performed.
///
/// The command returns 1 if the copy is performed (srcfield exists) otherwise
/// 0 is returned.
#[rcmd("hello.hcopy", "write deny-oom", 1, 1, 1)]
fn hello_hcopy(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.assert_type(KeyType::Hash, true)?;
    let old_val = key.hash_get(&args[2])?;
    if let Some(v) = &old_val {
        key.hash_set(None, &args[3], Some(v))?;
    }
    let ret: i64 = match &old_val {
        Some(_) => 1,
        None => 0,
    };
    Ok(ret.into())
}

/// HELLO.LEFTPAD str len ch
/// This is an implementation of the infamous LEFTPAD function, that
/// was at the center of an issue with the npm modules system in March 2016.
///
/// LEFTPAD is a good example of using a Redis Modules API called
/// "pool allocator", that was a famous way to allocate memory in yet another
/// open source project, the Apache web server.
#[rcmd("hello.leftpad", "", 0, 0, 0)]
fn hello_leftpad(_ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let pad_len = args[2]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid padding length"))? as usize;
    let the_str: &str = args[1].to_str()?;
    let the_char: &str = args[3].to_str()?;
    if the_str.len() >= pad_len {
        return Ok(the_str.into());
    }
    if the_char.len() != 1 {
        return Err(Error::new("ERR padding must be a single char"));
    }
    let the_char = the_char.chars().nth(0).unwrap();
    let mut pad_str = (0..(pad_len - the_str.len()))
        .map(|_| the_char)
        .collect::<String>();
    pad_str.push_str(the_str);
    Ok(pad_str.into())
}

#[rwrap("call")]
fn init(ctx: &mut Context, args: Vec<RStr>) -> Result<(), Error> {
    ctx.debug(&format!(
        "Module loaded with ARGV[{}] = {:?}\n",
        args.len(),
        args.iter()
            .map(|v| v.to_str().unwrap().to_owned())
            .collect::<Vec<String>>()
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
        hello_simple_cmd,
        hello_push_native_cmd,
        hello_push_call_cmd,
        hello_push_call2_cmd,
        hello_list_sum_len_cmd,
        hello_list_splice_cmd,
        hello_rand_array_cmd,
        hello_repl1_cmd,
        hello_repl2_cmd,
        hello_toggle_case_cmd,
        hello_more_expire_cmd,
        hello_zsumrange_cmd,
        hello_lexrange_cmd,
        hello_hcopy_cmd,
        hello_leftpad_cmd,
    ],
}
