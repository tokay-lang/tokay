//! Tokay builtin registry

// The content of this file is automatically generated by `_builtins.tok` from the Rust source code.
// To update the content of this file, `cd build` and run `make builtins` in a shell.

use crate::builtin::Builtin;

/*GENERATE cargo run -- _builtins.tok -- `find . -name "*.rs"` */
pub static BUILTINS: [Builtin; 65] = [
    Builtin {
        name: "Float",
        func: crate::value::token::tokay_token_float,
    },
    Builtin {
        name: "Ident",
        func: crate::value::token::tokay_token_ident,
    },
    Builtin {
        name: "Int",
        func: crate::value::token::tokay_token_int,
    },
    Builtin {
        name: "Word",
        func: crate::value::token::tokay_token_word,
    },
    Builtin {
        name: "ast",
        func: crate::compiler::ast::tokay_function_ast,
    },
    Builtin {
        name: "ast2rust",
        func: crate::compiler::ast::tokay_function_ast2rust,
    },
    Builtin {
        name: "ast_print",
        func: crate::compiler::ast::tokay_function_ast_print,
    },
    Builtin {
        name: "bool",
        func: crate::value::value::Value::tokay_method_bool,
    },
    Builtin {
        name: "chr",
        func: crate::builtin::tokay_function_chr,
    },
    Builtin {
        name: "debug",
        func: crate::builtin::tokay_function_debug,
    },
    Builtin {
        name: "dict",
        func: crate::value::dict::Dict::tokay_method_dict,
    },
    Builtin {
        name: "dict_clone",
        func: crate::value::dict::Dict::tokay_method_dict_clone,
    },
    Builtin {
        name: "dict_get_item",
        func: crate::value::dict::Dict::tokay_method_dict_get_item,
    },
    Builtin {
        name: "dict_items",
        func: crate::value::dict::Dict::tokay_method_dict_items,
    },
    Builtin {
        name: "dict_keys",
        func: crate::value::dict::Dict::tokay_method_dict_keys,
    },
    Builtin {
        name: "dict_len",
        func: crate::value::dict::Dict::tokay_method_dict_len,
    },
    Builtin {
        name: "dict_merge",
        func: crate::value::dict::Dict::tokay_method_dict_merge,
    },
    Builtin {
        name: "dict_pop",
        func: crate::value::dict::Dict::tokay_method_dict_pop,
    },
    Builtin {
        name: "dict_push",
        func: crate::value::dict::Dict::tokay_method_dict_push,
    },
    Builtin {
        name: "dict_set_item",
        func: crate::value::dict::Dict::tokay_method_dict_set_item,
    },
    Builtin {
        name: "eof",
        func: crate::builtin::tokay_function_eof,
    },
    Builtin {
        name: "error",
        func: crate::error::tokay_function_error,
    },
    Builtin {
        name: "float",
        func: crate::value::value::Value::tokay_method_float,
    },
    Builtin {
        name: "float_ceil",
        func: crate::value::value::Value::tokay_method_float_ceil,
    },
    Builtin {
        name: "float_fract",
        func: crate::value::value::Value::tokay_method_float_fract,
    },
    Builtin {
        name: "float_trunc",
        func: crate::value::value::Value::tokay_method_float_trunc,
    },
    Builtin {
        name: "int",
        func: crate::value::value::Value::tokay_method_int,
    },
    Builtin {
        name: "iter",
        func: crate::value::iter::iter::Iter::tokay_method_iter,
    },
    Builtin {
        name: "iter_collect",
        func: crate::value::iter::iter::Iter::tokay_method_iter_collect,
    },
    Builtin {
        name: "iter_enum",
        func: crate::value::iter::enumiter::EnumIter::tokay_method_iter_enum,
    },
    Builtin {
        name: "iter_len",
        func: crate::value::iter::iter::Iter::tokay_method_iter_len,
    },
    Builtin {
        name: "iter_map",
        func: crate::value::iter::mapiter::MapIter::tokay_method_iter_map,
    },
    Builtin {
        name: "iter_next",
        func: crate::value::iter::iter::Iter::tokay_method_iter_next,
    },
    Builtin {
        name: "iter_rev",
        func: crate::value::iter::iter::Iter::tokay_method_iter_rev,
    },
    Builtin {
        name: "list",
        func: crate::value::list::List::tokay_method_list,
    },
    Builtin {
        name: "list_add",
        func: crate::value::list::List::tokay_method_list_add,
    },
    Builtin {
        name: "list_flatten",
        func: crate::value::list::List::tokay_method_list_flatten,
    },
    Builtin {
        name: "list_get_item",
        func: crate::value::list::List::tokay_method_list_get_item,
    },
    Builtin {
        name: "list_iadd",
        func: crate::value::list::List::tokay_method_list_iadd,
    },
    Builtin {
        name: "list_len",
        func: crate::value::list::List::tokay_method_list_len,
    },
    Builtin {
        name: "list_pop",
        func: crate::value::list::List::tokay_method_list_pop,
    },
    Builtin {
        name: "list_push",
        func: crate::value::list::List::tokay_method_list_push,
    },
    Builtin {
        name: "list_set_item",
        func: crate::value::list::List::tokay_method_list_set_item,
    },
    Builtin {
        name: "list_sort",
        func: crate::value::list::List::tokay_method_list_sort,
    },
    Builtin {
        name: "offset",
        func: crate::builtin::tokay_function_offset,
    },
    Builtin {
        name: "ord",
        func: crate::builtin::tokay_function_ord,
    },
    Builtin {
        name: "print",
        func: crate::builtin::tokay_function_print,
    },
    Builtin {
        name: "range",
        func: crate::builtin::range::tokay_function_range,
    },
    Builtin {
        name: "repr",
        func: crate::builtin::tokay_function_repr,
    },
    Builtin {
        name: "str",
        func: crate::value::str::Str::tokay_method_str,
    },
    Builtin {
        name: "str_add",
        func: crate::value::str::Str::tokay_method_str_add,
    },
    Builtin {
        name: "str_byteslen",
        func: crate::value::str::Str::tokay_method_str_byteslen,
    },
    Builtin {
        name: "str_endswith",
        func: crate::value::str::Str::tokay_method_str_endswith,
    },
    Builtin {
        name: "str_find",
        func: crate::value::str::Str::tokay_method_str_find,
    },
    Builtin {
        name: "str_get_item",
        func: crate::value::str::Str::tokay_method_str_get_item,
    },
    Builtin {
        name: "str_join",
        func: crate::value::str::Str::tokay_method_str_join,
    },
    Builtin {
        name: "str_len",
        func: crate::value::str::Str::tokay_method_str_len,
    },
    Builtin {
        name: "str_lower",
        func: crate::value::str::Str::tokay_method_str_lower,
    },
    Builtin {
        name: "str_mul",
        func: crate::value::str::Str::tokay_method_str_mul,
    },
    Builtin {
        name: "str_replace",
        func: crate::value::str::Str::tokay_method_str_replace,
    },
    Builtin {
        name: "str_split",
        func: crate::value::str::Str::tokay_method_str_split,
    },
    Builtin {
        name: "str_startswith",
        func: crate::value::str::Str::tokay_method_str_startswith,
    },
    Builtin {
        name: "str_substr",
        func: crate::value::str::Str::tokay_method_str_substr,
    },
    Builtin {
        name: "str_upper",
        func: crate::value::str::Str::tokay_method_str_upper,
    },
    Builtin {
        name: "type",
        func: crate::builtin::tokay_function_type,
    },
];
/*ETARENEG*/
