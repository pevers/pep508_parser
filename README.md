# pep508_parser [![test suite](https://github.com/pevers/pep508_parser/actions/workflows/ci.yaml/badge.svg)](https://github.com/pevers/pep508_parser/actions/workflows/ci.yaml)

This crate parses [PEP-508](https://peps.python.org/pep-0508/) strings for Rust.
It uses [pest](https://github.com/pest-parser/pest) under the hood to parse a simplified version of the Parsing Expression Grammar (PEG) of the PEP-508 standard.

```toml
[dependencies]
pep508_parser = "0.1.0"
```

```rust
use pep508_parser::parse;

fn main() {
    let dependency =
        parse("name[quux, strange];python_version<'2.7' and platform_version=='2'").unwrap();
}
```