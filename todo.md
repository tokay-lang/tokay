# Brainstorming & Todo

This is a recently updated brainstorming and todo scribble file.

## Pritority

- [x] Parametized parselets
- [ ] Generic parselets

## General

- [x] Row & column number recording in Reader/Range by working with an Offset data structure rather than usize to allow further tracking, e.g. of row + col numbers
- Extensions by third-party crates
    - [ ] **indexmap** for Dict (https://github.com/bluss/indexmap)
    - [ ] **lazy_static** for parser and compiler

## Syntax

Syntax is under careful consideration.

- [ ] Missing expressional constructs
  - [ ] Assignment operations `+=`, `-=`, `*=`, `/=`
  - [ ] `&&` and `||`
  - [ ] `&` and `|`
  - [ ] `^`
- [ ] Loops with value collection using `continue` and `break`
- [ ] Sequence item aliasing: `a b::x c $2 $x $1
- Token call modifiers
  - [ ] How to distinguish token symbolic constants?
  - [ ] `expect` keyword
  - [ ] `not` keyword
  - [ ] `peek` keyword
  - [ ] Min(-Max)-Modifier syntax, e.g. `'t'{2, 4}` allowing for `tt`, `ttt`, `ttt` but not `tttt` .... `'t'{2}` should also work
- [x] Definition of Chars tokens `[A-Za-z_]` etc...
- [ ] Definition of Regex tokens `/Hel+o Wo?rld/` (not now, see https://github.com/phorward/tokay/issues/1)
- [ ] Implement `...` backticks for shell command values
  - [ ] Operators `>>` and `<<` for shell command read/write?
- [ ] until-Operator?
- [ ] *deref-Operator?

## Compiler

- [x] Don't require to re-initialize builtins every time a new compile is done.
- [x] Re-use statics when accessed multiple times
- [ ] Missing traversals for
  - [x] Comparison operators
  - [ ] In-place increment and decrement
  - [ ] while-loops
  - [ ] for-loops
  - [ ] break and continue
  - [ ] Assignment operations `+=`, `-=`, `*=`, `/=`

## REPL

- [x] REPL should hold global variables and statics context during execution
- [ ] REPL creates a new main parselet for every prompt executed; Old mains stay until program end.
- [ ] Main scope stays consumable even when the next prompt inserted unconsumable input

## Semantics

- [x] Use capitalized identifiers for consumable constants
- [ ] Capture::Named alias inferring
- [ ] Integer division `1/6` returns 0, but should return float. `1./6` correctly returns 0.16666666666666666
- [ ] Use string arithmetics for something like 123 ^ 3000 later on, which cannot be handled by i64.

## Optimization

- [x] Compiler: Check for existing static values and reuse them on redefinition.
- [x] Optimize away single-item sequences and blocks, use `Op::from_vec()` whenever Sequences without aliases are used
- [x] Modifier optimization, modifiers should generate different code when used by char-class, string, parselet, e.g.


## Tests

- [x] starting with the stuff from main.rs
- [ ] Tests for expect
- [ ] Tests for peek
- [ ] Tests for not

## Bugs

- [x] `x : print` is not possible
- [ ] builtins::collect() doesn't recognize correct total offset when main program is e.g. just `Integer collect`.
