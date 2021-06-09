# Tokay

![Tokay Logo](assets/tokay.svg)

An imperative, procedural programming language dedicated to parsing and other text-processing tasks.

## About

Tokay is a programming language designed for ad-hoc parsing. It is heavily inspired by [awk](https://en.wikipedia.org/wiki/AWK), but follows its own philosophy and design principles. It might also be useful as a general purpose scripting language, but mainly focuses on processing textual input and work on trees with information extracted from this input.

## Highlights

- Stream-based input processing
- Automatic parse tree synthesis
- Left-recursive parsing structures ("parselets") supported
- Internally implements a memoizing packrat parsing algorithm
- Memory safe due its implementation in [Rust](https://rust-lang.org) without any unsafe calls
- Dynamic lists and dicts
- Enabling awk-style one-liners in combination with other scripts and programs

There are plenty of further features planned, see [todo.md](todo.md) for details.

## Examples

This is how Tokay greets the world
```tokay
print("Hello World")
```
but Tokay can also greet any name coming in, that's
```tokay
Name print("Hello " + $1)
```

With its build-in abstract-syntax tree synthesis, Tokay is designed as a language for directly implementing ad-hoc parsers. The next program directly implements a left-recursive grammar for parsing and evaluating simple mathematical expressions, like `1+2+3` or `7*(8+2)/5`.

```tokay
# Factor is a parselet matching an Integer or grouped sub-expression.

Factor : @{
    Integer             # Integer is a builtin matching 64-bit
                        # signed integer values from a stream
    '(' Expr ')'
}

# The next two parselets define sequences and their evalutation.
# Calculated values in sequences take highest precedence, therefore
# the calculated values become the result of each sequence.

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

Tokay can also be used for writing programs without any parsing features.
Next one is a recursive attempt for calculating the faculty of a value.

```
faculty : @x {
    if !x return 1
    x * faculty(x - 1)
}

faculty(4)
```

## Logo

The Tokay logo and icon was designed by [Timmytiefkuehl](https://github.com/timmytiefkuehl).


## Contributions

Contributions of any kind are very welcome. Feel free to contact me.

Tokay is also my first project written in Rust, therefore I'm sure some things inside the code could easily be improved by more experienced Rustaceans out there.


## License

Tokay is free software under the MIT license.
Please see the LICENSE file for more details.
