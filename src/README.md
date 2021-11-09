# Tokay /src structure

```
.
├── builtin.rs       # Callable built-in functions
├── ccl.rs           # Character-classes with set-functions
├── compiler         # Tokay compiler
│   ├── ast.rs       # Traversal of the parsed AST
│   ├── compiler.rs  # Compiler interface
│   ├── iml          # Intermediate language units
│   ├── macros.rs    # Macros required for parser bootstrap
│   ├── parser.rs    # Tokay's own grammar implemented using macros
│   └── usage.rs     # Resolving symbols
├── error.rs         # Error object
├── reader.rs        # Unified Reader to read from files, strings, streams...
├── repl.rs          # Tokay REPL
├── test.rs          # Test cases
├── token.rs         # Callable tokens
├── utils.rs         # Utilities and miscelleanous
├── value            # Values and objects
└── vm               # Tokay virtual machine
```
