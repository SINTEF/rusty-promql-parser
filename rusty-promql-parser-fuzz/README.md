# rusty-promql-parser-fuzz

AFL fuzzer for rusty-promql-parser.

## Setup

```sh
cargo install cargo-afl
```

## Usage

```sh
make build      # Build the fuzzer
make create-in  # Generate input corpus
make fuzz       # Run fuzzer (builds and creates inputs if needed)
make clean      # Remove in/, out/, and target/ directories
```
