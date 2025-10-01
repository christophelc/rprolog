# rprolog

A small Prolog parser written in Rust.

## Overview

**rprolog** is a minimal Prolog parser implemented in Rust. It uses the [`pest`](https://pest.rs/) parser generator to define the Prolog grammar and parse Prolog facts, rules, queries, and terms.

## Features

- Parses Prolog facts, rules, and queries
- Supports atoms, variables, numbers, lists, and functors
- Written in safe, idiomatic Rust
- Easily extensible for more Prolog features

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021 or later)

### Build

```sh
cargo build
```

### Run Tests

```sh
cargo test
```

## Project Structure

- `src/grammar.pest` — Prolog grammar definition (pest format)
- `src/ast.rs` — Abstract Syntax Tree (AST) definitions for Prolog terms, expressions, and predicates
- `src/parser.rs` — Parser setup and unit tests

## Example

You can parse a Prolog fact like this:

```rust
let input = "parent(john, X).";
let result = rprolog::parser::G::parse(rprolog::parser::Rule::fact, input);
println!("{:?}", result);
```

## Extending

- Add more grammar rules to `grammar.pest` for additional Prolog features.
- Implement evaluation or unification logic in Rust for a full interpreter.

## License

MIT

