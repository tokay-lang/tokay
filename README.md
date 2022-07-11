<div align="center">
    <img src="https://raw.githubusercontent.com/tokay-lang/tokay-artwork/main/banner-with-darkmode-styling.svg" alt="The Tokay Logo" title="Tokay Programming Language" style="width: 80%"><br>
    <a href="https://github.com/tokay-lang/tokay/actions/workflows/main.yml">
        <img src="https://github.com/tokay-lang/tokay/actions/workflows/main.yml/badge.svg">
    </a>
    <a href="https://crates.io/crates/tokay">
        <img src="https://img.shields.io/crates/v/tokay" alt="Version badge for crates.io" title="crates.io/tokay">
    </a>
    <a href="https://docs.rs/tokay/latest/tokay">
        <img src="https://img.shields.io/docsrs/tokay" alt="Badge of docs.rs" title="docs.rs/tokay">
    </a>
    <a href="https://tokay.dev/">
        <img src="https://img.shields.io/website?down_color=red&down_message=offline&up_color=green&up_message=online&url=https%3A%2F%2Ftokay.dev%2F" alt="Indicator for tokay.dev Website availability" title="tokay.dev">
    </a>
    <a href="https://opensource.org/licenses/MIT">
        <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License badge" title="MIT License">
    </a>
</div>

# Tokay

Tokay is a programming language designed for ad-hoc parsing.

> Tokay is under development and not considered for production use yet; There are plenty of bugs, incomplete or experimental features and planned concepts. Be part of Tokay's ongoing development, and [contribute](#contribute)!

## About

Tokay is a language to quickly implement solutions for text processing problems. This can either be just simple data extractions, but also parsing entire structures or parts of it, and turning information into structured parse trees or abstract syntax trees for further processing.

