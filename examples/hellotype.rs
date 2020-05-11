use redismodule::{assert_len, define_module};
use redismodule::{ArgvFlags, Context, Digest, Error, RResult, RStr, TypeMethod, Value, IO};
use redismodule_macros::{rcall, rcmd, rfree, rtypedef};
use std::time::Duration;

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

#[rtypedef(name = "hellotype", version = 0)]
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
    fn aof_rewrite(&self, io: &mut IO, key: &RStr) {
        let eles: Vec<&i64> = self.iter().collect();
        let keyname = key.to_str().unwrap();
        eles.iter().for_each(|v| {
            io.emit_aof(
                "HELLOTYPE.INSERT",
                ArgvFlags::new(),
                &[keyname, &v.to_string()],
            )
        })
    }
    fn mem_usage(&self) -> usize {
        std::mem::size_of::<Self>() * self.len()
    }
    fn digest(&self, digest: &mut Digest) {
        let eles: Vec<&i64> = self.iter().collect();
        eles.iter().for_each(|v| digest.add_long_long(**v));
        digest.end_sequeue();
    }
}

#[rcmd(
    name = "hellotype.insert",
    flags = "write deny-oom",
    first_key = 1,
    last_key = 1,
    key_step = 1
)]
fn hellotype_insert(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let mut key = ctx.open_write_key(&args[1]);
    let exist = key.verify_module_type(&HELLOTYPE)?;
    let value = args[2]
        .get_long_long()
        .map_err(|_e| Error::generic("invalid value: must be a signed 64 bit integer"))?;

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

#[rcmd(
    name = "hellotype.range",
    flags = "readonly",
    first_key = 1,
    last_key = 1,
    key_step = 1
)]
fn hellotype_range(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 4);
    let key = ctx.open_write_key(&args[1]);
    key.verify_module_type(&HELLOTYPE)?;
    let first = args[2]
        .get_positive_integer()
        .map_err(|_| Error::generic("invalid first parameters"))? as usize;
    let count = args[3]
        .get_positive_integer()
        .map_err(|_| Error::generic("invalid count parameters"))? as usize;
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

#[rcmd(
    name = "hellotype.len",
    flags = "readonly",
    first_key = 1,
    last_key = 1,
    key_step = 1
)]
fn hellotype_len(ctx: &mut Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let key = ctx.open_write_key(&args[1]);
    key.verify_module_type(&HELLOTYPE)?;
    let hto = key.get_value::<HelloTypeNode>(&HELLOTYPE)?;
    if hto.is_none() {
        return Ok(0i64.into());
    }
    let len = hto.unwrap().len() as i64;
    Ok(len.into())
}

#[rcmd(
    name = "hellotype.brange",
    flags = "readonly",
    first_key = 1,
    last_key = 1,
    key_step = 1
)]
fn hellotype_brange(ctx: &mut Context, mut args: Vec<RStr>) -> RResult {
    assert_len!(args, 5);
    let key = ctx.open_write_key(&args[1]);
    let exists = key.verify_module_type(&HELLOTYPE)?;
    let timeout = args[4]
        .get_positive_integer()
        .map_err(|_| Error::generic("invalid timeout parameter"))?;
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

#[rcall]
fn helloblock_reply(ctx: &mut Context, mut args: Vec<RStr>) -> RResult {
    let keyname = ctx.get_blocked_client_ready_key()?;
    let key = ctx.open_read_key(&keyname);
    key.verify_module_type(&HELLOTYPE)?;
    args.remove(args.len() - 1);
    return hellotype_range(ctx, args);
}

#[rcall]
fn helloblock_timeout(ctx: &mut Context, _: Vec<RStr>) -> RResult {
    ctx.reply(Ok(Value::SimpleString("Request timeout".into())));
    Ok(().into())
}

#[rfree]
fn helloblock_free(ctx: &mut Context, data: Box<String>) {
    ctx.log_debug(&format!("free: {}", data.as_str()));
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
