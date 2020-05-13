# iredismodule

This crate provides an idiomatic Rust API for the [Redis Modules API](https://redis.io/topics/modules-intro).
It allows writing Redis modules in Rust, without needing to use raw pointers or unsafe code.

# Running the example module

1. [Install Rust](https://www.rust-lang.org/tools/install) 
2. [Install Redis](https://redis.io/download), most likely using your favorite package manager (Homebrew on Mac, APT or YUM on Linux)
3. Run `cargo build --example helloworld`
4. Start a redis server with the `helloworld` module 
   * Linux: `redis-server --loadmodule ./target/debug/examples/helloworld.so`
   * Mac: `redis-server --loadmodule ./target/debug/examples/helloworld.dylib`	
5. Open a Redis CLI, and run `HELLO.SIMPLE`. 

# Writing your own module

See the [examples](examples) directory for some sample modules.

This crate tries to provide high-level wrappers around the standard Redis Modules API, while preserving the API's basic concepts.
Therefore, following the [Redis Modules API](https://redis.io/topics/modules-intro) documentation will be mostly relevant here as well.