Therefore, Tokay is both a tool for simple one-liners, but can also be used to implement code-analyzers, refactoring tools, interpreters, compilers or transpilers. Actually [Tokay's own language parser](examples/tokay.tok) is implemented in Tokay itself.

Tokay is inspired by [awk](https://en.wikipedia.org/wiki/AWK), but follows its own philosophy, ideas and design principles. It might be usable as a common scripting language for various problems as well, but mainly focuses on the parsing features, which are a fundamental part built into the language.

Tokay is still a very young project and gains much potential. [Volunteers are welcome!](#contribute)

## Highlights

- Interpreted, procedural and imperative scripting language
- Concise and easy to learn syntax and object system
- Stream-based input processing
- Automatic parse tree construction and synthesis
- Left-recursive parsing structures ("parselets") supported
- Implements a memoizing packrat parsing algorithm internally
- Robust and fast, as it is written entirely in safe [Rust](https://rust-lang.org)
- Enabling awk-style one-liners in combination with other tools
- Generic functions and parselets (*coming soon)
- Import system to create modularized programs (*coming soon)
- Embedded interoperability with other programs (*coming soon)

## Installation

Using Rusts dependency manager and build-tool `cargo`, simply install Tokay with

```bash
$ cargo install tokay
```

Alternatively, there's also a [`tokay`](https://aur.archlinux.org/packages/tokay) and [`tokay-git`](https://aur.archlinux.org/packages/tokay-git) package in the [Arch Linux AUR](https://aur.archlinux.org/).

## Examples

Tokay's version of "Hello World" is quite obvious.

```tokay
print("Hello World")
```

> ```
> $ tokay 'print("Hello World")'
> Hello World
> ```

Tokay can also greet any wor(l)ds that are being fed to it. The next program prints "Hello Venus", "Hello Earth" or "Hello" followed by any other name previously parsed by the builtin `Word`-token. Any other input than a word is automatically omitted.

```tokay
world => Word   print("Hello " + $world)
```

> ```
> $ tokay 'world => Word   print("Hello " + $world)' -- "World 1337 Venus Mars 42 Max"
> Hello World
> Hello Venus
> Hello Mars
> Hello Max
> ```

A simple program for counting words which exists of a least three characters and printing a total can be implemented like this:

```tokay
Word(min=3) ++words accept
end words
```

> ```
> $ tokay "Word(min=3) ++words accept; end words" -- "this is just the 1st stage of 42.5 or .1 others"
> 5
> ```

The next, extended version of the program from above counts all words and prints any numbers.

```tokay
Word ++words accept
Number
end words + " words"
```

> ```
> $ tokay 'Word ++words accept; Number; end words + " words"' -- "this is just the 1st stage of 42.5 or .1 others"
> (1, 42.5, 0.1, "9 words")
> ```

By design, Tokay constructs syntax trees from consumed information automatically.

The next program directly implements a parser and interpreter for simple mathematical expressions, like `1 + 2 + 3` or `7 * (8 + 2) / 5`. The result of each expression is printed afterwards. Processing direct and indirect left-recursions without ending in infinite loops is one of Tokay's core features.

```tokay
_ : [ \t]+                # redefine whitespace to just tab and space

Factor : @{
    Int _                 # built-in 64-bit signed integer token
    '(' _ Expr ')' _
}

Term : @{
    Term '*' _ Factor     $1 * $4
    Term '/' _ Factor     $1 / $4
    Factor
}

Expr : @{
    Expr '+' _ Term       $1 + $4
    Expr '-' _ Term       $1 - $4
    Term
}

Expr _ print("= " + $1)   # gives some neat result output
```

> ```
> $ tokay examples/expr_from_readme.tok
> 1 + 2 + 3
> = 6
> 7 * (8 + 2) / 5
> = 14
> 7*(3-9)
> = -42
> ...
> ```

Tokay can also be used for programs without any parsing features.<br>
Next is a recursive attempt for calculating the faculty of an integer.

```tokay
faculty : @x {
    if !x return 1
    x * faculty(x - 1)
}

faculty(4)
```

> ```
> $ tokay examples/faculty.tok
> 24
> ```

And this version of above program calculates the faculty for any integer token matches from the input. Just the invocation is different, and uses the Number token.

```tokay
faculty : @x {
    if !x return 1
    x * faculty(x - 1)
}

print(faculty(int(Number)))
```

> ```
> $ tokay examples/faculty2.tok -- "5 6 ignored 7 other 14 yeah"
> 120
> 720
> 5040
> 87178291200
> $ tokay examples/faculty2.tok
> 5
> 120
> 6
> 720
> ignored 7
> 5040
> other 14
> 87178291200
> ...
> ```

## Documentation

Same as Tokay itself, the documentation is currently established. The latest version can be obtained on the website [tokay.dev](https://tokay.dev). The documentation source code is maintained in a [separate repository](https://github.com/tokay-lang/tokay-docs).

## Repository

This repository holds all required source files to provide Tokay with examples.

```
.                  # Build scripts, Cargo.toml, etc.
├── examples       # Example programs
├── macros         # Source of the tokay-macros crate required for building
├── src            # Tokay's source code
│   ├── compiler   # Compiler source
│   ├── value      # Object system source
│   └── vm         # Virtual machine source
└── tests          # Some use-case examples required by the test suite
```

## Contribute

Contributions of any kind, might it be code, bug reports, bugfixes, documentation, support or advertising are always welcome!

Take a look into the [bug tracker](https://github.com/tokay-lang/tokay/issues) or watch for `//fixme`- and `//todo`-comments in the source code for open issues and things that need to be improved (there are plenty of them).

If you want to create a pull request, ensure that `cargo run` and `cargo test` run without errors. When new features where added, don't miss to write some unit tests for them. Run `cargo fmt` before you finally commit.

Feel free to [contact me](https://phorward.info) directly on any questions, or [file an issue here](https://github.com/tokay-lang/tokay/issues/new).

## Logo

The Tokay programming language is named after the [Tokay gecko (Gekko gecko)](https://en.wikipedia.org/wiki/Tokay_gecko) from Asia, shouting out "token" in the night.

The Tokay logo and icon was thankfully designed by [Timmytiefkuehl](https://github.com/timmytiefkuehl).<br>
Check out the [tokay-artwork](https://github.com/tokay-lang/tokay-artwork) repository for different versions of the logo as well.

## License

Copyright © 2022 by Jan Max Meyer, Phorward Software Technologies.

Tokay is free software under the MIT license.<br>
Please see the LICENSE file for details.
