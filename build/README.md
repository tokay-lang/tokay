# build

This folder contains triggers for building parts of Tokay using Tokay itself.

## `make parser` - generate Tokay's parser from `tokay.tok`

Tokay uses a program written in itself (`src/compiler/tokay.tok`) to generate its own language parser (`src/compiler/parser.rs`) incrementally.

- `make parser` updates the content of `src/compiler/parser.rs` from `src/compiler/tokay.tok`.
- `make reset_parser` resets `src/compiler/parser.rs` from git, if something went wrong.

## `make builtins` - generate Tokay's builtin registry from the Rust sources

Generate the file `src/_builtins.rs` using `src/_builtins.tok`.

- `make builtins` updates the content of `src/_builtins.rs` using `src/_builtins.tok` from the Rust sources.
- `make reset_builtins` resets `src/_builtins.rs` from git, if something went wrong.
