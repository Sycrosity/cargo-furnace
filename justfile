build: fmt
  cargo build

release: fmt
  cargo build --release

#correctness/testing

fmt:
  cargo fmt

clippy:
  cargo clippy --all-targets -- -D warnings

test:
  cargo test

actions: fmt clippy test


all: fmt build actions
