use redismodule::{redis_module, assert_len};
use redismodule::{raw, Context, Error, RResult, RStr, RType, Value, IO, ArgvFlags, Digest};
use std::os::raw::{c_void, c_int};


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
    pub fn len(&self)  -> usize {
        self.len
    }
    pub fn iter(&self) -> HelloTypeNodeIter<'_> {
        HelloTypeNodeIter { next: self.head.as_ref().map(|node| &**node) }
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


fn hellotype_insert(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let mut key = ctx.open_write_key(&args[1]);
    let exist = key.verify_module_type(&HELLO_TYPE)?;
    let value = args[2].get_long_long().map_err(|e| Error::generic("invalid value: must be a signed 64 bit integer"))?;
    if exist {
        let hto = HelloTypeNode::new();
        key.set_value(&HELLO_TYPE, hto)?;
    }
    let hto: &mut HelloTypeNode = key.get_value(&HELLO_TYPE)?.unwrap();
    hto.push(value);
    ctx.signal_key_as_ready(&args[1]);
    ctx.replicate_verbatim();
    Ok(hto.len().into())
}

fn hellotype_range(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 3);
    let key = ctx.open_write_key(&args[1]);
    key.verify_module_type(&HELLO_TYPE)?;
    let first = args[2].get_positive_integer().map_err(|_| Error::generic("invalid first parameters"))? as usize;
    let count = args[3].get_positive_integer().map_err(|_| Error::generic("invalid count parameters"))? as usize;
    let hto = key.get_value::<HelloTypeNode>(&HELLO_TYPE)?;
    if hto.is_none() {
        return Ok(Value::Array(vec![]));
    }
    let eles: Vec<Value> = hto.unwrap().iter().skip(first).take(count).cloned().map(|v| v.into()).collect();
    Ok(Value::Array(eles))
}

fn hellotype_len(ctx: &Context, args: Vec<RStr>) -> RResult {
    assert_len!(args, 2);
    let key = ctx.open_write_key(&args[1]);
    key.verify_module_type(&HELLO_TYPE)?;
    let hto = key.get_value::<HelloTypeNode>(&HELLO_TYPE)?;
    if hto.is_none() {
        return Ok(0i64.into());
    }
    let len = hto.unwrap().len() as i64;
    Ok(len.into())
}

fn hellotype_brange(ctx: &Context, mut args: Vec<RStr>) -> RResult {
    assert_len!(args, 5);
    let key = ctx.open_write_key(&args[1]);
    let exists = key.verify_module_type(&HELLO_TYPE)?;
    let timeout = args[4].get_positive_integer().map_err(|_| Error::generic("invalid timeout parameter"))?;
    if exists {
        args.remove(args.len() - 1);
        return hellotype_range(ctx, args);
    }
    // TODO
    Ok("OK".into())
}

extern "C" fn hellotype_brange_reply(
    ctx: *mut raw::RedisModuleCtx , 
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int
) -> c_int {
    unimplemented!{}
}

redis_module! {
    name: "hellotype",
    version: 1,
    data_types: [],
    commands: [
        ["hellotype.insert", hellotype_insert, "write deny-oom", 1, 1, 1],
        ["hellotype.range", hellotype_range, "readonly", 1, 1, 1],
        ["hellotype.len", hellotype_len, "readonly", 1, 1, 1],
        ["hellotype.brange", hellotype_brange, "readonly", 1, 1, 1],
    ],
}

static HELLO_TYPE: RType = RType::new(
    "hellotype",
    0,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(hello_type_rdb_load),
        rdb_save: Some(hello_type_rdb_save),
        aof_rewrite: Some(hello_type_aof_rewrite),
        mem_usage: Some(hello_type_mem_usage),
        free: Some(hello_type_free),
        digest: Some(hello_type_digest),

        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
    },
);

extern "C" fn hello_type_rdb_load(rdb: *mut raw::RedisModuleIO, encver: c_int) -> *mut c_void {
    let mut io = IO::from_ptr(rdb);
    if encver != 0 {
        return  0 as *mut c_void;
    }
    let elements = io.load_unsigned();
    let mut hto = HelloTypeNode::new();
    for _ in 0..elements {
        let ele = io.load_signed();
        hto.push(ele);
    }
    Box::into_raw(Box::new(hto)) as *mut c_void
}

unsafe extern "C" fn hello_type_rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let mut io = IO::from_ptr(rdb);
    let hto = &*(value as *mut HelloTypeNode);
    let eles: Vec<&i64> = hto.iter().collect();
    io.save_unsigned(eles.len() as u64);
    eles.iter().for_each(|v| io.save_signed(**v));
}

unsafe extern "C" fn hello_type_aof_rewrite(aof: *mut raw::RedisModuleIO, key: *mut raw::RedisModuleString, value: *mut c_void) {
    let mut io = IO::from_ptr(aof);
    let hto = &*(value as *mut HelloTypeNode);
    let eles: Vec<&i64> = hto.iter().collect();
    let key = RStr::from_ptr(key);
    let keyname = key.to_str().unwrap();
    eles.iter().for_each(|v| io.emit_aof("HELLOTYPE.INSERT", ArgvFlags::new(),  &[keyname, &v.to_string()]))
}

unsafe extern "C" fn hello_type_mem_usage(value: *const c_void) -> usize {
    let hto = &*(value as *const HelloTypeNode);
    std::mem::size_of::<HelloTypeNode>() * hto.len()
}

unsafe extern "C" fn hello_type_digest(md: *mut raw::RedisModuleDigest, value: *mut c_void) {
    let mut md = Digest::from_ptr(md);
    let hto = &*(value as *const HelloTypeNode);
    let eles: Vec<&i64> = hto.iter().collect();
    eles.iter().for_each(|v| md.add_long_long(**v));
    md.end_sequeue();
}

unsafe extern "C" fn hello_type_free(value: *mut c_void) {
    Box::from_raw(value as *mut HelloTypeNode);
}
