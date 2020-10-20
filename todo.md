# Brainstorming & Todo

- reader.rs
  - [ ] Work with an Offset data structure rather than usize to allow
        further tracking, e.g. of row + col numbers
- ccl.rs
  - [ ] Charset parser written in Tokay
- tokay.rs
  - Runtime/Context
    - [ ] Merge capture and stack into one separate data structure
      - [ ] Capture should always become a Range
      - [ ] Remove any severities; Capture always considered as lowest severity.
    - [ ] Infer alias variables, like `Expr '+' Term` can be matched by any $Expr, $expr, $Ex or just $e, when no direct match is found

- General
  - Quantifier Optimization
    - Positive modifier must generate different code when used by char-class, string, parselet, e.g.
      - Achivement via an into_repeat() function on the Parser trait
      - `|0-9|+` => `Char(|0-9|, repeats=True)`
      - `|0-9|*` => `Repeat(Char(|0-9|, repeats=True), 0, 1)`
      - `"Hallo"+` => `Repeat(Op::Token(Match("Hallo")), 1, 0)`
      - `P+` => `@P' { P' P ; P }` (left-recursive repetition)
