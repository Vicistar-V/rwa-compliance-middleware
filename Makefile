CARGO = cargo

.PHONY: all build test lint fmt doc clean

all: build lint test doc

build:
	$(CARGO) build

test:
	$(CARGO) test

lint:
	$(CARGO) clippy --all-targets -- -D warnings

fmt:
	$(CARGO) fmt

doc:
	$(CARGO) doc --no-deps

clean:
	$(CARGO) clean
