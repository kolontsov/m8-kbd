RELEASE = target/thumbv6m-none-eabi/release/m8-kbd

all:
	cargo run --release

uf2:
	cargo build --release
	@export OUTPUT=$(RELEASE)-v$(shell cargo metadata --no-deps --format-version 1 | jq --raw-output '.packages[0].version').uf2 && \
	elf2uf2-rs $(RELEASE) $$OUTPUT && \
	file $$OUTPUT

debug:
	cargo run

reset:
	probe-rs reset --chip RP2040 --protocol swd
	probe-rs attach --chip RP2040 --protocol swd $(RELEASE)

prepare:
	rustup target install thumbv6m-none-eabi
	cargo install flip-link
	cargo install probe-rs --features=cli --locked
	cargo install elf2uf2-rs --locked

clean:
	rm -rf target Cargo.lock
