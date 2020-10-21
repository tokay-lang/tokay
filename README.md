# The Tokay Programming Language

Tokay is a programming language for parsing and text-processing - under development.


## About

Tokay's design goal is to provide a smart, user-friendly and expressive language and execution environment to easily process structured information.

It runs on an input stream and matches patterns against this stream. When a pattern successfully matches, further patterns or programmatic actions may follow. By default, Tokay automatically constructs an abstract syntax tree from the input successfully recognized.


## Features

- Implements a memoizing, backtracking recursive-decent parser ("Packrat-parser") with support for direct and indirect left-recursion
- Built-in support and building-blocks for standard tokens like numbers, strings, comments, etc.
- Modular structuring, programs can be included to intermix several parsers
- Inspired by awk, Python and Rust


## Example

This program recognizes and interprets expressions:

```tokay
Factor = @{
    Float
    '(' Expr ')'
}

Term = @{
    Term:t '*' Factor:f   $t * $f
    Term:t '/' Factor:f   $t / $f
    Factor
}

Expr = @{
    Expr:e '+' Term:t     $e + $t
    Expr:e '-' Term:t     $e - $t
    Term
}

Expr                      print
```

## Contributons

Any contributions are very welcome!

Tokay is also my very first real-world project I've implemented with Rust, therefore I'm sure some things could easily be done better by more experienced people out there.


## License

Tokay is licensed under the MIT license. Please see the LICENSE file for more details.
