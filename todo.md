# Brainstorming & Todo

- [ ] Row & column number recording in Reader/Range by working with an Offset data structure rather than usize to allow further tracking, e.g. of row + col numbers
- [ ] Test suite starting with the stuff from main.rs
- [ ] Syntactical definition of Char parser structure `[A-Za-z_]` etc...
- [ ] Capture::Named alias inferring
- [ ] Quantifier optimization
      - Positive modifier must generate different code when used by char-class, string, parselet, e.g.
        - Achivement via an into_repeat() function on the Parser trait
        - `[0-9]+` => `Char([0-9], repeats=True)`
        - `[0-9]*` => `Repeat(Char([0-9], repeats=True), 0, 1)`
        - `"Hallo"+` => `Repeat(Op::Match("Hallo"), 1, 0)`
        - `P+` => `@P' { P' P ; P }` (left-recursive repetition)
- Extensions by third-party crates
    - [ ] indexmap for Dict (https://github.com/bluss/indexmap)
    - [ ] lazy_static for parser and compiler
    - [ ] regex for regular expressions
- Use `>>` as alternative to `begin`-keyword
- Use `<<` as alternative to `end`-keyword
