all:
	cargo run --release

debug:
	cargo run

reset:
	probe-rs reset --chip RP2040 --protocol swd
	probe-rs attach --chip RP2040 --protocol swd target/thumbv6m-none-eabi/debug/m8-kbd

prepare:
	rustup target install thumbv6m-none-eabi
	cargo install flip-link
	cargo install probe-rs --features=cli --locked

clean:
	rm -rf target Cargo.lock
