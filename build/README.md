# build

This folder contains tooling for building parts of Tokay using Tokay itself.

# `make parser` - ast2rust using `tokay.tok` to generate `parser.rs`

Generate Tokay's own language parser `src/compiler/parser.rs` from `src/compiler/tokay.tok` using Tokay itself.

# `make builtins` - generate builtin registry

Generate the file `src/_builtins.rs` using `src/_builtins.tok`.
