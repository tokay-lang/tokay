# Changelog

## [main]

Current main branch.

- Self-hosted tokay parser in `examples/tokay.tok`


## [v0.4]

Released on Nov 15, 2021

- Implementation of a separated virtual machine (VM) to reduce overall stack usage (#8)
- Removal of the recursive interpreter due to VM replacement
- Cleaned-up and modularization of an encapsulated compiler
- Turned compile-time building blocks into intermediate language (iml)
- Built-in character classes
- Started loop implementation
- Improved debug facilities


## [v0.3]

Released on Jul 7, 2021

- Cleaning and finalizing syntax
- Collections with list and dict expression
- Consumables and built-in tokens
- Implemented all remaining operators
- Inline increment/decrement
- R-value handling for subscripts and attributes, with calls to built-ins
- Compile-time expression evaluation
- Whitespace handling
- Escape sequences
- Command-line parameters


## [v0.2]

Released on Mar 21, 2021

- Parselets and built-ins with parameters
- Improved error reporting
- Interactive REPL


## [v0.1]

Released on Mar 4, 2021

- Character-classes
- Universal Reader trait
- Macros for grammar expression
- First draft of the Tokay grammar
- Implementing recursive interpreter
- Left-recursive parselets
- Sequence capturing and AST construction
- Distinction between constants and variables
- Symbol resolving and scopes


[main]: https://github.com/phorward/tokay/compare/v0.4...main
[v0.4]: https://github.com/phorward/tokay/compare/v0.3...v0.4
[v0.3]: https://github.com/phorward/tokay/compare/v0.2...v0.3
[v0.2]: https://github.com/phorward/tokay/compare/v0.1...v0.2
[v0.1]: https://github.com/phorward/tokay/compare/2d74215f4842d295371112a630d15ab03442cd1e...v0.1
