# Value

There are 3 levels values are made of

- `RefValue` is a reference-counted (Rc) `Value` with dynamic borrowing (RefCell)
  - It implements `Object` for easier usage in common cases.
  - It provides `borrow()` and `borrow_mut()` when explicit borrowing is wanted.
  - It provides `unary_op()` and `binary_op()` functions to perform operations with values.
    - There are fast-lane implementations for any Value-item, except Object.
    - In case of an object, a builtin method named `<object>_<operation>()` is tried to be called.
- `Value` is an enum, either saving atomics, numeric values, or `dyn Object`.
  - It implements `Object` as well.
  - It implements `object::<T>()`, `object_mut::<T>()` and `into_object_::<T>()` to downcast an object to its specific type.
- `Object` is a trait serving an interface to any more specific type of data or information.
  - Current implementation for `Builtin`, `Dict`, `List`, `Parselet`, `Str`, `Token`.
  - It provides methods to quickly access Rust primary types, like `is_true()`, `to_i64()` or `to_string()`.

# Binary operation conversions

This is how Tokay builtin values are converted during binary operations.

|  + * - /  | **void** | **null** | **bool** | **int** | **float** | **addr** | **str** | **dict** | **list**
| --------- | -------- | -------- | -------- | ------- | --------- | -------- | ------- | -------- | --------
| **void**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **null**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **bool**  |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **int**   |   int    |   int    |   int    |   int   |   float   |   addr   |   str   |   dict   |   list
| **float** |   float  |   float  |   float  |   float |   float   |   addr   |   str   |   dict   |   list
| **str**   |   str    |   str    |   str    |   str   |   str     |   str    |   str   |   dict   |   list
| **dict**  |   dict   |   dict   |   dict   |   dict  |   dict    |   dict   |   dict  |   dict   |   list
| **list**  |   list   |   list   |   list   |   list  |   list    |   list   |   list  |   list   |   list

# Token severity

| Severity | Used by                                                                 |
| -------- | ----------------------------------------------------------------------- |
|        0 | `_`, `__`, `Touch`                                                      |
|        5 | Any token (`Char`, `Match`, `Int`, `Float`, `Number`), parselet default |
|       10 | Any explicitly pushed value                                             |
