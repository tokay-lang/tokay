/** Tokay default prelude

The prelude is a default stub of Tokay standard parselets, which implements
fundamental parts of the Tokay language itself.

It's defined in src/prelude.tok and pre-compiled as an AST within the code.
*/
use super::*;

impl Compiler {
    pub(super) fn load_prelude(&mut self) {
        // fixme: Make this lazy_static, so its created only once!
        let ast =
            /*GENERATE tokay -vmode=ast2rust -vlevel=3 src/compiler/Tokay.tok -- src/prelude.tok */
            crate::value!([
                "emit" => "main",
                "children" =>
                    (crate::value!([
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Not"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "sequence",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "P"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "op_reject"
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "Empty"
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Peek"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "sequence",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "P"
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "op_reset"
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Expect"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "sig",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "msg"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_void"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "op_accept",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "P"
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "call",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "error"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "callarg",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "op_logical_or",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "msg"
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "op_binary_add",
                                                                                                "children" =>
                                                                                                    (crate::value!([
                                                                                                        (crate::value!([
                                                                                                            "emit" => "op_binary_add",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "op_binary_add",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "value_string",
                                                                                                                                    "value" => "Expecting "
                                                                                                                                ])),
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "op_deref",
                                                                                                                                    "children" =>
                                                                                                                                        (crate::value!([
                                                                                                                                            "emit" => "identifier",
                                                                                                                                            "value" => "P"
                                                                                                                                        ]))
                                                                                                                                ]))
                                                                                                                            ]))
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "value_string",
                                                                                                                        "value" => ", but got "
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ])),
                                                                                                        (crate::value!([
                                                                                                            "emit" => "call",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "identifier",
                                                                                                                        "value" => "repr"
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "callarg",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                "emit" => "value_instance",
                                                                                                                                "children" =>
                                                                                                                                    (crate::value!([
                                                                                                                                        (crate::value!([
                                                                                                                                            "emit" => "identifier",
                                                                                                                                            "value" => "Peek"
                                                                                                                                        ])),
                                                                                                                                        (crate::value!([
                                                                                                                                            "emit" => "instarg",
                                                                                                                                            "children" =>
                                                                                                                                                (crate::value!([
                                                                                                                                                    "emit" => "block",
                                                                                                                                                    "children" =>
                                                                                                                                                        (crate::value!([
                                                                                                                                                            (crate::value!([
                                                                                                                                                                "emit" => "identifier",
                                                                                                                                                                "value" => "Token"
                                                                                                                                                            ])),
                                                                                                                                                            (crate::value!([
                                                                                                                                                                "emit" => "value_token_any"
                                                                                                                                                            ])),
                                                                                                                                                            (crate::value!([
                                                                                                                                                                "emit" => "value_string",
                                                                                                                                                                "value" => "end-of-file"
                                                                                                                                                            ]))
                                                                                                                                                        ]))
                                                                                                                                                ]))
                                                                                                                                        ]))
                                                                                                                                    ]))
                                                                                                                            ]))
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ]))
                                                                                                    ]))
                                                                                            ]))
                                                                                        ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Repeat"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "min"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_integer",
                                                                "value" => 1
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "max"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_void"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "blur"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_true"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "assign_drop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "res"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "list"
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "assign_drop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "cnt"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "value_integer",
                                                                            "value" => 0
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_loop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        "emit" => "block",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "sequence",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "P"
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "block",
                                                                                                "children" =>
                                                                                                    (crate::value!([
                                                                                                        (crate::value!([
                                                                                                            "emit" => "call",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "rvalue",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "identifier",
                                                                                                                                    "value" => "res"
                                                                                                                                ])),
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "attribute",
                                                                                                                                    "children" =>
                                                                                                                                        (crate::value!([
                                                                                                                                            "emit" => "value_string",
                                                                                                                                            "value" => "push"
                                                                                                                                        ]))
                                                                                                                                ]))
                                                                                                                            ]))
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "callarg",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                "emit" => "capture_index",
                                                                                                                                "children" =>
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "value_integer",
                                                                                                                                        "value" => 1
                                                                                                                                    ]))
                                                                                                                            ]))
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ])),
                                                                                                        (crate::value!([
                                                                                                            "emit" => "assign_add_drop",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "lvalue",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                "emit" => "identifier",
                                                                                                                                "value" => "cnt"
                                                                                                                            ]))
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "value_integer",
                                                                                                                        "value" => 1
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ])),
                                                                                                        (crate::value!([
                                                                                                            "emit" => "op_if",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "op_logical_and",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "identifier",
                                                                                                                                    "value" => "max"
                                                                                                                                ])),
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "comparison",
                                                                                                                                    "children" =>
                                                                                                                                        (crate::value!([
                                                                                                                                            (crate::value!([
                                                                                                                                                "emit" => "identifier",
                                                                                                                                                "value" => "cnt"
                                                                                                                                            ])),
                                                                                                                                            (crate::value!([
                                                                                                                                                "emit" => "cmp_eq",
                                                                                                                                                "children" =>
                                                                                                                                                    (crate::value!([
                                                                                                                                                        "emit" => "identifier",
                                                                                                                                                        "value" => "max"
                                                                                                                                                    ]))
                                                                                                                                            ]))
                                                                                                                                        ]))
                                                                                                                                ]))
                                                                                                                            ]))
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "op_break"
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ]))
                                                                                                    ]))
                                                                                            ]))
                                                                                        ]))
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "op_if",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "comparison",
                                                                                                "children" =>
                                                                                                    (crate::value!([
                                                                                                        (crate::value!([
                                                                                                            "emit" => "identifier",
                                                                                                            "value" => "cnt"
                                                                                                        ])),
                                                                                                        (crate::value!([
                                                                                                            "emit" => "cmp_lt",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    "emit" => "identifier",
                                                                                                                    "value" => "min"
                                                                                                                ]))
                                                                                                        ]))
                                                                                                    ]))
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "op_reject"
                                                                                            ]))
                                                                                        ]))
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "op_break"
                                                                                ]))
                                                                            ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_if",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "comparison",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "identifier",
                                                                                        "value" => "cnt"
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "cmp_lt",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "min"
                                                                                            ]))
                                                                                    ]))
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "op_reject"
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_if",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "blur"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "block",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "op_if",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                (crate::value!([
                                                                                                    "emit" => "comparison",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            (crate::value!([
                                                                                                                "emit" => "rvalue",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "identifier",
                                                                                                                            "value" => "res"
                                                                                                                        ])),
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "attribute",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "value_string",
                                                                                                                                    "value" => "len"
                                                                                                                                ]))
                                                                                                                        ]))
                                                                                                                    ]))
                                                                                                            ])),
                                                                                                            (crate::value!([
                                                                                                                "emit" => "cmp_eq",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "value_integer",
                                                                                                                        "value" => 0
                                                                                                                    ]))
                                                                                                            ]))
                                                                                                        ]))
                                                                                                ])),
                                                                                                (crate::value!([
                                                                                                    "emit" => "op_accept",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            "emit" => "value_void"
                                                                                                        ]))
                                                                                                ]))
                                                                                            ]))
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "op_if",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                (crate::value!([
                                                                                                    "emit" => "comparison",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            (crate::value!([
                                                                                                                "emit" => "rvalue",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "identifier",
                                                                                                                            "value" => "res"
                                                                                                                        ])),
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "attribute",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "value_string",
                                                                                                                                    "value" => "len"
                                                                                                                                ]))
                                                                                                                        ]))
                                                                                                                    ]))
                                                                                                            ])),
                                                                                                            (crate::value!([
                                                                                                                "emit" => "cmp_eq",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "value_integer",
                                                                                                                        "value" => 1
                                                                                                                    ]))
                                                                                                            ]))
                                                                                                        ]))
                                                                                                ])),
                                                                                                (crate::value!([
                                                                                                    "emit" => "op_accept",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            "emit" => "rvalue",
                                                                                                            "children" =>
                                                                                                                (crate::value!([
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "identifier",
                                                                                                                        "value" => "res"
                                                                                                                    ])),
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "item",
                                                                                                                        "children" =>
                                                                                                                            (crate::value!([
                                                                                                                                "emit" => "value_integer",
                                                                                                                                "value" => 0
                                                                                                                            ]))
                                                                                                                    ]))
                                                                                                                ]))
                                                                                                        ]))
                                                                                                ]))
                                                                                            ]))
                                                                                    ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "res"
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Pos"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "blur"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_true"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "value_instance",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "Repeat"
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "instarg",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                "emit" => "identifier",
                                                                                "value" => "P"
                                                                            ]))
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "instarg_named",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "blur"
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "blur"
                                                                                ]))
                                                                            ]))
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Kle"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "blur"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_true"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "op_logical_or",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "value_instance",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "Repeat"
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "instarg",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            "emit" => "identifier",
                                                                                            "value" => "P"
                                                                                        ]))
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "instarg_named",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "min"
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "value_integer",
                                                                                                "value" => 0
                                                                                            ]))
                                                                                        ]))
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "instarg_named",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "blur"
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "blur"
                                                                                            ]))
                                                                                        ]))
                                                                                ]))
                                                                            ]))
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "value_void"
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Opt"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "block",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "P"
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "Empty"
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "List"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "Separator"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "sequence",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "value_token_touch",
                                                                            "value" => ","
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "_"
                                                                        ]))
                                                                    ]))
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "empty"
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "value_true"
                                                            ]))
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "sequence",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "Self"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "Separator"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "P"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "op_binary_add",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "capture_index",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                "emit" => "value_integer",
                                                                                                "value" => 1
                                                                                            ]))
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "capture_index",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                "emit" => "value_integer",
                                                                                                "value" => 3
                                                                                            ]))
                                                                                    ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_if",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "empty"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "sequence",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "identifier",
                                                                                        "value" => "Self"
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "identifier",
                                                                                        "value" => "Separator"
                                                                                    ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "sequence",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "P"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "list",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "capture_index",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            "emit" => "value_integer",
                                                                                            "value" => 1
                                                                                        ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Keyword"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "gen",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "P"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "sequence",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "identifier",
                                                                        "value" => "P"
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "value_instance",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "Not"
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "instarg",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            "emit" => "block",
                                                                                            "children" =>
                                                                                                (crate::value!([
                                                                                                    (crate::value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "value" => "Alphanumeric"
                                                                                                    ])),
                                                                                                    (crate::value!([
                                                                                                        "emit" => "value_token_touch",
                                                                                                        "value" => "_"
                                                                                                    ]))
                                                                                                ]))
                                                                                        ]))
                                                                                ]))
                                                                            ]))
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Number"
                                    ])),
                                    (crate::value!([
                                        "emit" => "block",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "identifier",
                                                    "value" => "Float"
                                                ])),
                                                (crate::value!([
                                                    "emit" => "identifier",
                                                    "value" => "Int"
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "Token"
                                    ])),
                                    (crate::value!([
                                        "emit" => "block",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "identifier",
                                                    "value" => "AsciiPunctuation"
                                                ])),
                                                (crate::value!([
                                                    "emit" => "identifier",
                                                    "value" => "Word"
                                                ])),
                                                (crate::value!([
                                                    "emit" => "identifier",
                                                    "value" => "Number"
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "max"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "sig",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "value"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "rvalue",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "call",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "iter"
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "callarg",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            "emit" => "identifier",
                                                                                            "value" => "value"
                                                                                        ]))
                                                                                ]))
                                                                            ]))
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "attribute",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                "emit" => "value_string",
                                                                                "value" => "max"
                                                                            ]))
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "min"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "sig",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "value"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "rvalue",
                                                            "children" =>
                                                                (crate::value!([
                                                                    (crate::value!([
                                                                        "emit" => "call",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "iter"
                                                                                ])),
                                                                                (crate::value!([
                                                                                    "emit" => "callarg",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            "emit" => "identifier",
                                                                                            "value" => "value"
                                                                                        ]))
                                                                                ]))
                                                                            ]))
                                                                    ])),
                                                                    (crate::value!([
                                                                        "emit" => "attribute",
                                                                        "children" =>
                                                                            (crate::value!([
                                                                                "emit" => "value_string",
                                                                                "value" => "min"
                                                                            ]))
                                                                    ]))
                                                                ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "sum"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "sig",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "value"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "assign_drop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "res"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "value_void"
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_for",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "i"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "value"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "block",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "assign_add_drop",
                                                                                    "children" =>
                                                                                        (crate::value!([
                                                                                            (crate::value!([
                                                                                                "emit" => "lvalue",
                                                                                                "children" =>
                                                                                                    (crate::value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "value" => "res"
                                                                                                    ]))
                                                                                            ])),
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "i"
                                                                                            ]))
                                                                                        ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "identifier",
                                                                "value" => "res"
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ])),
                        (crate::value!([
                            "emit" => "constant",
                            "children" =>
                                (crate::value!([
                                    (crate::value!([
                                        "emit" => "identifier",
                                        "value" => "avg"
                                    ])),
                                    (crate::value!([
                                        "emit" => "value_parselet",
                                        "children" =>
                                            (crate::value!([
                                                (crate::value!([
                                                    "emit" => "sig",
                                                    "children" =>
                                                        (crate::value!([
                                                            "emit" => "identifier",
                                                            "value" => "value"
                                                        ]))
                                                ])),
                                                (crate::value!([
                                                    "emit" => "body",
                                                    "children" =>
                                                        (crate::value!([
                                                            (crate::value!([
                                                                "emit" => "assign_drop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "res"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "value_void"
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "assign_drop",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "cnt"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "value_integer",
                                                                            "value" => 0
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_for",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "lvalue",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "identifier",
                                                                                    "value" => "i"
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "value"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "block",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "assign_add_drop",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                (crate::value!([
                                                                                                    "emit" => "lvalue",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            "emit" => "identifier",
                                                                                                            "value" => "res"
                                                                                                        ]))
                                                                                                ])),
                                                                                                (crate::value!([
                                                                                                    "emit" => "identifier",
                                                                                                    "value" => "i"
                                                                                                ]))
                                                                                            ]))
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "inplace_post_inc",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                "emit" => "lvalue",
                                                                                                "children" =>
                                                                                                    (crate::value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "value" => "cnt"
                                                                                                    ]))
                                                                                            ]))
                                                                                    ]))
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_if",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "op_logical_or",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    (crate::value!([
                                                                                        "emit" => "op_unary_not",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                "emit" => "identifier",
                                                                                                "value" => "cnt"
                                                                                            ]))
                                                                                    ])),
                                                                                    (crate::value!([
                                                                                        "emit" => "op_logical_and",
                                                                                        "children" =>
                                                                                            (crate::value!([
                                                                                                (crate::value!([
                                                                                                    "emit" => "op_logical_and",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            (crate::value!([
                                                                                                                "emit" => "op_logical_and",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "comparison",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "call",
                                                                                                                                        "children" =>
                                                                                                                                            (crate::value!([
                                                                                                                                                (crate::value!([
                                                                                                                                                    "emit" => "identifier",
                                                                                                                                                    "value" => "type"
                                                                                                                                                ])),
                                                                                                                                                (crate::value!([
                                                                                                                                                    "emit" => "callarg",
                                                                                                                                                    "children" =>
                                                                                                                                                        (crate::value!([
                                                                                                                                                            "emit" => "identifier",
                                                                                                                                                            "value" => "res"
                                                                                                                                                        ]))
                                                                                                                                                ]))
                                                                                                                                            ]))
                                                                                                                                    ])),
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "cmp_neq",
                                                                                                                                        "children" =>
                                                                                                                                            (crate::value!([
                                                                                                                                                "emit" => "value_string",
                                                                                                                                                "value" => "int"
                                                                                                                                            ]))
                                                                                                                                    ]))
                                                                                                                                ]))
                                                                                                                        ])),
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "comparison",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "call",
                                                                                                                                        "children" =>
                                                                                                                                            (crate::value!([
                                                                                                                                                (crate::value!([
                                                                                                                                                    "emit" => "identifier",
                                                                                                                                                    "value" => "type"
                                                                                                                                                ])),
                                                                                                                                                (crate::value!([
                                                                                                                                                    "emit" => "callarg",
                                                                                                                                                    "children" =>
                                                                                                                                                        (crate::value!([
                                                                                                                                                            "emit" => "identifier",
                                                                                                                                                            "value" => "res"
                                                                                                                                                        ]))
                                                                                                                                                ]))
                                                                                                                                            ]))
                                                                                                                                    ])),
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "cmp_neq",
                                                                                                                                        "children" =>
                                                                                                                                            (crate::value!([
                                                                                                                                                "emit" => "value_string",
                                                                                                                                                "value" => "float"
                                                                                                                                            ]))
                                                                                                                                    ]))
                                                                                                                                ]))
                                                                                                                        ]))
                                                                                                                    ]))
                                                                                                            ])),
                                                                                                            (crate::value!([
                                                                                                                "emit" => "comparison",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "call",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "identifier",
                                                                                                                                        "value" => "type"
                                                                                                                                    ])),
                                                                                                                                    (crate::value!([
                                                                                                                                        "emit" => "callarg",
                                                                                                                                        "children" =>
                                                                                                                                            (crate::value!([
                                                                                                                                                "emit" => "identifier",
                                                                                                                                                "value" => "res"
                                                                                                                                            ]))
                                                                                                                                    ]))
                                                                                                                                ]))
                                                                                                                        ])),
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "cmp_neq",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "value_string",
                                                                                                                                    "value" => "bool"
                                                                                                                                ]))
                                                                                                                        ]))
                                                                                                                    ]))
                                                                                                            ]))
                                                                                                        ]))
                                                                                                ])),
                                                                                                (crate::value!([
                                                                                                    "emit" => "comparison",
                                                                                                    "children" =>
                                                                                                        (crate::value!([
                                                                                                            (crate::value!([
                                                                                                                "emit" => "call",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "identifier",
                                                                                                                            "value" => "type"
                                                                                                                        ])),
                                                                                                                        (crate::value!([
                                                                                                                            "emit" => "callarg",
                                                                                                                            "children" =>
                                                                                                                                (crate::value!([
                                                                                                                                    "emit" => "identifier",
                                                                                                                                    "value" => "res"
                                                                                                                                ]))
                                                                                                                        ]))
                                                                                                                    ]))
                                                                                                            ])),
                                                                                                            (crate::value!([
                                                                                                                "emit" => "cmp_neq",
                                                                                                                "children" =>
                                                                                                                    (crate::value!([
                                                                                                                        "emit" => "value_string",
                                                                                                                        "value" => "null"
                                                                                                                    ]))
                                                                                                            ]))
                                                                                                        ]))
                                                                                                ]))
                                                                                            ]))
                                                                                    ]))
                                                                                ]))
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "op_accept",
                                                                            "children" =>
                                                                                (crate::value!([
                                                                                    "emit" => "value_void"
                                                                                ]))
                                                                        ]))
                                                                    ]))
                                                            ])),
                                                            (crate::value!([
                                                                "emit" => "op_binary_div",
                                                                "children" =>
                                                                    (crate::value!([
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "res"
                                                                        ])),
                                                                        (crate::value!([
                                                                            "emit" => "identifier",
                                                                            "value" => "cnt"
                                                                        ]))
                                                                    ]))
                                                            ]))
                                                        ]))
                                                ]))
                                            ]))
                                    ]))
                                ]))
                        ]))
                    ]))
            ])
            /*ETARENEG*/
        ;

        self.compile_from_ast(&ast, Some("prelude".to_string()))
            .expect("prelude cannot be compiled!")
            .expect("prelude contains no main?");
    }
}
