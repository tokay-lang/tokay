# Roadmap

This document describes upcoming changes to achieve with a specific version.

## 0.7

- [x] Implement iterators and `for...in`-syntax (#101)
- [x] Implement generic parselets (#10, #105)
  - [x] `Keyword<P>` (#121)
  - [ ] `Until<P, Escape: '\\'>`
  - [ ] `String<Start, End: Void, Escape: '\\'>`
- [ ] Implement inlined parselets (#120)
- [x] New list syntax `,`, redefining sequence/`dict` syntax (#100)
  - Top-level `list` definition `l = ,`
  - Top-level `dict` definition `d = ()`
- [ ] Compiler refactoring and improvements
  - [x] Scope-struct
  - [ ] Clean-up Scope-Compiler-intermezzo
  - [x] `void` and `Void`
  - [x] `Empty`
  - [ ] ImlRefValue
  - [ ] ImlVariable

## 0.8

- [ ] Importable modules
