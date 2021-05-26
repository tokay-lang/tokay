# Tokay

Tokay is an imperative, procedural programming language dedicated to parsing and other text-processing tasks.


## Examples

This is how Tokay greets the world

```tokay
print("Hello World")
```

Tokay can also first match the world, and then greet to it... or any other planet?

```tokay
'World' print("Hello Earth")
planet => Name print("Hello " + $planet)
```

Tokay is designed as a programming language for writing ad-hoc parsers with build-in abstract-syntax tree synthesis. The next example directly implements a left-recursive parser for simple mathematical expressions, like `1+2+3` or `7*(8+2)/5`.

```tokay
# Factor is a consumable parselet either matching an Integer or another
# expression enclosed by brackets.
Factor : @{
    Integer             # Integer is a builtin matching 64-bit
                        # signed integer values from a stream
    '(' Expr ')'
}

# The next two parselets define sequences and their interpretation.
# Calculated values in sequences take highest precedence, therefore
# the calculate values become the result of each sequence.

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

# This is the main level, where an expression can occur.
# Any other input - by default - is ignored and skipped.
Expr
```

The next program implements a functional, recursive attempt to calculate the faculty of a vlaue. It doesn't use any parsing features, and represents a standard function in other programming languages.

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

Tokay is also my very first project with Rust, therefore I'm sure some things inside the code could easily be improved by more experienced Rustaceans out there.


## License

Tokay is free software under the MIT license.
Please see the LICENSE file for more details.
