# Brainstorming & Todo

- reader.rs
  - [ ] Work with an Offset data structure rather than usize to additionally track row + col numbers
- ccl.rs
  - [ ] Charset parser written in Tokay
- tokay.rs
  - Runtime/Context
    - [ ] Merge capture and stack into one separate data structure
    - [ ] 
    - [ ] Infer alias variables, like `Expr '+' Term` can be matched by any $Expr, $expr, $Ex or just $e, when no direct match is found

- General
  - Quantifier Optimization
    - Positive modifier must generate different code when used by char-class, string, parselet, e.g.
      - `|0-9|+` => `Atomic::Token(Chars(|0-9|))`
      - `"Hallo"+` => `Repeat(Atomic::Token(Match("Hallo")), 1, 0, false)`
      - `P+` => `@P' { P' P ; P }` (left-recursive repetition)
