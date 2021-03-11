# Tokay

Tokay is an imperative programming language dedicated to parsing and other text-processing tasks.


## Examples

How Tokay greets the world

```tokay
"Hello World"
```

Tokay is designed as a programming language for parsers with build-in abstract-syntax tree synthesis. This example directly implements a left-recursive grammar for simple mathematical expressions.

```tokay
Factor : @{
    Integer
    '(' Expr ')'
}

Term : @{
    Term '*' Factor     $1 * $3
    Term '/' Factor     $1 / $3
    Factor
}

Expr : @{
    Expr '+' Term       $1 + $3
    Expr '-' Term       $1 - $3
    Term
}


Expr
```

This program implements a functional, recursive attempt to calculate faculties. It doesn't use any parsing features.

```
faculty : @x {
    if !x return 1
    x * faculty(x - 1)
}

faculty(4)
```

Ok, that's it for now. This document is currently under development, same as Tokay itself. Stay tuned!

## Features

- Dynamically typed programming language with a self-hosted syntax and own semantics, embedded inside a grammar definition
- Internally runs a memoizing, backtracking recursive-descend parser ("Packrat-parser") with support for direct and indirect left-recursive grammars
- Programs _can_ represent grammars, but they don't have to.
- Universal _complex_ data type for representing lists and maps used for further value structuring
- Built-in support and building-blocks (generative parselets) for standard tokens like numbers, strings, comments, etc.
- Modular structuring, programs can be included to intermix several parsers
- Inspired by awk, Python and Rust, implemented in Rust!


We're looking for volunteers to extend this list!


## Contributions

Contributions of any kind are very welcome. Feel free to contact me.

Tokay is also my very first real-world project with Rust, therefore I'm sure some things inside the code could easily be improved by more experienced Rustaceans out there.


## License

Tokay is free software under the MIT license.
Please see the LICENSE file for more details.
