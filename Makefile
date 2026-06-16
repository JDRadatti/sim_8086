.PHONY: all build test clean decode

all: build

build:
	cargo build

test: build
	cargo test

decode: build $(file:.s=.bin)
	./target/debug/sim_8086 decode $(file:.s=.bin)

tests/%.bin: tests/%.s
	nasm -f bin $< -o $@

clean:
	cargo clean
	find tests -type f \( -name '*.bin' -o -name '*_dec.s' -o -name '*_ref.*' \) -delete
