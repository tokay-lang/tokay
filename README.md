# The Tokay Programming Language

Tokay is a programming language for parsing and text-processing - under development.


## About

Welcome to the Tokay Programming Language!

Tokay's design goal is to provide a smart, user-friendly and expressive language and runtime environment to easily analyze, process and compile structured information.

It runs on an input stream and matches patterns against this stream. When a pattern successfully matches, further patterns or programmatic actions can be expressed. By default, Tokay automatically constructs an abstract syntax tree from the input successfully recognized.

Nevertheless, Tokay can also be used as a straightforward, procedural programming language to quickly operate on any information, without using the parsing features at all.


## Features

- Dynamically typed programming language with a self-hosted syntax and own semantics, embedded inside a grammar definition
- Internally runs a memoizing, backtracking recursive-descend parser ("Packrat-parser") with support for direct and indirect left-recursive grammars
- Programs _can_ represent grammars, but they don't have to.
- Universal _complex_ data type for representing lists and maps used for further value structuring
- Built-in support and building-blocks (generative parselets) for standard tokens like numbers, strings, comments, etc.
- Modular structuring, programs can be included to intermix several parsers
- Inspired by awk, Python and Rust, implemented in Rust!


We're looking for volunteers to extend this list!

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

This is also a Tokay program, wihtout using the parsing facilities. It  recursively calculates faculties.

```
Faculty = @x {
    if !x return 1
    x * Faculty(x - 1)
}

Faculty(4)
```

More to come. Enjoy!

## Contributions

Contributions of any kind are very welcome. Feel free to contact me.

Tokay is also my very first real-world project with Rust, therefore I'm sure some things inside the code could easily be improved by more experienced Rustaceans out there.


## License

Tokay is licensed under the MIT license.
Please see the LICENSE file for more details.
