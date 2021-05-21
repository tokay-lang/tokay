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
  - [ ] **log** crate and related logger
- [ ] Grammar view
  - [ ] Perform left-recursion detection on begin and end also?
  - [ ] Resolve indirect left-recursion as done in pegen? (see src/test.rs comments)

## Syntax

Syntax is under careful consideration.

- [x] Implementation of Lists `(a, b, c)` and Dicts `(a => b, c => d)`
  - [ ] instead of `=>`, consider to use `>>` and `<<` syntax to allow for key-value or value-key notation; This might be useful, e.g. when in sequences `Integer << i` rather than `i >> Integer`, but `a -> b` in a dict...
- [ ] Missing expressional constructs
  - [x] Assignment operations `+=`, `-=`, `*=`, `/=`
  - [x] `&&` and `||`
  - [ ] `&` and `|`
  - [ ] `^` (pow)
- [ ] Loops with value collection using `continue` and `break`
- [x] Sequence item aliasing: `a x => b c $2 $x $1
- Token call modifiers
  - [x] `expect` keyword
  - [x] `not` keyword
  - [x] `peek` keyword
  - [ ] Min(-Max)-Modifier syntax, e.g. `'t'{2, 4}` allowing for `tt`, `ttt`, `ttt` but not `tttt` .... `'t'{2}` should also work (syntax is ugly)
- [x] Definition of Chars tokens `[A-Za-z_]` etc...
- [!] Definition of Regex tokens `/Hel+o Wo?rld/` (not now, see https://github.com/phorward/tokay/issues/1)
- [ ] Implement `ls -ltra` backticks (`) for shell command values
  - [ ] Operators `|>` and `<|` for shell command read/write?
- [ ] until-Operator?
- [ ] *deref-Operator?
- [ ] Parselets should allow for *args and **nargs catchall

## Compiler

- [x] Don't require to re-initialize builtins every time a new compile is done.
- [x] Re-use statics when accessed multiple times
- [ ] Missing traversals for
  - [x] Comparison operators
  - [x] In-place increment and decrement `a++`, `++a`
  - [ ] for-loops
  - [ ] while-loops
  - [ ] break and continue
  - [x] Assignment operations `+=`, `-=`, `*=`, `/=`

## REPL

- [x] REPL should hold global variables and statics context during execution
- [ ] REPL creates a new main parselet for every prompt executed; Old mains stay until program end.
- [ ] Main scope stays consumable even when the next prompt inserted unconsumable input

## Semantics

- [x] Use capitalized identifiers for consumable constants
- [ ] Undefined variables incremented or decremented (`i++`, `++i`, `i--`, `--i`) as well as variables assigned by `+=`, `-=`, `*=`, `/=` should enforce initialize the undefined variable to 0, so for a simple counting, an explicit setting to 0 is not required.
- [ ] Capture::Named alias inferring
- [ ] Capture::Named should recognize severity?
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
- [ ] Lists
- [ ] Dicts
- [ ] Sequences with aliases
- [ ] Sequences with aliases and normal items

## Bugs

- [x] `x : print` is not possible
- [ ] builtins::collect() doesn't recognize correct total offset when main program is e.g. just `Integer collect`.
