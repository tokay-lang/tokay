# Roadmap

This document describes upcoming changes to achieve with a specific version.

## 0.7

- [ ] Implement iterators (#101)
- [ ] Implement generic parselets (#10)
- [ ] New list syntax `[...]`, redefining sequence/`dict` syntax (#100)
  - The character-class token syntax will be replaced by a `Char`-builtin
  - List definition `list = []`
  - Dict definition `dict = ()`
  - Builtins `dict` and `list` should become obsolete, so variables can take their names
