.PHONY: all build test clean decode

all: build

build:
	cargo build

test: build
	cargo test

decode: build
	./target/debug/sim_8086 decode $(file)

clean:
	cargo clean
	rm -f tests/*.bin tests/*_dec.s tests/*_ref.bin tests/*_dec.bin
