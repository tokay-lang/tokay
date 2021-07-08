# Brainstorming & Todo

This is a recently updated brainstorming and todo scribble file.

## Pritority

- [ ] Generic parselets
  - [ ] `Until<P, Escape=Void>`, e.g. `print(Until<EOF>)`
  - [ ] `Not<P>`
  - [ ] `Peek<P>`
  - [ ] `Repeat<P, min:1, max:0>`
- [ ] Implementation of a module system
  - [ ] Import constants from other module
  - [ ] Run other module as separate programs and work on resulting values
  - [ ] Pass string to separate tokay process (`tokay`-keyword, `@>` operator?)
- [ ] Implement loops
  - `for` construct
  - `loop` construct
  - Implement `break` and `continue`
- [ ] Implement the escape keyword

## General

- [x] Row & column number recording in Reader/Range by working with an Offset data structure rather than usize to allow further tracking, e.g. of row + col numbers
- [ ] Grammar view
  - [ ] Perform left-recursion detection on begin and end also?
  - [ ] Resolve indirect left-recursion as done in pegen? (see src/test.rs comments)
- [ ] Values
  - [ ] Dict should use **indexmap** crate (https://github.com/bluss/indexmap)
  - [ ] Locked objects (this is required to disallow modification of Dict and List constants)
  - [ ] Object method interface, e.g. `(1 2 3).len()`, `(1 2 3).pop()`, `(a => 32).insert("b", 64)`
  - [x] Integer division `1/6` returns 0, but should return float. `1./6` correctly returns 0.16666666666666666
  - [ ] Use string arithmetics for something like 123 ^ 3000 later on, which cannot be handled by i64.
    - [ ] Use external crate **num_bigint** for integers instead of i64
  - [ ] `Op::Repeat(min=1, max=0)` needs some more clarity what is being accepted, especially when no input is consumed

## Syntax

Syntax is under careful consideration.

- [ ] Missing expressional constructs (should be discussed if these are really necessary)
  - [ ] binary `&` (`&=`) and `|` `|=`?
  - [ ] `^` and `^=` (xor)?
  - [ ] `**` and `**=` (powers)?
  - [ ] `//` and `//=` (integer division)?
- [ ] Definition of Regex tokens `/Hel+o Wo?rld/` (not now, see https://github.com/phorward/tokay/issues/1)
- [ ] Implement `ls -ltra` backticks (`) for shell command values
  - [ ] Operators `|>` and `<|` for shell command read/write?
- [ ] *deref-Operator to avoid automatic calling values when they are directly callable
- [ ] Parselets should allow for *args and **nargs catchall
- [ ] Token operators as generics (see on Priority also)
  - [ ] `until`-Operator (not available yet, but might be `Until<P, Escape=Void>`)
  - [ ] `not`-Operator (`Not<P>`)
  - [ ] `peek`-Operator (`Peek<P>`)
  - [ ] Generic `Repeat<P, min=1, max=0>` instead of `{min, max}` syntax considered below
  - (Old topic) Token call modifiers
    - [x] `expect` keyword
    - [x] `not` keyword
    - [x] `peek` keyword
    - [ ] Min(-Max)-Modifier syntax, e.g. `'t'{2, 4}` allowing for `tt`, `ttt`, `ttt` but not `tttt` .... `'t'{2}` should also work (syntax is ugly)
- [x] `push` and `next` for sequences

## Compiler

- [ ] Parser improvements
  - [x] Unescaping of character-class items
  - [ ] Use built-in tokens like Integer or Float on appropriate positions

## REPL

- [ ] REPL creates a new main parselet for every prompt executed; Old mains stay until program end.
- [ ] Main scope stays consumable even when the next prompt inserted unconsumable input

## Semantics

- [x] Use capitalized identifiers for consumable constants
- [ ] Undefined variables incremented or decremented (`i++`, `++i`, `i--`, `--i`) as well as variables assigned by `+=`, `-=`, `*=`, `/=` should enforce initialize the undefined variable to 0, so for a simple counting, an explicit setting to 0 is not required.
- [ ] Capture alias inferring, so `name => Name $n` $n shorthands $name?

## Optimization

- [x] Compiler: Check for existing static values and reuse them on redefinition.
- [x] Optimize away single-item sequences and blocks, use `Op::from_vec()` whenever Sequences without aliases are used
- [x] Modifier optimization, modifiers should generate different code when used by char-class, string, parselet, e.g.

## Built-ins

- [ ] Implement a generic `Token` builtin-token that matches anything belonging togeter (Identifier, Punctuation), except whitespace?
- [ ] Implement `Float`
  - [ ] What about scientific notations like `1.3e-08`?
- [ ] Implement `Number`, as the union of `Integer` and `Float`
- [x] Implement `Word` with min-parameter `Word(min=3)` to accept words with specified length only
- [ ] Further built-in and pre-defined tokens for matching standard cases like
  - [ ] Builtin tokens or character classes
    - `Alphabetic`
    - `Alphanumeric`
    - `Ascii`
    - `AsciiAlphabetic`
    - `AsciiAlphanumeric`
    - `AsciiControl`
    - `AsciiDigit`
    - `AsciiGraphic`
    - `AsciiHexdigit`
    - `AsciiLowercase`
    - `AsciiPunctuation`
    - `AsciiUppercase`
    - `AsciiWhitespace`
    - `Control`
    - `Digit`
    - `Lowercase`
    - `Numeric`
    - `Uppercase`
    - `Whitespace`
  - [ ] Pre-defined tokens matching `Alphabetic+`
    - `Alphabetics`
    - `Alphanumerics`
    - `Asciis`
    - `AsciiAlphabetics`
    - `AsciiAlphanumerics`
    - `AsciiControls`
    - `AsciiDigits`
    - `AsciiGraphics`
    - `AsciiHexdigits`
    - `AsciiLowercases`
    - `AsciiPunctuations`
    - `AsciiUppercases`
    - `AsciiWhitespaces`
    - `Controls`
    - `Digits`
    - `Lowercases`
    - `Numerics`
    - `Uppercases`
    - `Whitespaces` exists!

## Tests

We need sooo many tests!!!

- [x] starting with the stuff from main.rs
- [ ] Tests for expect
- [ ] Tests for peek
- [ ] Tests for not
- [ ] Lists
- [ ] Dicts
- [ ] Sequences with aliases
- [ ] Sequences with aliases and normal items
