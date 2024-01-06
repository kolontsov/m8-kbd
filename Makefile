all:
	cargo run

release:
	cargo run --release

prepare:
	rustup target install thumbv6m-none-eabi
	cargo install flip-link
	cargo install probe-rs --features=cli --locked

clean:
	rm -rf target Cargo.lock
