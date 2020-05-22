use iredismodule::io::{Digest, IO};
use iredismodule::prelude::*;
use iredismodule::rtype::TypeMethod;
use iredismodule_macros::{rcmd, rtypedef, rwrap};
use std::time::Duration;

// ========================== Internal data structure  =======================
// This is just a linked list of 64 bit integers where elements are inserted
// in-place, so it's ordered. There is no pop/push operation but just insert
// because it is enough to show the implementation of new data types without
// making things complex.

pub struct HelloTypeNode {
    head: Link,
    len: usize,
}

type Link = Option<Box<Node>>;

struct Node {
    elem: i64,
    next: Link,
}

impl HelloTypeNode {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }
    pub fn push(&mut self, elem: i64) {
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
        self.len += 1;
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn iter(&self) -> HelloTypeNodeIter<'_> {
        HelloTypeNodeIter {
            next: self.head.as_ref().map(|node| &**node),
        }
    }
}
pub struct HelloTypeNodeIter<'a> {
    next: Option<&'a Node>,
}

impl<'a> Iterator for HelloTypeNodeIter<'a> {
    type Item = &'a i64;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            &node.elem
        })
    }
}

/// ========================== "hellotype" type methods =======================
#[rtypedef("hellotype", 0)]
impl TypeMethod for HelloTypeNode {
    fn rdb_load(io: &mut IO, encver: u32) -> Option<Box<Self>> {
        if encver != 0 {
            return None;
        }
        let elements = io.load_unsigned();
        let mut hto = Self::new();
        for _ in 0..elements {
            let ele = io.load_signed();
            hto.push(ele);
        }
        Some(Box::new(hto))
    }
    fn rdb_save(&self, io: &mut IO) {
        let eles: Vec<&i64> = self.iter().collect();
        io.save_unsigned(eles.len() as u64);
        eles.iter().for_each(|v| io.save_signed(**v));
    }
    fn free(_: Box<Self>) {}
    fn aof_rewrite<T: AsRef<str>>(&self, io: &mut IO, key: T) {
        let eles: Vec<&i64> = self.iter().collect();
        let keyname = key.as_ref();
        eles.iter()
            .for_each(|v| io.emit_aof("HELLOTYPE.INSERT", &[keyname, &v.to_string()]))
    }
    /// The goal of this function is to return the amount of memory used by
    /// the HelloType value.
    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() * self.len()
    }
    fn digest(&self, digest: &mut Digest) {
        let eles: Vec<&i64> = self.iter().collect();
        eles.iter().for_each(|v| digest.add_integer(**v));
        digest.end_sequeue();
    }
}

// ========================= "hellotype" type commands =======================

/// HELLOTYPE.INSERT key value
#[rcmd("hellotype.insert", "write deny-oom", 1, 1, 1)]
fn hellotype_insert(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    if args.len() != 3 {
        return Err(Error::WrongArity);
    }
    let key = ctx.open_write_key(&args[1]);
    let exist = key.check_module_type(&HELLOTYPE)?;
    let value = args[2]
        .get_integer()
        .map_err(|_e| Error::new("ERR invalid value: must be a signed 64 bit integer"))?;

    let hto: &mut HelloTypeNode = if exist {
        key.get_value(&HELLOTYPE)?.unwrap()
    } else {
        let hto = HelloTypeNode::new();
        key.set_value(&HELLOTYPE, hto)?
    };
    hto.push(value);
    ctx.signal_key_as_ready(&args[1]);
    ctx.replicate_verbatim();
    Ok(hto.len().into())
}

/// HELLOTYPE.RANGE key skip limit
#[rcmd("hellotype.range", "readonly", 1, 1, 1)]
fn hellotype_range(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    if args.len() != 4 {
        return Err(Error::WrongArity);
    }
    let key = ctx.open_write_key(&args[1]);
    key.check_module_type(&HELLOTYPE)?;
    let first = args[2]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid first parameters"))? as usize;
    let count = args[3]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid count parameters"))? as usize;
    let hto = key.get_value::<HelloTypeNode>(&HELLOTYPE)?;
    if hto.is_none() {
        return Ok(Value::Array(vec![]));
    }
    let eles: Vec<Value> = hto
        .unwrap()
        .iter()
        .skip(first)
        .take(count)
        .cloned()
        .map(|v| v.into())
        .collect();
    Ok(Value::Array(eles))
}

/// HELLOTYPE.LEN key
#[rcmd("hellotype.len", "readonly", 1, 1, 1)]
fn hellotype_len(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    if args.len() != 2 {
        return Err(Error::WrongArity);
    }
    let key = ctx.open_write_key(&args[1]);
    key.check_module_type(&HELLOTYPE)?;
    let hto = key.get_value::<HelloTypeNode>(&HELLOTYPE)?;
    if hto.is_none() {
        return Ok(0i64.into());
    }
    let len = hto.unwrap().len() as i64;
    Ok(len.into())
}

// ====================== Example of a blocking command ====================

/// HELLOTYPE.BRANGE key offset limit timeout -- This is a blocking verison of
/// the RANGE operation, in order to show how to use the API
/// Context:block_client_on_keys.
#[rcmd("hellotype.brange", "readonly", 1, 1, 1)]
fn hellotype_brange(ctx: &mut Context, mut args: Vec<RStr>) -> RResult {
    if args.len() != 5 {
        return Err(Error::WrongArity);
    }
    let key = ctx.open_write_key(&args[1]);
    let exists = key.check_module_type(&HELLOTYPE)?;
    let timeout = args[4]
        .assert_integer(|v| v > 0)
        .map_err(|_| Error::new("ERR invalid timeout parameter"))?;
    if exists {
        args.remove(args.len() - 1);
        return hellotype_range(ctx, args);
    }
    let args_bc = vec![&args[1]];
    let privdata = "some data".to_owned();
    ctx.block_client_on_keys(
        Some(helloblock_reply_c),
        Some(helloblock_timeout_c),
        Some(helloblock_free_c),
        Duration::from_secs(timeout),
        &args_bc,
        privdata,
    );
    Ok(Value::NoReply)
}

/// Reply callback for blocking command HELLOTYPE.BRANGE, this will get
/// called when the key we blocked for is ready: we need to check if we
/// can really serve the client, and reply OK or ERR accordingly.
#[rwrap("call")]
fn helloblock_reply(ctx: &mut Context, mut args: Vec<RStr>) -> RResult {
    let keyname = ctx.get_blocked_client_ready_key().unwrap();
    let key = ctx.open_read_key(&keyname);
    key.check_module_type(&HELLOTYPE)?;
    args.remove(args.len() - 1);
    return hellotype_range(ctx, args);
}

/// Timeout callback for blocking command HELLOTYPE.BRANGE
#[rwrap("call")]
fn helloblock_timeout(_ctx: &mut Context, _: Vec<RStr>) -> RResult {
    Ok(Value::String("Request timeout".into()))
}

/// Private data freeing callback for HELLOTYPE.BRANGE command.
#[rwrap("free")]
fn helloblock_free(ctx: &mut Context, data: Box<String>) {
    ctx.debug(&format!("free: {}", data.as_str()));
}

define_module! {
    name: "hellotype",
    version: 1,
    data_types: [
        HELLOTYPE,
    ],
    init_funcs: [],
    commands: [
        hellotype_insert_cmd,
        hellotype_range_cmd,
        hellotype_len_cmd,
        hellotype_brange_cmd,
    ],
}
