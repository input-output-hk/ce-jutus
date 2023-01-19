build: system-info
  cargo build

mk: check build

release: system-info
  cargo build --release

fmt: system-info
  cargo fmt --all -- --check

check: fmt
  cargo clippy -- -D warnings

fix: system-info
  cargo clippy --fix

# Run examples/parse
parse: system-info
  cargo run --example parse

# Run examples/parse_function
parse_function: system-info
  cargo run --example parse_function

# Run examples/parse_function
parse_ts_function: system-info
  cargo run --example parse_ts_function

examples: parse parse_function parse_ts_function

test: check examples

system-info:
  @echo "Running on {{arch()}}/{{os_family()}}/{{os()}}"
