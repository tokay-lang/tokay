# ast2rust: tokay.tok => grammar.rs

This is a little tooling to generate the file `src/compiler/grammar.rs` from `examples/tokay.tok` using mostly Tokay itself. Only some help of awk and a Makefile is used to quickly get this thing up.

Afterwards, the new parser can be enabled by the env var `TOKAY_PARSER_USE_TOKAY_TOK`:

- `TOKAY_PARSER_USE_TOKAY_TOK=1 cargo run` runs Tokay REPL with the new parser
- `TOKAY_PARSER_USE_TOKAY_TOK=1 cargo test` runs Tokay test suite with new parser (currently this throws errors)
