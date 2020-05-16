examples: examples/*.rs
	cargo build --example helloacl
	cargo build --example helloblock
	cargo build --example hellocluster
	cargo build --example hellohook
	cargo build --example hellotimer
	cargo build --example hellotype
	cargo build --example helloworld
	cargo build --example simple
	cargo build --example testmodule
publish:
	cargo fix && cargo fmt
	cargo publish
publish-macros:
	cargo fix && cargo fmt
	cd macros && cargo publish