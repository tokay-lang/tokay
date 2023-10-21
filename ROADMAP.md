# Roadmap

This document describes upcoming changes to achieve with a specific version.

## 0.7

- [x] Implement iterators and `for...in`-syntax (#101)
- [x] Implement generic parselets (#10, #105)
- [ ] Implement embedded parselets (#120)
- [ ] New list syntax `[...]`, redefining sequence/`dict` syntax (#100)
  - The character-class token syntax was replaced by a `Char`-builtin
  - List definition `list = []`
  - Dict definition `dict = ()`
  - Builtins `dict` and `list` should become obsolete, so variables can take their names
