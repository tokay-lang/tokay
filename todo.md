# Brainstorming & Todo

This is a recently updated brainstorming and todo scribble file.

## Pritority

- [ ] Parametized parselets
- [ ] Generic parselets

## General

- [x] Row & column number recording in Reader/Range by working with an Offset data structure rather than usize to allow further tracking, e.g. of row + col numbers
- Extensions by third-party crates
    - [ ] **indexmap** for Dict (https://github.com/bluss/indexmap)
    - [ ] **lazy_static** for parser and compiler
    - [ ] **regex** for regular expressions

## Syntax

Syntax is under careful consideration.

- [ ] Use `>>` as alternative for `begin`-keyword
- [ ] Use `<<` as alternative for `end`-keyword
- Token call modifiers
  - [ ] How to distinguish token symbolic constants?
  - [ ] `expect` keyword
  - [ ] `not` keyword
  - [ ] `peek` keyword
  - [ ] Min(-Max)-Modifier syntax, e.g. `'t'{2, 4}` allowing for `tt`, `ttt`, `ttt` but not `tttt` .... `'t'{2}` should also work
  - [ ] Prefixed token modifier keywords `pos`, `kle`, `opt`, instead of `+`, `*`, `?`, e.g. `expect pos X kle X opt X`
    - The postfix-notation is more staight-forward, but its bad on symbols which aren't consumable at least and crashes with the syntax of add, mul and sequence, so `a+ 1` (`kle a 1`) != `a + 1` (`a + 1`) != `a +1` (`a +1`)
- [ ] Definition of Chars scanable structure `[A-Za-z_]` etc...
- [ ] Definition of Regex scanable structure `/Hel+o Wo?rld/`
- [ ] Implement ` backticks for shell commands

## Semantics

- [ ] Use capitalized identifiers for consumable statics only?
  - Examples
    - `A : 'Hello'                  # turns into @{'Hello'}`
    - `A : X                        # turns into @{ X }. Because X is upper-case, it is consumable.`
    - `A : X+                       # turns into @{ X+ }`
    - `A : @{'Hello'; A 'World'+ }  # just fine :-), a consuming parselet`
    - `A : @{ if lol 'Hallo' }      # even just fine :-), a consuming parselet when lol is true.`
    - `A : 123                      # Error value is not consumable`
    - `A : @{ "Hello" }             # Error value is not consumable`
    - `a : 'Hello'                  # turns into @{'Hello'}, just a parselet value`
    - `a : X+                       # turns into @{ X+ }, just a parselet value`
    - `a : @{'Hello'; A 'World'+ }  # taken as is, just a parselet-value`
    - `a : 123                      # taken as is, just an integer value`
    - `a : @{ "Hello" }             # taken as is, just a parselet value`
    - `Pi = 3.1415                  # Error capitalized identifier not allowed for variable`
    - `pi = 3.1415                  # OK just a variable`
  - Benefits
    - Solution for parameterized parselets with static values
    - A parselet can immediately be identified as consumable when it either calls
      - a scanable or
      - consuming parselet identified by a capitalized identifier again
    - It is backward-compatible to existing Tokay code
    - Parselet parameters immediatelly can be classified by their name if they're consumable
  - Drawbacks
    - `Pi` can't be used as identifier for a constant value `Pi` (but `pi` can!)
    - `a = @A, b, c=2 { A+ b c }` *A* is a consumable constant, *b* is always a variable, *c* is always a variable defaulting to 2.
      - `a = @A, b:, c=2 { A+ b c }` *A* is a consumable constant, *b* is a constant, *c* is always a variable defaulting to 2.
      - `a = @A, b: Integer, c=2 { A+ b c }` *A* is a consumable constant, *b* is a constant defaulting to Integer, *c* is always a variable defaulting to 2.
      - `a = @A, B: Integer, c=2 { A+ b c }` *A* is a consumable constant, *b* is a consumable constant defaulting to Integer, *c* is always a variable defaulting to 2.
- [ ] Capture::Named alias inferring
- [ ] Integer division `1/6` returns 0, but should return float. `1./6` correctly returns 0.16666666666666666

## Optimization

- [ ] Check for existing static values and reuse them on redefinition.
- [ ] Optimize away single-item sequences and blocks, use `Op::from_vec()` whenever Sequences without aliases are used
- [ ] Modifier optimization, modifiers should generate different code when used by char-class, string, parselet, e.g.
  - Achivement via an into_repeat() function on the Runable trait
    - `[0-9]+` => `Char([0-9], repeats=True)`
    - `[0-9]*` => `Repeat(Char([0-9], repeats=True), 0, 1)`
    - `"Hallo"+` => `Repeat(Op::Match("Hallo"), 1, 0)`
    - `P+` => `@P' { P' P ; P }` (left-recursive repetition)

## Tests

- [x] starting with the stuff from main.rs
- [ ] Tests for expect
- [ ] Tests for peek
- [ ] Tests for not
- [ ] Tests for kle, pos, opt as implemented in future (see above)
