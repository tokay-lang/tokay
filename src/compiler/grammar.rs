use crate::RefValue;
use crate::value;

pub(super) fn tokay() -> RefValue {
    /*GENERATE cargo run -- "`sed 's/ast("main")/ast2rust(ast("main"))/g' ../examples/tokay.tok`" -- ../examples/tokay.tok */
    (value!([
        "emit" => "main",
        "row" => 1,
        "col" => 1,
        "children" => (
            value!([
                (value!([
                    "emit" => "constant",
                    "row" => 24,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 24,
                                "col" => 1,
                                "value" => "_"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 24,
                                "col" => 5,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 24,
                                        "col" => 6,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "op_mod_pos",
                                                    "row" => 25,
                                                    "col" => 5,
                                                    "children" => (
                                                        (value!([
                                                            "emit" => "value_token_ccl",
                                                            "row" => 25,
                                                            "col" => 5,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "ccl",
                                                                    "row" => 25,
                                                                    "col" => 6,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "char",
                                                                                "row" => 25,
                                                                                "col" => 6,
                                                                                "value" => "\t"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "char",
                                                                                "row" => 25,
                                                                                "col" => 8,
                                                                                "value" => " "
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ]))
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 26,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 26,
                                                                "col" => 5,
                                                                "value" => "#"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_kle",
                                                                "row" => 26,
                                                                "col" => 9,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_ccl",
                                                                        "row" => 26,
                                                                        "col" => 9,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "ccl_neg",
                                                                                "row" => 26,
                                                                                "col" => 10,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "char",
                                                                                        "row" => 26,
                                                                                        "col" => 11,
                                                                                        "value" => "\n"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 27,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 27,
                                                                "col" => 5,
                                                                "value" => "\\"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 27,
                                                                "col" => 10,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 27,
                                                                        "col" => 10,
                                                                        "value" => "\r"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 27,
                                                                "col" => 16,
                                                                "value" => "\n"
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 30,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 30,
                                "col" => 1,
                                "value" => "___"
                            ])),
                            (value!([
                                "emit" => "op_mod_kle",
                                "row" => 30,
                                "col" => 7,
                                "children" => (
                                    (value!([
                                        "emit" => "inline_sequence",
                                        "row" => 30,
                                        "col" => 8,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 30,
                                                    "col" => 8,
                                                    "value" => "T_EOL"
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 30,
                                                    "col" => 14,
                                                    "value" => "_"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 32,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 32,
                                "col" => 1,
                                "value" => "_SeparatedIdentifier"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 32,
                                "col" => 24,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 32,
                                        "col" => 25,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 33,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "op_mod_peek",
                                                            "row" => 33,
                                                            "col" => 5,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "op_mod_not",
                                                                    "row" => 33,
                                                                    "col" => 10,
                                                                    "children" => (
                                                                        (value!([
                                                                            "emit" => "value_token_ccl",
                                                                            "row" => 33,
                                                                            "col" => 14,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "ccl",
                                                                                    "row" => 33,
                                                                                    "col" => 15,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "range",
                                                                                                "row" => 33,
                                                                                                "col" => 15,
                                                                                                "value" => "AZ"
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "char",
                                                                                                "row" => 33,
                                                                                                "col" => 18,
                                                                                                "value" => "_"
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "range",
                                                                                                "row" => 33,
                                                                                                "col" => 19,
                                                                                                "value" => "az"
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 33,
                                                            "col" => 24,
                                                            "value" => "_"
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 36,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 36,
                                "col" => 1,
                                "value" => "T_EOL"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 36,
                                "col" => 9,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 36,
                                        "col" => 10,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 37,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 37,
                                                                "col" => 5,
                                                                "value" => "\n"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 37,
                                                                "col" => 10,
                                                                "value" => "_"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 38,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 38,
                                                                "col" => 5,
                                                                "value" => "\r"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 38,
                                                                "col" => 10,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 38,
                                                                        "col" => 10,
                                                                        "value" => "\n"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 38,
                                                                "col" => 16,
                                                                "value" => "_"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 39,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 39,
                                                                "col" => 5,
                                                                "value" => ";"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 39,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 40,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "op_mod_peek",
                                                                "row" => 40,
                                                                "col" => 5,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 40,
                                                                        "col" => 10,
                                                                        "value" => "EOF"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_accept",
                                                                "row" => 40,
                                                                "col" => 15,
                                                                "value" => "accept"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "op_mod_peek",
                                                    "row" => 41,
                                                    "col" => 5,
                                                    "children" => (
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 41,
                                                            "col" => 10,
                                                            "value" => "}"
                                                        ]))
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 46,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 46,
                                "col" => 1,
                                "value" => "T_OctDigit"
                            ])),
                            (value!([
                                "emit" => "value_token_ccl",
                                "row" => 46,
                                "col" => 14,
                                "children" => (
                                    (value!([
                                        "emit" => "ccl",
                                        "row" => 46,
                                        "col" => 15,
                                        "children" => (
                                            (value!([
                                                "emit" => "range",
                                                "row" => 46,
                                                "col" => 15,
                                                "value" => "07"
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 48,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 48,
                                "col" => 1,
                                "value" => "T_HexDigit"
                            ])),
                            (value!([
                                "emit" => "value_token_ccl",
                                "row" => 48,
                                "col" => 14,
                                "children" => (
                                    (value!([
                                        "emit" => "ccl",
                                        "row" => 48,
                                        "col" => 15,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "range",
                                                    "row" => 48,
                                                    "col" => 15,
                                                    "value" => "09"
                                                ])),
                                                (value!([
                                                    "emit" => "range",
                                                    "row" => 48,
                                                    "col" => 18,
                                                    "value" => "AF"
                                                ])),
                                                (value!([
                                                    "emit" => "range",
                                                    "row" => 48,
                                                    "col" => 21,
                                                    "value" => "af"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 50,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 50,
                                "col" => 1,
                                "value" => "T_EscapeSequence"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 50,
                                "col" => 20,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 50,
                                        "col" => 21,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 51,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 51,
                                                                "col" => 5,
                                                                "value" => "a"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 51,
                                                                "col" => 10,
                                                                "value" => "\u{7}"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 52,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 52,
                                                                "col" => 5,
                                                                "value" => "b"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 52,
                                                                "col" => 10,
                                                                "value" => "\u{8}"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 53,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 53,
                                                                "col" => 5,
                                                                "value" => "f"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 53,
                                                                "col" => 10,
                                                                "value" => "\u{c}"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 54,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 54,
                                                                "col" => 5,
                                                                "value" => "n"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 54,
                                                                "col" => 10,
                                                                "value" => "\n"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 55,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 55,
                                                                "col" => 5,
                                                                "value" => "r"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 55,
                                                                "col" => 10,
                                                                "value" => "\r"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 56,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 56,
                                                                "col" => 5,
                                                                "value" => "t"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 56,
                                                                "col" => 10,
                                                                "value" => "\t"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 57,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 57,
                                                                "col" => 5,
                                                                "value" => "v"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 57,
                                                                "col" => 10,
                                                                "value" => "\u{b}"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 62,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 62,
                                                                "col" => 5,
                                                                "value" => "T_OctDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 62,
                                                                "col" => 16,
                                                                "value" => "T_OctDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 62,
                                                                "col" => 27,
                                                                "value" => "T_OctDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 62,
                                                                "col" => 39,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 62,
                                                                            "col" => 39,
                                                                            "value" => "chr"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 62,
                                                                            "col" => 43,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "op_binary_add",
                                                                                    "row" => 62,
                                                                                    "col" => 43,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "op_binary_add",
                                                                                                "row" => 62,
                                                                                                "col" => 43,
                                                                                                "children" => (
                                                                                                    value!([
                                                                                                        (value!([
                                                                                                            "emit" => "op_binary_mul",
                                                                                                            "row" => 62,
                                                                                                            "col" => 43,
                                                                                                            "children" => (
                                                                                                                value!([
                                                                                                                    (value!([
                                                                                                                        "emit" => "op_binary_mul",
                                                                                                                        "row" => 62,
                                                                                                                        "col" => 43,
                                                                                                                        "children" => (
                                                                                                                            value!([
                                                                                                                                (value!([
                                                                                                                                    "emit" => "call",
                                                                                                                                    "row" => 62,
                                                                                                                                    "col" => 43,
                                                                                                                                    "children" => (
                                                                                                                                        value!([
                                                                                                                                            (value!([
                                                                                                                                                "emit" => "identifier",
                                                                                                                                                "row" => 62,
                                                                                                                                                "col" => 43,
                                                                                                                                                "value" => "int"
                                                                                                                                            ])),
                                                                                                                                            (value!([
                                                                                                                                                "emit" => "callarg",
                                                                                                                                                "row" => 62,
                                                                                                                                                "col" => 47,
                                                                                                                                                "children" => (
                                                                                                                                                    (value!([
                                                                                                                                                        "emit" => "capture_index",
                                                                                                                                                        "row" => 62,
                                                                                                                                                        "col" => 47,
                                                                                                                                                        "children" => (
                                                                                                                                                            (value!([
                                                                                                                                                                "emit" => "value_integer",
                                                                                                                                                                "row" => 62,
                                                                                                                                                                "col" => 48,
                                                                                                                                                                "value" => "1"
                                                                                                                                                            ]))
                                                                                                                                                        )
                                                                                                                                                    ]))
                                                                                                                                                )
                                                                                                                                            ]))
                                                                                                                                        ])
                                                                                                                                    )
                                                                                                                                ])),
                                                                                                                                (value!([
                                                                                                                                    "emit" => "value_integer",
                                                                                                                                    "row" => 62,
                                                                                                                                    "col" => 53,
                                                                                                                                    "value" => "8"
                                                                                                                                ]))
                                                                                                                            ])
                                                                                                                        )
                                                                                                                    ])),
                                                                                                                    (value!([
                                                                                                                        "emit" => "value_integer",
                                                                                                                        "row" => 62,
                                                                                                                        "col" => 57,
                                                                                                                        "value" => "8"
                                                                                                                    ]))
                                                                                                                ])
                                                                                                            )
                                                                                                        ])),
                                                                                                        (value!([
                                                                                                            "emit" => "op_binary_mul",
                                                                                                            "row" => 62,
                                                                                                            "col" => 61,
                                                                                                            "children" => (
                                                                                                                value!([
                                                                                                                    (value!([
                                                                                                                        "emit" => "call",
                                                                                                                        "row" => 62,
                                                                                                                        "col" => 61,
                                                                                                                        "children" => (
                                                                                                                            value!([
                                                                                                                                (value!([
                                                                                                                                    "emit" => "identifier",
                                                                                                                                    "row" => 62,
                                                                                                                                    "col" => 61,
                                                                                                                                    "value" => "int"
                                                                                                                                ])),
                                                                                                                                (value!([
                                                                                                                                    "emit" => "callarg",
                                                                                                                                    "row" => 62,
                                                                                                                                    "col" => 65,
                                                                                                                                    "children" => (
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "capture_index",
                                                                                                                                            "row" => 62,
                                                                                                                                            "col" => 65,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_integer",
                                                                                                                                                    "row" => 62,
                                                                                                                                                    "col" => 66,
                                                                                                                                                    "value" => "2"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ]))
                                                                                                                                    )
                                                                                                                                ]))
                                                                                                                            ])
                                                                                                                        )
                                                                                                                    ])),
                                                                                                                    (value!([
                                                                                                                        "emit" => "value_integer",
                                                                                                                        "row" => 62,
                                                                                                                        "col" => 71,
                                                                                                                        "value" => "8"
                                                                                                                    ]))
                                                                                                                ])
                                                                                                            )
                                                                                                        ]))
                                                                                                    ])
                                                                                                )
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "call",
                                                                                                "row" => 62,
                                                                                                "col" => 75,
                                                                                                "children" => (
                                                                                                    value!([
                                                                                                        (value!([
                                                                                                            "emit" => "identifier",
                                                                                                            "row" => 62,
                                                                                                            "col" => 75,
                                                                                                            "value" => "int"
                                                                                                        ])),
                                                                                                        (value!([
                                                                                                            "emit" => "callarg",
                                                                                                            "row" => 62,
                                                                                                            "col" => 79,
                                                                                                            "children" => (
                                                                                                                (value!([
                                                                                                                    "emit" => "capture_index",
                                                                                                                    "row" => 62,
                                                                                                                    "col" => 79,
                                                                                                                    "children" => (
                                                                                                                        (value!([
                                                                                                                            "emit" => "value_integer",
                                                                                                                            "row" => 62,
                                                                                                                            "col" => 80,
                                                                                                                            "value" => "3"
                                                                                                                        ]))
                                                                                                                    )
                                                                                                                ]))
                                                                                                            )
                                                                                                        ]))
                                                                                                    ])
                                                                                                )
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 63,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 63,
                                                                "col" => 5,
                                                                "value" => "x"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 63,
                                                                "col" => 9,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 63,
                                                                "col" => 20,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 63,
                                                                "col" => 31,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 63,
                                                                            "col" => 31,
                                                                            "value" => "chr"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 63,
                                                                            "col" => 35,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "call",
                                                                                    "row" => 63,
                                                                                    "col" => 35,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 63,
                                                                                                "col" => 35,
                                                                                                "value" => "int"
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "callarg",
                                                                                                "row" => 63,
                                                                                                "col" => 39,
                                                                                                "children" => (
                                                                                                    (value!([
                                                                                                        "emit" => "op_binary_add",
                                                                                                        "row" => 63,
                                                                                                        "col" => 39,
                                                                                                        "children" => (
                                                                                                            value!([
                                                                                                                (value!([
                                                                                                                    "emit" => "value_string",
                                                                                                                    "row" => 63,
                                                                                                                    "col" => 39,
                                                                                                                    "value" => "0x"
                                                                                                                ])),
                                                                                                                (value!([
                                                                                                                    "emit" => "call",
                                                                                                                    "row" => 63,
                                                                                                                    "col" => 46,
                                                                                                                    "children" => (
                                                                                                                        value!([
                                                                                                                            (value!([
                                                                                                                                "emit" => "rvalue",
                                                                                                                                "row" => 63,
                                                                                                                                "col" => 46,
                                                                                                                                "children" => (
                                                                                                                                    value!([
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "capture_index",
                                                                                                                                            "row" => 63,
                                                                                                                                            "col" => 46,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_integer",
                                                                                                                                                    "row" => 63,
                                                                                                                                                    "col" => 47,
                                                                                                                                                    "value" => "0"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ])),
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "attribute",
                                                                                                                                            "row" => 63,
                                                                                                                                            "col" => 48,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_string",
                                                                                                                                                    "row" => 63,
                                                                                                                                                    "col" => 49,
                                                                                                                                                    "value" => "substr"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ]))
                                                                                                                                    ])
                                                                                                                                )
                                                                                                                            ])),
                                                                                                                            (value!([
                                                                                                                                "emit" => "callarg",
                                                                                                                                "row" => 63,
                                                                                                                                "col" => 56,
                                                                                                                                "children" => (
                                                                                                                                    (value!([
                                                                                                                                        "emit" => "value_integer",
                                                                                                                                        "row" => 63,
                                                                                                                                        "col" => 56,
                                                                                                                                        "value" => "1"
                                                                                                                                    ]))
                                                                                                                                )
                                                                                                                            ]))
                                                                                                                        ])
                                                                                                                    )
                                                                                                                ]))
                                                                                                            ])
                                                                                                        )
                                                                                                    ]))
                                                                                                )
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 64,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 64,
                                                                "col" => 5,
                                                                "value" => "u"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 64,
                                                                "col" => 9,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 64,
                                                                "col" => 20,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 64,
                                                                "col" => 31,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 64,
                                                                "col" => 42,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 64,
                                                                "col" => 54,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 64,
                                                                            "col" => 54,
                                                                            "value" => "chr"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 64,
                                                                            "col" => 58,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "call",
                                                                                    "row" => 64,
                                                                                    "col" => 58,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 64,
                                                                                                "col" => 58,
                                                                                                "value" => "int"
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "callarg",
                                                                                                "row" => 64,
                                                                                                "col" => 62,
                                                                                                "children" => (
                                                                                                    (value!([
                                                                                                        "emit" => "op_binary_add",
                                                                                                        "row" => 64,
                                                                                                        "col" => 62,
                                                                                                        "children" => (
                                                                                                            value!([
                                                                                                                (value!([
                                                                                                                    "emit" => "value_string",
                                                                                                                    "row" => 64,
                                                                                                                    "col" => 62,
                                                                                                                    "value" => "0x"
                                                                                                                ])),
                                                                                                                (value!([
                                                                                                                    "emit" => "call",
                                                                                                                    "row" => 64,
                                                                                                                    "col" => 69,
                                                                                                                    "children" => (
                                                                                                                        value!([
                                                                                                                            (value!([
                                                                                                                                "emit" => "rvalue",
                                                                                                                                "row" => 64,
                                                                                                                                "col" => 69,
                                                                                                                                "children" => (
                                                                                                                                    value!([
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "capture_index",
                                                                                                                                            "row" => 64,
                                                                                                                                            "col" => 69,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_integer",
                                                                                                                                                    "row" => 64,
                                                                                                                                                    "col" => 70,
                                                                                                                                                    "value" => "0"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ])),
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "attribute",
                                                                                                                                            "row" => 64,
                                                                                                                                            "col" => 71,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_string",
                                                                                                                                                    "row" => 64,
                                                                                                                                                    "col" => 72,
                                                                                                                                                    "value" => "substr"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ]))
                                                                                                                                    ])
                                                                                                                                )
                                                                                                                            ])),
                                                                                                                            (value!([
                                                                                                                                "emit" => "callarg",
                                                                                                                                "row" => 64,
                                                                                                                                "col" => 79,
                                                                                                                                "children" => (
                                                                                                                                    (value!([
                                                                                                                                        "emit" => "value_integer",
                                                                                                                                        "row" => 64,
                                                                                                                                        "col" => 79,
                                                                                                                                        "value" => "1"
                                                                                                                                    ]))
                                                                                                                                )
                                                                                                                            ]))
                                                                                                                        ])
                                                                                                                    )
                                                                                                                ]))
                                                                                                            ])
                                                                                                        )
                                                                                                    ]))
                                                                                                )
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 65,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 65,
                                                                "col" => 5,
                                                                "value" => "U"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 65,
                                                                "col" => 9,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 65,
                                                                "col" => 20,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 65,
                                                                "col" => 31,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 65,
                                                                "col" => 42,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 66,
                                                                "col" => 9,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 66,
                                                                "col" => 20,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 66,
                                                                "col" => 31,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 66,
                                                                "col" => 42,
                                                                "value" => "T_HexDigit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 67,
                                                                "col" => 9,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 67,
                                                                            "col" => 9,
                                                                            "value" => "chr"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 67,
                                                                            "col" => 13,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "call",
                                                                                    "row" => 67,
                                                                                    "col" => 13,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 67,
                                                                                                "col" => 13,
                                                                                                "value" => "int"
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "callarg",
                                                                                                "row" => 67,
                                                                                                "col" => 17,
                                                                                                "children" => (
                                                                                                    (value!([
                                                                                                        "emit" => "op_binary_add",
                                                                                                        "row" => 67,
                                                                                                        "col" => 17,
                                                                                                        "children" => (
                                                                                                            value!([
                                                                                                                (value!([
                                                                                                                    "emit" => "value_string",
                                                                                                                    "row" => 67,
                                                                                                                    "col" => 17,
                                                                                                                    "value" => "0x"
                                                                                                                ])),
                                                                                                                (value!([
                                                                                                                    "emit" => "call",
                                                                                                                    "row" => 67,
                                                                                                                    "col" => 24,
                                                                                                                    "children" => (
                                                                                                                        value!([
                                                                                                                            (value!([
                                                                                                                                "emit" => "rvalue",
                                                                                                                                "row" => 67,
                                                                                                                                "col" => 24,
                                                                                                                                "children" => (
                                                                                                                                    value!([
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "capture_index",
                                                                                                                                            "row" => 67,
                                                                                                                                            "col" => 24,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_integer",
                                                                                                                                                    "row" => 67,
                                                                                                                                                    "col" => 25,
                                                                                                                                                    "value" => "0"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ])),
                                                                                                                                        (value!([
                                                                                                                                            "emit" => "attribute",
                                                                                                                                            "row" => 67,
                                                                                                                                            "col" => 26,
                                                                                                                                            "children" => (
                                                                                                                                                (value!([
                                                                                                                                                    "emit" => "value_string",
                                                                                                                                                    "row" => 67,
                                                                                                                                                    "col" => 27,
                                                                                                                                                    "value" => "substr"
                                                                                                                                                ]))
                                                                                                                                            )
                                                                                                                                        ]))
                                                                                                                                    ])
                                                                                                                                )
                                                                                                                            ])),
                                                                                                                            (value!([
                                                                                                                                "emit" => "callarg",
                                                                                                                                "row" => 67,
                                                                                                                                "col" => 34,
                                                                                                                                "children" => (
                                                                                                                                    (value!([
                                                                                                                                        "emit" => "value_integer",
                                                                                                                                        "row" => 67,
                                                                                                                                        "col" => 34,
                                                                                                                                        "value" => "1"
                                                                                                                                    ]))
                                                                                                                                )
                                                                                                                            ]))
                                                                                                                        ])
                                                                                                                    )
                                                                                                                ]))
                                                                                                            ])
                                                                                                        )
                                                                                                    ]))
                                                                                                )
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 69,
                                                    "col" => 5,
                                                    "value" => "Any"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 72,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 72,
                                "col" => 1,
                                "value" => "T_Identifier"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 72,
                                "col" => 16,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 72,
                                        "col" => 17,
                                        "children" => (
                                            (value!([
                                                "emit" => "call",
                                                "row" => 73,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 73,
                                                            "col" => 5,
                                                            "value" => "ast"
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 73,
                                                            "col" => 9,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_string",
                                                                    "row" => 73,
                                                                    "col" => 9,
                                                                    "value" => "identifier"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 73,
                                                            "col" => 23,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 73,
                                                                    "col" => 23,
                                                                    "value" => "Ident"
                                                                ]))
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 76,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 76,
                                "col" => 1,
                                "value" => "T_Consumable"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 76,
                                "col" => 16,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 76,
                                        "col" => 17,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 77,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_ccl",
                                                            "row" => 77,
                                                            "col" => 5,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "ccl",
                                                                    "row" => 77,
                                                                    "col" => 6,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "range",
                                                                                "row" => 77,
                                                                                "col" => 6,
                                                                                "value" => "AZ"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "char",
                                                                                "row" => 77,
                                                                                "col" => 9,
                                                                                "value" => "_"
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 77,
                                                            "col" => 12,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_token_ccl",
                                                                    "row" => 77,
                                                                    "col" => 12,
                                                                    "children" => (
                                                                        (value!([
                                                                            "emit" => "ccl",
                                                                            "row" => 77,
                                                                            "col" => 13,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 77,
                                                                                        "col" => 13,
                                                                                        "value" => "09"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 77,
                                                                                        "col" => 16,
                                                                                        "value" => "AZ"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "char",
                                                                                        "row" => 77,
                                                                                        "col" => 19,
                                                                                        "value" => "_"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 77,
                                                                                        "col" => 20,
                                                                                        "value" => "az"
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ]))
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 77,
                                                            "col" => 27,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 77,
                                                                        "col" => 27,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 77,
                                                                        "col" => 31,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 77,
                                                                                "col" => 31,
                                                                                "value" => "identifier"
                                                                            ]))
                                                                        )
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 77,
                                                                        "col" => 45,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "capture_index",
                                                                                "row" => 77,
                                                                                "col" => 45,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "value_integer",
                                                                                        "row" => 77,
                                                                                        "col" => 46,
                                                                                        "value" => "0"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 80,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 80,
                                "col" => 1,
                                "value" => "T_Alias"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 80,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 80,
                                        "col" => 12,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 81,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_ccl",
                                                            "row" => 81,
                                                            "col" => 5,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "ccl",
                                                                    "row" => 81,
                                                                    "col" => 6,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "range",
                                                                                "row" => 81,
                                                                                "col" => 6,
                                                                                "value" => "AZ"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "char",
                                                                                "row" => 81,
                                                                                "col" => 9,
                                                                                "value" => "_"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "range",
                                                                                "row" => 81,
                                                                                "col" => 10,
                                                                                "value" => "az"
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 81,
                                                            "col" => 15,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_token_ccl",
                                                                    "row" => 81,
                                                                    "col" => 15,
                                                                    "children" => (
                                                                        (value!([
                                                                            "emit" => "ccl",
                                                                            "row" => 81,
                                                                            "col" => 16,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 81,
                                                                                        "col" => 16,
                                                                                        "value" => "09"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 81,
                                                                                        "col" => 19,
                                                                                        "value" => "AZ"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "char",
                                                                                        "row" => 81,
                                                                                        "col" => 22,
                                                                                        "value" => "_"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "range",
                                                                                        "row" => 81,
                                                                                        "col" => 23,
                                                                                        "value" => "az"
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ]))
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 81,
                                                            "col" => 30,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 81,
                                                                        "col" => 30,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 81,
                                                                        "col" => 34,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 81,
                                                                                "col" => 34,
                                                                                "value" => "value_string"
                                                                            ]))
                                                                        )
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 81,
                                                                        "col" => 50,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "capture_index",
                                                                                "row" => 81,
                                                                                "col" => 50,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "value_integer",
                                                                                        "row" => 81,
                                                                                        "col" => 51,
                                                                                        "value" => "0"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 84,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 84,
                                "col" => 1,
                                "value" => "T_String"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 84,
                                "col" => 12,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 84,
                                        "col" => 13,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 85,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 85,
                                                            "col" => 5,
                                                            "value" => "\""
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 85,
                                                            "col" => 9,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "block",
                                                                    "row" => 85,
                                                                    "col" => 9,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 86,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 86,
                                                                                            "col" => 9,
                                                                                            "value" => "\\"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 86,
                                                                                            "col" => 14,
                                                                                            "value" => "T_EscapeSequence"
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "value_token_ccl",
                                                                                "row" => 87,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "ccl_neg",
                                                                                        "row" => 87,
                                                                                        "col" => 10,
                                                                                        "children" => (
                                                                                            value!([
                                                                                                (value!([
                                                                                                    "emit" => "char",
                                                                                                    "row" => 87,
                                                                                                    "col" => 11,
                                                                                                    "value" => "\\"
                                                                                                ])),
                                                                                                (value!([
                                                                                                    "emit" => "char",
                                                                                                    "row" => 87,
                                                                                                    "col" => 13,
                                                                                                    "value" => "\""
                                                                                                ]))
                                                                                            ])
                                                                                        )
                                                                                    ]))
                                                                                )
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 88,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 88,
                                                                                            "col" => 9,
                                                                                            "value" => "EOF"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "call",
                                                                                            "row" => 88,
                                                                                            "col" => 14,
                                                                                            "children" => (
                                                                                                value!([
                                                                                                    (value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "row" => 88,
                                                                                                        "col" => 14,
                                                                                                        "value" => "error"
                                                                                                    ])),
                                                                                                    (value!([
                                                                                                        "emit" => "callarg",
                                                                                                        "row" => 88,
                                                                                                        "col" => 20,
                                                                                                        "children" => (
                                                                                                            (value!([
                                                                                                                "emit" => "value_string",
                                                                                                                "row" => 88,
                                                                                                                "col" => 20,
                                                                                                                "value" => "Unclosed string, expecting '\"'"
                                                                                                            ]))
                                                                                                        )
                                                                                                    ]))
                                                                                                ])
                                                                                            )
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 89,
                                                            "col" => 9,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 89,
                                                                        "col" => 9,
                                                                        "value" => "str_join"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 89,
                                                                        "col" => 18,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 89,
                                                                                "col" => 18,
                                                                                "value" => ""
                                                                            ]))
                                                                        )
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 89,
                                                                        "col" => 22,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "capture_index",
                                                                                "row" => 89,
                                                                                "col" => 22,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "value_integer",
                                                                                        "row" => 89,
                                                                                        "col" => 23,
                                                                                        "value" => "2"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_expect",
                                                            "row" => 89,
                                                            "col" => 26,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_token_touch",
                                                                    "row" => 89,
                                                                    "col" => 33,
                                                                    "value" => "\""
                                                                ]))
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 92,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 92,
                                "col" => 1,
                                "value" => "T_Touch"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 92,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 92,
                                        "col" => 12,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 93,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 93,
                                                            "col" => 5,
                                                            "value" => "'"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 93,
                                                            "col" => 10,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "block",
                                                                    "row" => 93,
                                                                    "col" => 10,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 94,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 94,
                                                                                            "col" => 9,
                                                                                            "value" => "\\"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 94,
                                                                                            "col" => 14,
                                                                                            "value" => "T_EscapeSequence"
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "value_token_ccl",
                                                                                "row" => 95,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "ccl_neg",
                                                                                        "row" => 95,
                                                                                        "col" => 10,
                                                                                        "children" => (
                                                                                            value!([
                                                                                                (value!([
                                                                                                    "emit" => "char",
                                                                                                    "row" => 95,
                                                                                                    "col" => 11,
                                                                                                    "value" => "\\"
                                                                                                ])),
                                                                                                (value!([
                                                                                                    "emit" => "char",
                                                                                                    "row" => 95,
                                                                                                    "col" => 13,
                                                                                                    "value" => "'"
                                                                                                ]))
                                                                                            ])
                                                                                        )
                                                                                    ]))
                                                                                )
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 96,
                                                                                "col" => 9,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 96,
                                                                                            "col" => 9,
                                                                                            "value" => "EOF"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "call",
                                                                                            "row" => 96,
                                                                                            "col" => 14,
                                                                                            "children" => (
                                                                                                value!([
                                                                                                    (value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "row" => 96,
                                                                                                        "col" => 14,
                                                                                                        "value" => "error"
                                                                                                    ])),
                                                                                                    (value!([
                                                                                                        "emit" => "callarg",
                                                                                                        "row" => 96,
                                                                                                        "col" => 20,
                                                                                                        "children" => (
                                                                                                            (value!([
                                                                                                                "emit" => "value_string",
                                                                                                                "row" => 96,
                                                                                                                "col" => 20,
                                                                                                                "value" => "Unclosed match, expecting '''"
                                                                                                            ]))
                                                                                                        )
                                                                                                    ]))
                                                                                                ])
                                                                                            )
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 97,
                                                            "col" => 9,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 97,
                                                                        "col" => 9,
                                                                        "value" => "str_join"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 97,
                                                                        "col" => 18,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 97,
                                                                                "col" => 18,
                                                                                "value" => ""
                                                                            ]))
                                                                        )
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 97,
                                                                        "col" => 22,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "capture_index",
                                                                                "row" => 97,
                                                                                "col" => 22,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "value_integer",
                                                                                        "row" => 97,
                                                                                        "col" => 23,
                                                                                        "value" => "2"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_expect",
                                                            "row" => 97,
                                                            "col" => 26,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_token_touch",
                                                                    "row" => 97,
                                                                    "col" => 33,
                                                                    "value" => "'"
                                                                ]))
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 100,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 100,
                                "col" => 1,
                                "value" => "T_Integer"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 100,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 100,
                                        "col" => 14,
                                        "children" => (
                                            (value!([
                                                "emit" => "call",
                                                "row" => 101,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 101,
                                                            "col" => 5,
                                                            "value" => "ast"
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 101,
                                                            "col" => 9,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_string",
                                                                    "row" => 101,
                                                                    "col" => 9,
                                                                    "value" => "value_integer"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 101,
                                                            "col" => 26,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 101,
                                                                    "col" => 26,
                                                                    "value" => "Int"
                                                                ]))
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 104,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 104,
                                "col" => 1,
                                "value" => "T_Float"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 104,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 104,
                                        "col" => 12,
                                        "children" => (
                                            (value!([
                                                "emit" => "call",
                                                "row" => 105,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 105,
                                                            "col" => 5,
                                                            "value" => "ast"
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 105,
                                                            "col" => 9,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_string",
                                                                    "row" => 105,
                                                                    "col" => 9,
                                                                    "value" => "value_float"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "callarg",
                                                            "row" => 105,
                                                            "col" => 24,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 105,
                                                                    "col" => 24,
                                                                    "value" => "Float"
                                                                ]))
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 110,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 110,
                                "col" => 1,
                                "value" => "CclChar"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 110,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 110,
                                        "col" => 12,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 111,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 111,
                                                                "col" => 5,
                                                                "value" => "\\"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 111,
                                                                "col" => 10,
                                                                "value" => "T_EscapeSequence"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "value_token_ccl",
                                                    "row" => 112,
                                                    "col" => 5,
                                                    "children" => (
                                                        (value!([
                                                            "emit" => "ccl_neg",
                                                            "row" => 112,
                                                            "col" => 6,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "char",
                                                                    "row" => 112,
                                                                    "col" => 7,
                                                                    "value" => "]"
                                                                ]))
                                                            )
                                                        ]))
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 113,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 113,
                                                                "col" => 5,
                                                                "value" => "EOF"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 113,
                                                                "col" => 10,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 113,
                                                                            "col" => 10,
                                                                            "value" => "error"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 113,
                                                                            "col" => 16,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 113,
                                                                                    "col" => 16,
                                                                                    "value" => "Unclosed character-class, expecting ']'"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 116,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 116,
                                "col" => 1,
                                "value" => "CclRange"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 116,
                                "col" => 12,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 116,
                                        "col" => 13,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 117,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 117,
                                                                "col" => 5,
                                                                "value" => "CclChar"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 117,
                                                                "col" => 13,
                                                                "value" => "-"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 117,
                                                                "col" => 17,
                                                                "value" => "CclChar"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 117,
                                                                "col" => 26,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 117,
                                                                            "col" => 26,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 117,
                                                                            "col" => 30,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 117,
                                                                                    "col" => 30,
                                                                                    "value" => "range"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 117,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "op_binary_add",
                                                                                    "row" => 117,
                                                                                    "col" => 39,
                                                                                    "children" => (
                                                                                        value!([
                                                                                            (value!([
                                                                                                "emit" => "capture_index",
                                                                                                "row" => 117,
                                                                                                "col" => 39,
                                                                                                "children" => (
                                                                                                    (value!([
                                                                                                        "emit" => "value_integer",
                                                                                                        "row" => 117,
                                                                                                        "col" => 40,
                                                                                                        "value" => "1"
                                                                                                    ]))
                                                                                                )
                                                                                            ])),
                                                                                            (value!([
                                                                                                "emit" => "capture_index",
                                                                                                "row" => 117,
                                                                                                "col" => 44,
                                                                                                "children" => (
                                                                                                    (value!([
                                                                                                        "emit" => "value_integer",
                                                                                                        "row" => 117,
                                                                                                        "col" => 45,
                                                                                                        "value" => "3"
                                                                                                    ]))
                                                                                                )
                                                                                            ]))
                                                                                        ])
                                                                                    )
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 118,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 118,
                                                                "col" => 5,
                                                                "value" => "CclChar"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 118,
                                                                "col" => 14,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 118,
                                                                            "col" => 14,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 118,
                                                                            "col" => 18,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 118,
                                                                                    "col" => 18,
                                                                                    "value" => "char"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 121,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 121,
                                "col" => 1,
                                "value" => "Ccl"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 121,
                                "col" => 7,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 121,
                                        "col" => 8,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 122,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 122,
                                                                "col" => 5,
                                                                "value" => "^"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_kle",
                                                                "row" => 122,
                                                                "col" => 9,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 122,
                                                                        "col" => 9,
                                                                        "value" => "CclRange"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 122,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 122,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 122,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 122,
                                                                                    "col" => 24,
                                                                                    "value" => "ccl_neg"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 123,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "op_mod_kle",
                                                                "row" => 123,
                                                                "col" => 5,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 123,
                                                                        "col" => 5,
                                                                        "value" => "CclRange"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 123,
                                                                "col" => 16,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 123,
                                                                            "col" => 16,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 123,
                                                                            "col" => 20,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 123,
                                                                                    "col" => 20,
                                                                                    "value" => "ccl"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 128,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 128,
                                "col" => 1,
                                "value" => "Subscript"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 128,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 128,
                                        "col" => 14,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 129,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 129,
                                                            "col" => 5,
                                                            "value" => "["
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 129,
                                                            "col" => 9,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 129,
                                                            "col" => 11,
                                                            "value" => "Expression"
                                                        ])),
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 129,
                                                            "col" => 22,
                                                            "value" => "]"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 129,
                                                            "col" => 26,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 129,
                                                            "col" => 29,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 129,
                                                                        "col" => 29,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 129,
                                                                        "col" => 33,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 129,
                                                                                "col" => 33,
                                                                                "value" => "item"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 132,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 132,
                                "col" => 1,
                                "value" => "Attribute"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 132,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 132,
                                        "col" => 14,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 133,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 133,
                                                            "col" => 5,
                                                            "value" => "."
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 133,
                                                            "col" => 9,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 133,
                                                            "col" => 11,
                                                            "value" => "T_Alias"
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 133,
                                                            "col" => 20,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 133,
                                                                        "col" => 20,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 133,
                                                                        "col" => 24,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 133,
                                                                                "col" => 24,
                                                                                "value" => "attribute"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 136,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 136,
                                "col" => 1,
                                "value" => "Capture"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 136,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 136,
                                        "col" => 12,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 137,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 137,
                                                                "col" => 5,
                                                                "value" => "$"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 137,
                                                                "col" => 9,
                                                                "value" => "T_Alias"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 137,
                                                                "col" => 17,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 137,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 137,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 137,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 137,
                                                                                    "col" => 24,
                                                                                    "value" => "capture_alias"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 138,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 138,
                                                                "col" => 5,
                                                                "value" => "$"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 138,
                                                                "col" => 9,
                                                                "value" => "T_Integer"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 138,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 138,
                                                                "col" => 22,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 138,
                                                                            "col" => 22,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 138,
                                                                            "col" => 26,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 138,
                                                                                    "col" => 26,
                                                                                    "value" => "capture_index"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 139,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 139,
                                                                "col" => 5,
                                                                "value" => "$"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 139,
                                                                "col" => 9,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 139,
                                                                "col" => 13,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 139,
                                                                "col" => 15,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 139,
                                                                "col" => 19,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 139,
                                                                "col" => 30,
                                                                "value" => ")"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 139,
                                                                "col" => 34,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 139,
                                                                "col" => 37,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 139,
                                                                            "col" => 37,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 139,
                                                                            "col" => 41,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 139,
                                                                                    "col" => 41,
                                                                                    "value" => "capture_expr"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 140,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 140,
                                                                "col" => 5,
                                                                "value" => "$"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 140,
                                                                "col" => 10,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 140,
                                                                            "col" => 10,
                                                                            "value" => "error"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 140,
                                                                            "col" => 16,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 140,
                                                                                    "col" => 16,
                                                                                    "value" => "'$...': Expecting identifier, integer or (expression)"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 143,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 143,
                                "col" => 1,
                                "value" => "Variable"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 143,
                                "col" => 12,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 143,
                                        "col" => 13,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 144,
                                                    "col" => 5,
                                                    "value" => "T_Identifier"
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 145,
                                                    "col" => 5,
                                                    "value" => "Capture"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 148,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 148,
                                "col" => 1,
                                "value" => "Lvalue"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 148,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 148,
                                        "col" => 11,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 149,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 149,
                                                            "col" => 5,
                                                            "value" => "Variable"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 149,
                                                            "col" => 14,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 149,
                                                            "col" => 16,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 149,
                                                                    "col" => 16,
                                                                    "value" => "Subscript"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 149,
                                                            "col" => 27,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 149,
                                                                        "col" => 27,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 149,
                                                                        "col" => 31,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 149,
                                                                                "col" => 31,
                                                                                "value" => "lvalue"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 152,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 152,
                                "col" => 1,
                                "value" => "Load"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 152,
                                "col" => 8,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 152,
                                        "col" => 9,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 153,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 153,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 153,
                                                                "col" => 12,
                                                                "value" => "++"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 153,
                                                                "col" => 18,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 153,
                                                                            "col" => 18,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 153,
                                                                            "col" => 22,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 153,
                                                                                    "col" => 22,
                                                                                    "value" => "inplace_post_inc"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 154,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 154,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 154,
                                                                "col" => 12,
                                                                "value" => "--"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 154,
                                                                "col" => 18,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 154,
                                                                            "col" => 18,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 154,
                                                                            "col" => 22,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 154,
                                                                                    "col" => 22,
                                                                                    "value" => "inplace_post_dec"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 155,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 155,
                                                                "col" => 5,
                                                                "value" => "++"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 155,
                                                                "col" => 10,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 155,
                                                                        "col" => 17,
                                                                        "value" => "Lvalue"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 155,
                                                                "col" => 25,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 155,
                                                                            "col" => 25,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 155,
                                                                            "col" => 29,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 155,
                                                                                    "col" => 29,
                                                                                    "value" => "inplace_pre_inc"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 156,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 156,
                                                                "col" => 5,
                                                                "value" => "--"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 156,
                                                                "col" => 10,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 156,
                                                                        "col" => 17,
                                                                        "value" => "Lvalue"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 156,
                                                                "col" => 25,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 156,
                                                                            "col" => 25,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 156,
                                                                            "col" => 29,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 156,
                                                                                    "col" => 29,
                                                                                    "value" => "inplace_pre_dec"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 157,
                                                    "col" => 5,
                                                    "value" => "Variable"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 162,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 162,
                                "col" => 1,
                                "value" => "Parselet"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 162,
                                "col" => 12,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 162,
                                        "col" => 13,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 163,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 163,
                                                            "col" => 5,
                                                            "value" => "@"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 163,
                                                            "col" => 9,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_opt",
                                                            "row" => 163,
                                                            "col" => 11,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 163,
                                                                    "col" => 11,
                                                                    "value" => "ParseletGenerics"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 163,
                                                            "col" => 29,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_opt",
                                                            "row" => 163,
                                                            "col" => 31,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 163,
                                                                    "col" => 31,
                                                                    "value" => "ParseletArguments"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_expect",
                                                            "row" => 163,
                                                            "col" => 50,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 163,
                                                                    "col" => 57,
                                                                    "value" => "Block"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 163,
                                                            "col" => 64,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 163,
                                                                        "col" => 64,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 163,
                                                                        "col" => 68,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 163,
                                                                                "col" => 68,
                                                                                "value" => "value_parselet"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 168,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 168,
                                "col" => 1,
                                "value" => "ParseletGeneric"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 168,
                                "col" => 19,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 168,
                                        "col" => 20,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 169,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 169,
                                                            "col" => 5,
                                                            "value" => "T_Identifier"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 169,
                                                            "col" => 18,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_opt",
                                                            "row" => 169,
                                                            "col" => 20,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "inline_sequence",
                                                                    "row" => 169,
                                                                    "col" => 21,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "value_token_touch",
                                                                                "row" => 169,
                                                                                "col" => 21,
                                                                                "value" => ":"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "identifier",
                                                                                "row" => 169,
                                                                                "col" => 25,
                                                                                "value" => "_"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "op_mod_expect",
                                                                                "row" => 169,
                                                                                "col" => 27,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 169,
                                                                                        "col" => 34,
                                                                                        "value" => "Atomic"
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 169,
                                                            "col" => 44,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 169,
                                                                        "col" => 44,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 169,
                                                                        "col" => 48,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 169,
                                                                                "col" => 48,
                                                                                "value" => "gen"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 172,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 172,
                                "col" => 1,
                                "value" => "ParseletGenerics"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 172,
                                "col" => 20,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 172,
                                        "col" => 21,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 173,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "value_token_touch",
                                                            "row" => 173,
                                                            "col" => 5,
                                                            "value" => "<"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 173,
                                                            "col" => 9,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_kle",
                                                            "row" => 173,
                                                            "col" => 11,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "inline_sequence",
                                                                    "row" => 173,
                                                                    "col" => 12,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "identifier",
                                                                                "row" => 173,
                                                                                "col" => 12,
                                                                                "value" => "ParseletGeneric"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "op_mod_opt",
                                                                                "row" => 173,
                                                                                "col" => 28,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "inline_sequence",
                                                                                        "row" => 173,
                                                                                        "col" => 29,
                                                                                        "children" => (
                                                                                            value!([
                                                                                                (value!([
                                                                                                    "emit" => "value_token_touch",
                                                                                                    "row" => 173,
                                                                                                    "col" => 29,
                                                                                                    "value" => ","
                                                                                                ])),
                                                                                                (value!([
                                                                                                    "emit" => "identifier",
                                                                                                    "row" => 173,
                                                                                                    "col" => 33,
                                                                                                    "value" => "_"
                                                                                                ]))
                                                                                            ])
                                                                                        )
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 173,
                                                            "col" => 39,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_expect",
                                                            "row" => 173,
                                                            "col" => 41,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "value_token_touch",
                                                                    "row" => 173,
                                                                    "col" => 48,
                                                                    "value" => ">"
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 173,
                                                            "col" => 52,
                                                            "value" => "_"
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 178,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 178,
                                "col" => 1,
                                "value" => "ParseletArgument"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 178,
                                "col" => 20,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 178,
                                        "col" => 21,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 179,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 179,
                                                            "col" => 5,
                                                            "value" => "T_Identifier"
                                                        ])),
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 179,
                                                            "col" => 18,
                                                            "value" => "_"
                                                        ])),
                                                        (value!([
                                                            "emit" => "op_mod_opt",
                                                            "row" => 179,
                                                            "col" => 20,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "inline_sequence",
                                                                    "row" => 179,
                                                                    "col" => 21,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "value_token_touch",
                                                                                "row" => 179,
                                                                                "col" => 21,
                                                                                "value" => "="
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "identifier",
                                                                                "row" => 179,
                                                                                "col" => 25,
                                                                                "value" => "_"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "op_mod_expect",
                                                                                "row" => 179,
                                                                                "col" => 27,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "op_mod_opt",
                                                                                        "row" => 179,
                                                                                        "col" => 34,
                                                                                        "children" => (
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 179,
                                                                                                "col" => 34,
                                                                                                "value" => "Expression"
                                                                                            ]))
                                                                                        )
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 179,
                                                            "col" => 49,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 179,
                                                                        "col" => 49,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 179,
                                                                        "col" => 53,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 179,
                                                                                "col" => 53,
                                                                                "value" => "arg"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 182,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 182,
                                "col" => 1,
                                "value" => "ParseletArguments"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 182,
                                "col" => 21,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 182,
                                        "col" => 22,
                                        "children" => (
                                            (value!([
                                                "emit" => "op_mod_pos",
                                                "row" => 183,
                                                "col" => 5,
                                                "children" => (
                                                    (value!([
                                                        "emit" => "inline_sequence",
                                                        "row" => 183,
                                                        "col" => 6,
                                                        "children" => (
                                                            value!([
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 183,
                                                                    "col" => 6,
                                                                    "value" => "ParseletArgument"
                                                                ])),
                                                                (value!([
                                                                    "emit" => "op_mod_opt",
                                                                    "row" => 183,
                                                                    "col" => 23,
                                                                    "children" => (
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 183,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "value_token_touch",
                                                                                        "row" => 183,
                                                                                        "col" => 24,
                                                                                        "value" => ","
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 183,
                                                                                        "col" => 28,
                                                                                        "value" => "_"
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ]))
                                                                    )
                                                                ]))
                                                            ])
                                                        )
                                                    ]))
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 188,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 188,
                                "col" => 1,
                                "value" => "StaticParseletInstance"
                            ])),
                            (value!([
                                "emit" => "block",
                                "row" => 188,
                                "col" => 26,
                                "children" => (
                                    value!([
                                        (value!([
                                            "emit" => "sequence",
                                            "row" => 188,
                                            "col" => 26,
                                            "children" => (
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 188,
                                                    "col" => 26,
                                                    "value" => "T_Consumable"
                                                ]))
                                            )
                                        ])),
                                        (value!([
                                            "emit" => "sequence",
                                            "row" => 188,
                                            "col" => 41,
                                            "children" => (
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 188,
                                                    "col" => 41,
                                                    "value" => "Parselet"
                                                ]))
                                            )
                                        ]))
                                    ])
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 190,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 190,
                                "col" => 1,
                                "value" => "ParseletInstanceArgument"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 190,
                                "col" => 28,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 190,
                                        "col" => 29,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 191,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 191,
                                                                "col" => 5,
                                                                "value" => "T_Identifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 191,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 191,
                                                                "col" => 20,
                                                                "value" => ":"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 191,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 191,
                                                                "col" => 26,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 191,
                                                                        "col" => 33,
                                                                        "value" => "Atomic"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 191,
                                                                "col" => 40,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 191,
                                                                "col" => 43,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 191,
                                                                            "col" => 43,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 191,
                                                                            "col" => 47,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 191,
                                                                                    "col" => 47,
                                                                                    "value" => "genarg_named"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 192,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 192,
                                                                "col" => 5,
                                                                "value" => "Atomic"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 192,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 192,
                                                                "col" => 15,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 192,
                                                                            "col" => 15,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 192,
                                                                            "col" => 19,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 192,
                                                                                    "col" => 19,
                                                                                    "value" => "genarg"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 195,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 195,
                                "col" => 1,
                                "value" => "ParseletInstance"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 195,
                                "col" => 20,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 195,
                                        "col" => 21,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 196,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 196,
                                                                "col" => 5,
                                                                "value" => "StaticParseletInstance"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 196,
                                                                "col" => 28,
                                                                "value" => "<"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 196,
                                                                "col" => 32,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_pos",
                                                                "row" => 196,
                                                                "col" => 34,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "inline_sequence",
                                                                        "row" => 196,
                                                                        "col" => 35,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 196,
                                                                                    "col" => 35,
                                                                                    "value" => "ParseletInstanceArgument"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "op_mod_opt",
                                                                                    "row" => 196,
                                                                                    "col" => 60,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "inline_sequence",
                                                                                            "row" => 196,
                                                                                            "col" => 61,
                                                                                            "children" => (
                                                                                                value!([
                                                                                                    (value!([
                                                                                                        "emit" => "value_token_touch",
                                                                                                        "row" => 196,
                                                                                                        "col" => 61,
                                                                                                        "value" => ","
                                                                                                    ])),
                                                                                                    (value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "row" => 196,
                                                                                                        "col" => 65,
                                                                                                        "value" => "_"
                                                                                                    ]))
                                                                                                ])
                                                                                            )
                                                                                        ]))
                                                                                    )
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 196,
                                                                "col" => 71,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 196,
                                                                "col" => 73,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 196,
                                                                        "col" => 80,
                                                                        "value" => ">"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 196,
                                                                "col" => 84,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 196,
                                                                "col" => 87,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 196,
                                                                            "col" => 87,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 196,
                                                                            "col" => 91,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 196,
                                                                                    "col" => 91,
                                                                                    "value" => "value_generic"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 197,
                                                    "col" => 5,
                                                    "value" => "StaticParseletInstance"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 202,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 202,
                                "col" => 1,
                                "value" => "InlineSequenceItem"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 202,
                                "col" => 22,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 202,
                                        "col" => 23,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 203,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 203,
                                                                "col" => 5,
                                                                "value" => "T_Alias"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 203,
                                                                "col" => 13,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 203,
                                                                "col" => 15,
                                                                "value" => "=>"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 203,
                                                                "col" => 20,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 203,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 203,
                                                                        "col" => 29,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 203,
                                                                "col" => 41,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 203,
                                                                            "col" => 41,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 203,
                                                                            "col" => 45,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 203,
                                                                                    "col" => 45,
                                                                                    "value" => "alias"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 204,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 204,
                                                                "col" => 5,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 204,
                                                                "col" => 16,
                                                                "value" => "=>"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 204,
                                                                "col" => 21,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 204,
                                                                "col" => 23,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 204,
                                                                        "col" => 30,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 204,
                                                                "col" => 42,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 204,
                                                                            "col" => 42,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 204,
                                                                            "col" => 46,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 204,
                                                                                    "col" => 46,
                                                                                    "value" => "alias"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 205,
                                                    "col" => 5,
                                                    "value" => "Expression"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 208,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 208,
                                "col" => 1,
                                "value" => "InlineSequence"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 208,
                                "col" => 18,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 208,
                                        "col" => 19,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 210,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 210,
                                                                "col" => 5,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 210,
                                                                "col" => 16,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 210,
                                                                "col" => 20,
                                                                "value" => ","
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 210,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 210,
                                                                "col" => 26,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_peek",
                                                                "row" => 210,
                                                                "col" => 30,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 210,
                                                                        "col" => 35,
                                                                        "value" => ")"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 210,
                                                                "col" => 40,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 210,
                                                                            "col" => 40,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 210,
                                                                            "col" => 44,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 210,
                                                                                    "col" => 44,
                                                                                    "value" => "list"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 213,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "op_mod_pos",
                                                                "row" => 213,
                                                                "col" => 5,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "inline_sequence",
                                                                        "row" => 213,
                                                                        "col" => 6,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 213,
                                                                                    "col" => 6,
                                                                                    "value" => "InlineSequenceItem"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 213,
                                                                                    "col" => 25,
                                                                                    "value" => "___"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "op_mod_opt",
                                                                                    "row" => 213,
                                                                                    "col" => 29,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "inline_sequence",
                                                                                            "row" => 213,
                                                                                            "col" => 30,
                                                                                            "children" => (
                                                                                                value!([
                                                                                                    (value!([
                                                                                                        "emit" => "value_token_touch",
                                                                                                        "row" => 213,
                                                                                                        "col" => 30,
                                                                                                        "value" => ","
                                                                                                    ])),
                                                                                                    (value!([
                                                                                                        "emit" => "identifier",
                                                                                                        "row" => 213,
                                                                                                        "col" => 34,
                                                                                                        "value" => "_"
                                                                                                    ]))
                                                                                                ])
                                                                                            )
                                                                                        ]))
                                                                                    )
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 213,
                                                                                    "col" => 38,
                                                                                    "value" => "___"
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 213,
                                                                "col" => 45,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 213,
                                                                            "col" => 45,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 213,
                                                                            "col" => 49,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 213,
                                                                                    "col" => 49,
                                                                                    "value" => "inline_sequence"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 216,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 216,
                                                                "col" => 5,
                                                                "value" => "Void"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 216,
                                                                "col" => 11,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 216,
                                                                            "col" => 11,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 216,
                                                                            "col" => 15,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 216,
                                                                                    "col" => 15,
                                                                                    "value" => "list"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 219,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 219,
                                "col" => 1,
                                "value" => "InlineBlock"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 219,
                                "col" => 15,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 219,
                                        "col" => 16,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 220,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 220,
                                                                "col" => 5,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 220,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 220,
                                                                "col" => 11,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 220,
                                                                "col" => 15,
                                                                "value" => "InlineSequence"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_pos",
                                                                "row" => 220,
                                                                "col" => 30,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "block",
                                                                        "row" => 220,
                                                                        "col" => 30,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 220,
                                                                                "col" => 31,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 220,
                                                                                            "col" => 31,
                                                                                            "value" => "___"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 220,
                                                                                            "col" => 35,
                                                                                            "value" => "|"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 220,
                                                                                            "col" => 39,
                                                                                            "value" => "_"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 220,
                                                                                            "col" => 41,
                                                                                            "value" => "___"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 220,
                                                                                            "col" => 45,
                                                                                            "value" => "InlineSequence"
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 220,
                                                                "col" => 62,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 220,
                                                                "col" => 66,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 220,
                                                                        "col" => 73,
                                                                        "value" => ")"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 220,
                                                                "col" => 78,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 220,
                                                                            "col" => 78,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 220,
                                                                            "col" => 82,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 220,
                                                                                    "col" => 82,
                                                                                    "value" => "block"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 221,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 221,
                                                                "col" => 5,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 221,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 221,
                                                                "col" => 11,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 221,
                                                                "col" => 15,
                                                                "value" => "InlineSequence"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 221,
                                                                "col" => 30,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 221,
                                                                "col" => 34,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 221,
                                                                        "col" => 41,
                                                                        "value" => ")"
                                                                    ]))
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 226,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 226,
                                "col" => 1,
                                "value" => "CallArgument"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 226,
                                "col" => 16,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 226,
                                        "col" => 17,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 227,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 227,
                                                                "col" => 5,
                                                                "value" => "T_Identifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 227,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 227,
                                                                "col" => 20,
                                                                "value" => "="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 227,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 227,
                                                                "col" => 26,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 227,
                                                                        "col" => 33,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 227,
                                                                "col" => 45,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 227,
                                                                            "col" => 45,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 227,
                                                                            "col" => 49,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 227,
                                                                                    "col" => 49,
                                                                                    "value" => "callarg_named"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 228,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 228,
                                                                "col" => 5,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 228,
                                                                "col" => 17,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 228,
                                                                            "col" => 17,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 228,
                                                                            "col" => 21,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 228,
                                                                                    "col" => 21,
                                                                                    "value" => "callarg"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 231,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 231,
                                "col" => 1,
                                "value" => "CallArguments"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 231,
                                "col" => 17,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 231,
                                        "col" => 18,
                                        "children" => (
                                            (value!([
                                                "emit" => "op_mod_pos",
                                                "row" => 232,
                                                "col" => 5,
                                                "children" => (
                                                    (value!([
                                                        "emit" => "inline_sequence",
                                                        "row" => 232,
                                                        "col" => 6,
                                                        "children" => (
                                                            value!([
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 232,
                                                                    "col" => 6,
                                                                    "value" => "CallArgument"
                                                                ])),
                                                                (value!([
                                                                    "emit" => "op_mod_opt",
                                                                    "row" => 232,
                                                                    "col" => 19,
                                                                    "children" => (
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 232,
                                                                            "col" => 20,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "value_token_touch",
                                                                                        "row" => 232,
                                                                                        "col" => 20,
                                                                                        "value" => ","
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 232,
                                                                                        "col" => 24,
                                                                                        "value" => "_"
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ]))
                                                                    )
                                                                ])),
                                                                (value!([
                                                                    "emit" => "identifier",
                                                                    "row" => 232,
                                                                    "col" => 28,
                                                                    "value" => "___"
                                                                ]))
                                                            ])
                                                        )
                                                    ]))
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 237,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 237,
                                "col" => 1,
                                "value" => "TokenLiteral"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 237,
                                "col" => 16,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 237,
                                        "col" => 17,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 238,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 238,
                                                                "col" => 5,
                                                                "value" => "'"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 238,
                                                                "col" => 10,
                                                                "value" => "T_Touch"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 238,
                                                                "col" => 18,
                                                                "value" => "'"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 238,
                                                                "col" => 24,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 238,
                                                                            "col" => 24,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 238,
                                                                            "col" => 28,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 238,
                                                                                    "col" => 28,
                                                                                    "value" => "value_token_match"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 239,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 239,
                                                                "col" => 5,
                                                                "value" => "T_Touch"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 239,
                                                                "col" => 14,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 239,
                                                                            "col" => 14,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 239,
                                                                            "col" => 18,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 239,
                                                                                    "col" => 18,
                                                                                    "value" => "value_token_touch"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 240,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 240,
                                                                "col" => 5,
                                                                "value" => "."
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 240,
                                                                "col" => 10,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 240,
                                                                            "col" => 10,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 240,
                                                                            "col" => 14,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 240,
                                                                                    "col" => 14,
                                                                                    "value" => "value_token_any"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 241,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 241,
                                                                "col" => 5,
                                                                "value" => "["
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 241,
                                                                "col" => 9,
                                                                "value" => "Ccl"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 241,
                                                                "col" => 13,
                                                                "value" => "]"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 241,
                                                                "col" => 18,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 241,
                                                                            "col" => 18,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 241,
                                                                            "col" => 22,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 241,
                                                                                    "col" => 22,
                                                                                    "value" => "value_token_ccl"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 244,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 244,
                                "col" => 1,
                                "value" => "TokenAtom"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 244,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 244,
                                        "col" => 14,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 245,
                                                    "col" => 5,
                                                    "value" => "TokenLiteral"
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 246,
                                                    "col" => 5,
                                                    "value" => "InlineBlock"
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 247,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 247,
                                                                "col" => 5,
                                                                "value" => "@"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 247,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 247,
                                                                "col" => 11,
                                                                "value" => "InlineBlock"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 247,
                                                                "col" => 24,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 247,
                                                                            "col" => 24,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 247,
                                                                            "col" => 28,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 247,
                                                                                    "col" => 28,
                                                                                    "value" => "area"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 248,
                                                    "col" => 5,
                                                    "value" => "Block"
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 249,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 249,
                                                                "col" => 5,
                                                                "value" => "ParseletInstance"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 249,
                                                                "col" => 22,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 249,
                                                                "col" => 26,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 249,
                                                                "col" => 28,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 249,
                                                                "col" => 32,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 249,
                                                                        "col" => 32,
                                                                        "value" => "CallArguments"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 249,
                                                                "col" => 47,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 249,
                                                                "col" => 51,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 249,
                                                                        "col" => 58,
                                                                        "value" => ")"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 249,
                                                                "col" => 63,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 249,
                                                                            "col" => 63,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 249,
                                                                            "col" => 67,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 249,
                                                                                    "col" => 67,
                                                                                    "value" => "call"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 250,
                                                    "col" => 5,
                                                    "value" => "ParseletInstance"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 253,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 253,
                                "col" => 1,
                                "value" => "Token1"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 253,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 253,
                                        "col" => 11,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 254,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 254,
                                                                "col" => 5,
                                                                "value" => "TokenAtom"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 254,
                                                                "col" => 15,
                                                                "value" => "+"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 254,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 254,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 254,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 254,
                                                                                    "col" => 24,
                                                                                    "value" => "op_mod_pos"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 255,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 255,
                                                                "col" => 5,
                                                                "value" => "TokenAtom"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 255,
                                                                "col" => 15,
                                                                "value" => "*"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 255,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 255,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 255,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 255,
                                                                                    "col" => 24,
                                                                                    "value" => "op_mod_kle"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 256,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 256,
                                                                "col" => 5,
                                                                "value" => "TokenAtom"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 256,
                                                                "col" => 15,
                                                                "value" => "?"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 256,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 256,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 256,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 256,
                                                                                    "col" => 24,
                                                                                    "value" => "op_mod_opt"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 257,
                                                    "col" => 5,
                                                    "value" => "TokenAtom"
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 258,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 258,
                                                                "col" => 5,
                                                                "value" => "peek"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 258,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 258,
                                                                "col" => 33,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 258,
                                                                        "col" => 40,
                                                                        "value" => "Token1"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 258,
                                                                "col" => 48,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 258,
                                                                            "col" => 48,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 258,
                                                                            "col" => 52,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 258,
                                                                                    "col" => 52,
                                                                                    "value" => "op_mod_peek"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 259,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 259,
                                                                "col" => 5,
                                                                "value" => "not"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 259,
                                                                "col" => 11,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 259,
                                                                "col" => 32,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 259,
                                                                        "col" => 39,
                                                                        "value" => "Token1"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 259,
                                                                "col" => 47,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 259,
                                                                            "col" => 47,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 259,
                                                                            "col" => 51,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 259,
                                                                                    "col" => 51,
                                                                                    "value" => "op_mod_not"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 260,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 260,
                                                                "col" => 5,
                                                                "value" => "expect"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 260,
                                                                "col" => 14,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 260,
                                                                "col" => 35,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 260,
                                                                        "col" => 42,
                                                                        "value" => "Token1"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 260,
                                                                "col" => 50,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 260,
                                                                            "col" => 50,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 260,
                                                                            "col" => 54,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 260,
                                                                                    "col" => 54,
                                                                                    "value" => "op_mod_expect"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 267,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 267,
                                "col" => 1,
                                "value" => "Literal"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 267,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 267,
                                        "col" => 12,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 268,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 268,
                                                                "col" => 5,
                                                                "value" => "true"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 268,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 268,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 268,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 268,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 268,
                                                                                    "col" => 38,
                                                                                    "value" => "value_true"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 269,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 269,
                                                                "col" => 5,
                                                                "value" => "false"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 269,
                                                                "col" => 13,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 269,
                                                                "col" => 35,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 269,
                                                                            "col" => 35,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 269,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 269,
                                                                                    "col" => 39,
                                                                                    "value" => "value_false"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 270,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 270,
                                                                "col" => 5,
                                                                "value" => "void"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 270,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 270,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 270,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 270,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 270,
                                                                                    "col" => 38,
                                                                                    "value" => "value_void"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 271,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 271,
                                                                "col" => 5,
                                                                "value" => "null"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 271,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 271,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 271,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 271,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 271,
                                                                                    "col" => 38,
                                                                                    "value" => "value_null"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 272,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 272,
                                                                "col" => 5,
                                                                "value" => "T_String"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 272,
                                                                "col" => 15,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 272,
                                                                            "col" => 15,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 272,
                                                                            "col" => 19,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 272,
                                                                                    "col" => 19,
                                                                                    "value" => "value_string"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 273,
                                                    "col" => 5,
                                                    "value" => "T_Float"
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 274,
                                                    "col" => 5,
                                                    "value" => "T_Integer"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 279,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 279,
                                "col" => 1,
                                "value" => "Atomic"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 279,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 279,
                                        "col" => 11,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 280,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 280,
                                                                "col" => 5,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 280,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 280,
                                                                "col" => 11,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 280,
                                                                "col" => 15,
                                                                "value" => "HoldExpression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 280,
                                                                "col" => 30,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 280,
                                                                "col" => 34,
                                                                "value" => ")"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 281,
                                                    "col" => 5,
                                                    "value" => "Literal"
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 282,
                                                    "col" => 5,
                                                    "value" => "Token1"
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 283,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 283,
                                                                "col" => 5,
                                                                "value" => "if"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 283,
                                                                "col" => 10,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 283,
                                                                "col" => 31,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 283,
                                                                "col" => 42,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 283,
                                                                "col" => 46,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 283,
                                                                        "col" => 53,
                                                                        "value" => "Statement"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 283,
                                                                "col" => 63,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "inline_sequence",
                                                                        "row" => 283,
                                                                        "col" => 64,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 283,
                                                                                    "col" => 64,
                                                                                    "value" => "___"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "value_token_touch",
                                                                                    "row" => 283,
                                                                                    "col" => 68,
                                                                                    "value" => "else"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 283,
                                                                                    "col" => 75,
                                                                                    "value" => "_SeparatedIdentifier"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 283,
                                                                                    "col" => 96,
                                                                                    "value" => "___"
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "op_mod_expect",
                                                                                    "row" => 283,
                                                                                    "col" => 100,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 283,
                                                                                            "col" => 107,
                                                                                            "value" => "Statement"
                                                                                        ]))
                                                                                    )
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 283,
                                                                "col" => 120,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 283,
                                                                            "col" => 120,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 283,
                                                                            "col" => 124,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 283,
                                                                                    "col" => 124,
                                                                                    "value" => "op_if"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 284,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 284,
                                                                "col" => 5,
                                                                "value" => "for"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 11,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "block",
                                                                "row" => 284,
                                                                "col" => 32,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 284,
                                                                            "col" => 33,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 284,
                                                                                    "col" => 33,
                                                                                    "value" => "Sequence"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 284,
                                                                            "col" => 44,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 284,
                                                                                    "col" => 44,
                                                                                    "value" => "Nop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 284,
                                                                "col" => 49,
                                                                "value" => ";"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 53,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "block",
                                                                "row" => 284,
                                                                "col" => 55,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 284,
                                                                            "col" => 56,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 284,
                                                                                    "col" => 56,
                                                                                    "value" => "Sequence"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 284,
                                                                            "col" => 67,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 284,
                                                                                    "col" => 67,
                                                                                    "value" => "Nop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 284,
                                                                "col" => 72,
                                                                "value" => ";"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 76,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 78,
                                                                "value" => "Statement"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 88,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 284,
                                                                "col" => 90,
                                                                "value" => "Block"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 284,
                                                                "col" => 97,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 284,
                                                                            "col" => 97,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 284,
                                                                            "col" => 101,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 284,
                                                                                    "col" => 101,
                                                                                    "value" => "op_for"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 285,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 285,
                                                                "col" => 5,
                                                                "value" => "for"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 11,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "block",
                                                                "row" => 285,
                                                                "col" => 32,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 285,
                                                                            "col" => 33,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 285,
                                                                                    "col" => 33,
                                                                                    "value" => "Sequence"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 285,
                                                                            "col" => 44,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 285,
                                                                                    "col" => 44,
                                                                                    "value" => "Nop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 285,
                                                                "col" => 49,
                                                                "value" => ";"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 53,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "block",
                                                                "row" => 285,
                                                                "col" => 55,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 285,
                                                                            "col" => 56,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 285,
                                                                                    "col" => 56,
                                                                                    "value" => "Sequence"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "inline_sequence",
                                                                            "row" => 285,
                                                                            "col" => 67,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "identifier",
                                                                                    "row" => 285,
                                                                                    "col" => 67,
                                                                                    "value" => "Nop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 285,
                                                                "col" => 72,
                                                                "value" => ";"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 76,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 78,
                                                                "value" => "Nop"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 82,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 285,
                                                                "col" => 84,
                                                                "value" => "Block"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 285,
                                                                "col" => 91,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 285,
                                                                            "col" => 91,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 285,
                                                                            "col" => 95,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 285,
                                                                                    "col" => 95,
                                                                                    "value" => "op_for"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 286,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 286,
                                                                "col" => 5,
                                                                "value" => "for"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 286,
                                                                "col" => 11,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 286,
                                                                "col" => 33,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 286,
                                                                            "col" => 33,
                                                                            "value" => "error"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 286,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 286,
                                                                                    "col" => 39,
                                                                                    "value" => "'for': Expecting initial; condition; increment { body }"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 287,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 287,
                                                                "col" => 5,
                                                                "value" => "loop"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 287,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 287,
                                                                "col" => 33,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 287,
                                                                "col" => 44,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 287,
                                                                "col" => 46,
                                                                "value" => "Statement"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 287,
                                                                "col" => 57,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 287,
                                                                            "col" => 57,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 287,
                                                                            "col" => 61,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 287,
                                                                                    "col" => 61,
                                                                                    "value" => "op_loop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 288,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 288,
                                                                "col" => 5,
                                                                "value" => "loop"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 288,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 288,
                                                                "col" => 33,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 288,
                                                                        "col" => 40,
                                                                        "value" => "Statement"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 288,
                                                                "col" => 51,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 288,
                                                                            "col" => 51,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 288,
                                                                            "col" => 55,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 288,
                                                                                    "col" => 55,
                                                                                    "value" => "op_loop"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 289,
                                                    "col" => 5,
                                                    "value" => "Load"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 294,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 294,
                                "col" => 1,
                                "value" => "Rvalue"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 294,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 294,
                                        "col" => 11,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 295,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 295,
                                                                "col" => 5,
                                                                "value" => "Rvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 295,
                                                                "col" => 12,
                                                                "value" => "("
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 295,
                                                                "col" => 16,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 295,
                                                                "col" => 18,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 295,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 295,
                                                                        "col" => 22,
                                                                        "value" => "CallArguments"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 295,
                                                                "col" => 37,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 295,
                                                                        "col" => 44,
                                                                        "value" => ")"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 295,
                                                                "col" => 49,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 295,
                                                                            "col" => 49,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 295,
                                                                            "col" => 53,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 295,
                                                                                    "col" => 53,
                                                                                    "value" => "call"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 296,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 296,
                                                                "col" => 5,
                                                                "value" => "Rvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_kle",
                                                                "row" => 296,
                                                                "col" => 12,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "block",
                                                                        "row" => 296,
                                                                        "col" => 12,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 296,
                                                                                    "col" => 13,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 296,
                                                                                            "col" => 13,
                                                                                            "value" => "Attribute"
                                                                                        ]))
                                                                                    )
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 296,
                                                                                    "col" => 25,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 296,
                                                                                            "col" => 25,
                                                                                            "value" => "Subscript"
                                                                                        ]))
                                                                                    )
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 296,
                                                                "col" => 38,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 296,
                                                                            "col" => 38,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 296,
                                                                            "col" => 42,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 296,
                                                                                    "col" => 42,
                                                                                    "value" => "rvalue"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 297,
                                                    "col" => 5,
                                                    "value" => "Atomic"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 300,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 300,
                                "col" => 1,
                                "value" => "Unary"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 300,
                                "col" => 9,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 300,
                                        "col" => 10,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 301,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 301,
                                                                "col" => 5,
                                                                "value" => "-"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_not",
                                                                "row" => 301,
                                                                "col" => 9,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 301,
                                                                        "col" => 13,
                                                                        "value" => "-"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 301,
                                                                "col" => 17,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 301,
                                                                "col" => 19,
                                                                "value" => "Unary"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 301,
                                                                "col" => 26,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 301,
                                                                            "col" => 26,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 301,
                                                                            "col" => 30,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 301,
                                                                                    "col" => 30,
                                                                                    "value" => "op_unary_neg"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 302,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 302,
                                                                "col" => 5,
                                                                "value" => "!"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 302,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 302,
                                                                "col" => 11,
                                                                "value" => "Unary"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 302,
                                                                "col" => 18,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 302,
                                                                            "col" => 18,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 302,
                                                                            "col" => 22,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 302,
                                                                                    "col" => 22,
                                                                                    "value" => "op_unary_not"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 303,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 303,
                                                                "col" => 5,
                                                                "value" => "Rvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 303,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 306,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 306,
                                "col" => 1,
                                "value" => "MulDiv"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 306,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 306,
                                        "col" => 11,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 307,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 307,
                                                                "col" => 5,
                                                                "value" => "MulDiv"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 307,
                                                                "col" => 12,
                                                                "value" => "*"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 307,
                                                                "col" => 16,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 307,
                                                                "col" => 18,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 307,
                                                                        "col" => 25,
                                                                        "value" => "Unary"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 307,
                                                                "col" => 32,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 307,
                                                                            "col" => 32,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 307,
                                                                            "col" => 36,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 307,
                                                                                    "col" => 36,
                                                                                    "value" => "op_binary_mul"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 308,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 308,
                                                                "col" => 5,
                                                                "value" => "MulDiv"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 308,
                                                                "col" => 12,
                                                                "value" => "//"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 308,
                                                                "col" => 17,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 308,
                                                                "col" => 19,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 308,
                                                                        "col" => 26,
                                                                        "value" => "Unary"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 308,
                                                                "col" => 33,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 308,
                                                                            "col" => 33,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 308,
                                                                            "col" => 37,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 308,
                                                                                    "col" => 37,
                                                                                    "value" => "op_binary_divi"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 309,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 309,
                                                                "col" => 5,
                                                                "value" => "MulDiv"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 309,
                                                                "col" => 12,
                                                                "value" => "/"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 309,
                                                                "col" => 16,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 309,
                                                                "col" => 18,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 309,
                                                                        "col" => 25,
                                                                        "value" => "Unary"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 309,
                                                                "col" => 32,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 309,
                                                                            "col" => 32,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 309,
                                                                            "col" => 36,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 309,
                                                                                    "col" => 36,
                                                                                    "value" => "op_binary_div"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 310,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 310,
                                                                "col" => 5,
                                                                "value" => "MulDiv"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 310,
                                                                "col" => 12,
                                                                "value" => "%"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 310,
                                                                "col" => 16,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 310,
                                                                "col" => 18,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 310,
                                                                        "col" => 25,
                                                                        "value" => "Unary"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 310,
                                                                "col" => 32,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 310,
                                                                            "col" => 32,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 310,
                                                                            "col" => 36,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 310,
                                                                                    "col" => 36,
                                                                                    "value" => "op_binary_mul"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 311,
                                                    "col" => 5,
                                                    "value" => "Unary"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 314,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 314,
                                "col" => 1,
                                "value" => "AddSub"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 314,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 314,
                                        "col" => 11,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 315,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 315,
                                                                "col" => 5,
                                                                "value" => "AddSub"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 315,
                                                                "col" => 12,
                                                                "value" => "+"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_not",
                                                                "row" => 315,
                                                                "col" => 16,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 315,
                                                                        "col" => 20,
                                                                        "value" => "+"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 315,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 315,
                                                                "col" => 26,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 315,
                                                                        "col" => 33,
                                                                        "value" => "MulDiv"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 315,
                                                                "col" => 41,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 315,
                                                                            "col" => 41,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 315,
                                                                            "col" => 45,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 315,
                                                                                    "col" => 45,
                                                                                    "value" => "op_binary_add"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 316,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 316,
                                                                "col" => 5,
                                                                "value" => "AddSub"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 316,
                                                                "col" => 12,
                                                                "value" => "-"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_not",
                                                                "row" => 316,
                                                                "col" => 16,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 316,
                                                                        "col" => 20,
                                                                        "value" => "-"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 316,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 316,
                                                                "col" => 26,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 316,
                                                                        "col" => 33,
                                                                        "value" => "MulDiv"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 316,
                                                                "col" => 41,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 316,
                                                                            "col" => 41,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 316,
                                                                            "col" => 45,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 316,
                                                                                    "col" => 45,
                                                                                    "value" => "op_binary_sub"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 317,
                                                    "col" => 5,
                                                    "value" => "MulDiv"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 320,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 320,
                                "col" => 1,
                                "value" => "Compare"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 320,
                                "col" => 11,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 320,
                                        "col" => 12,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 321,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 321,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 321,
                                                                "col" => 13,
                                                                "value" => "=="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 321,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 321,
                                                                "col" => 20,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 321,
                                                                        "col" => 27,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 321,
                                                                "col" => 35,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 321,
                                                                            "col" => 35,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 321,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 321,
                                                                                    "col" => 39,
                                                                                    "value" => "op_compare_eq"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 322,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 322,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 322,
                                                                "col" => 13,
                                                                "value" => "!="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 322,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 322,
                                                                "col" => 20,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 322,
                                                                        "col" => 27,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 322,
                                                                "col" => 35,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 322,
                                                                            "col" => 35,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 322,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 322,
                                                                                    "col" => 39,
                                                                                    "value" => "op_compare_neq"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 323,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 323,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 323,
                                                                "col" => 13,
                                                                "value" => "<="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 323,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 323,
                                                                "col" => 20,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 323,
                                                                        "col" => 27,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 323,
                                                                "col" => 35,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 323,
                                                                            "col" => 35,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 323,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 323,
                                                                                    "col" => 39,
                                                                                    "value" => "op_compare_lteq"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 324,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 324,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 324,
                                                                "col" => 13,
                                                                "value" => ">="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 324,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 324,
                                                                "col" => 20,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 324,
                                                                        "col" => 27,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 324,
                                                                "col" => 35,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 324,
                                                                            "col" => 35,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 324,
                                                                            "col" => 39,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 324,
                                                                                    "col" => 39,
                                                                                    "value" => "op_compare_gteq"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 325,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 325,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 325,
                                                                "col" => 13,
                                                                "value" => "<"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 325,
                                                                "col" => 17,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 325,
                                                                "col" => 19,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 325,
                                                                        "col" => 26,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 325,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 325,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 325,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 325,
                                                                                    "col" => 38,
                                                                                    "value" => "op_compare_lt"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 326,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 326,
                                                                "col" => 5,
                                                                "value" => "Compare"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 326,
                                                                "col" => 13,
                                                                "value" => ">"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 326,
                                                                "col" => 17,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 326,
                                                                "col" => 19,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 326,
                                                                        "col" => 26,
                                                                        "value" => "AddSub"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 326,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 326,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 326,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 326,
                                                                                    "col" => 38,
                                                                                    "value" => "op_compare_gt"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 327,
                                                    "col" => 5,
                                                    "value" => "AddSub"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 330,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 330,
                                "col" => 1,
                                "value" => "LogicalAnd"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 330,
                                "col" => 14,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 330,
                                        "col" => 15,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 331,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 331,
                                                                "col" => 5,
                                                                "value" => "LogicalAnd"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 331,
                                                                "col" => 16,
                                                                "value" => "&&"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 331,
                                                                "col" => 21,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 331,
                                                                "col" => 23,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 331,
                                                                        "col" => 30,
                                                                        "value" => "Compare"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 331,
                                                                "col" => 39,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 331,
                                                                            "col" => 39,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 331,
                                                                            "col" => 43,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 331,
                                                                                    "col" => 43,
                                                                                    "value" => "op_logical_and"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 332,
                                                    "col" => 5,
                                                    "value" => "Compare"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 335,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 335,
                                "col" => 1,
                                "value" => "LogicalOr"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 335,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 335,
                                        "col" => 14,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 336,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 336,
                                                                "col" => 5,
                                                                "value" => "LogicalOr"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 336,
                                                                "col" => 15,
                                                                "value" => "||"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 336,
                                                                "col" => 20,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 336,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 336,
                                                                        "col" => 29,
                                                                        "value" => "LogicalAnd"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 336,
                                                                "col" => 41,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 336,
                                                                            "col" => 41,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 336,
                                                                            "col" => 45,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 336,
                                                                                    "col" => 45,
                                                                                    "value" => "op_logical_or"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 337,
                                                    "col" => 5,
                                                    "value" => "LogicalAnd"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 340,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 340,
                                "col" => 1,
                                "value" => "HoldExpression"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 340,
                                "col" => 18,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 340,
                                        "col" => 19,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 341,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 341,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 341,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 341,
                                                                "col" => 14,
                                                                "value" => "+="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 341,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 341,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 341,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 341,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 341,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 341,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 341,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_add_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 342,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 342,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 342,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 342,
                                                                "col" => 14,
                                                                "value" => "-="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 342,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 342,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 342,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 342,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 342,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 342,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 342,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_sub_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 343,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 343,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 343,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 343,
                                                                "col" => 14,
                                                                "value" => "*="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 343,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 343,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 343,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 343,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 343,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 343,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 343,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_mul_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 344,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 344,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 344,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 344,
                                                                "col" => 14,
                                                                "value" => "/="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 344,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 344,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 344,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 344,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 344,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 344,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 344,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_div_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 345,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 345,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 345,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 345,
                                                                "col" => 14,
                                                                "value" => "//="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 345,
                                                                "col" => 20,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 345,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 345,
                                                                        "col" => 29,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 345,
                                                                "col" => 45,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 345,
                                                                            "col" => 45,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 345,
                                                                            "col" => 49,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 345,
                                                                                    "col" => 49,
                                                                                    "value" => "assign_divi_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 346,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 346,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 346,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 346,
                                                                "col" => 14,
                                                                "value" => "%="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 346,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 346,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 346,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 346,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 346,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 346,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 346,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_mod_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 347,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 347,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 347,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 347,
                                                                "col" => 14,
                                                                "value" => "="
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_not",
                                                                "row" => 347,
                                                                "col" => 18,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "block",
                                                                        "row" => 347,
                                                                        "col" => 22,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 347,
                                                                                    "col" => 23,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 347,
                                                                                            "col" => 23,
                                                                                            "value" => ">"
                                                                                        ]))
                                                                                    )
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 347,
                                                                                    "col" => 29,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 347,
                                                                                            "col" => 29,
                                                                                            "value" => "="
                                                                                        ]))
                                                                                    )
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 347,
                                                                "col" => 34,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 347,
                                                                "col" => 36,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 347,
                                                                        "col" => 43,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 347,
                                                                "col" => 59,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 347,
                                                                            "col" => 59,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 347,
                                                                            "col" => 63,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 347,
                                                                                    "col" => 63,
                                                                                    "value" => "assign_hold"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 348,
                                                    "col" => 5,
                                                    "value" => "LogicalOr"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 351,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 351,
                                "col" => 1,
                                "value" => "Expression"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 351,
                                "col" => 14,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 351,
                                        "col" => 15,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 352,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 352,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 352,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 352,
                                                                "col" => 14,
                                                                "value" => "+="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 352,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 352,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 352,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 352,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 352,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 352,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 352,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_add"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 353,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 353,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 353,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 353,
                                                                "col" => 14,
                                                                "value" => "-="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 353,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 353,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 353,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 353,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 353,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 353,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 353,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_sub"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 354,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 354,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 354,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 354,
                                                                "col" => 14,
                                                                "value" => "*="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 354,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 354,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 354,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 354,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 354,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 354,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 354,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_mul"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 355,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 355,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 355,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 355,
                                                                "col" => 14,
                                                                "value" => "/="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 355,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 355,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 355,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 355,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 355,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 355,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 355,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_div"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 356,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 356,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 356,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 356,
                                                                "col" => 14,
                                                                "value" => "//="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 356,
                                                                "col" => 20,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 356,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 356,
                                                                        "col" => 29,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 356,
                                                                "col" => 45,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 356,
                                                                            "col" => 45,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 356,
                                                                            "col" => 49,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 356,
                                                                                    "col" => 49,
                                                                                    "value" => "assign_divi"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 357,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 357,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 357,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 357,
                                                                "col" => 14,
                                                                "value" => "%="
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 357,
                                                                "col" => 19,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 357,
                                                                "col" => 21,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 357,
                                                                        "col" => 28,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 357,
                                                                "col" => 44,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 357,
                                                                            "col" => 44,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 357,
                                                                            "col" => 48,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 357,
                                                                                    "col" => 48,
                                                                                    "value" => "assign_mod"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 358,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 358,
                                                                "col" => 5,
                                                                "value" => "Lvalue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 358,
                                                                "col" => 12,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 358,
                                                                "col" => 14,
                                                                "value" => "="
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_not",
                                                                "row" => 358,
                                                                "col" => 18,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "block",
                                                                        "row" => 358,
                                                                        "col" => 22,
                                                                        "children" => (
                                                                            value!([
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 358,
                                                                                    "col" => 23,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 358,
                                                                                            "col" => 23,
                                                                                            "value" => ">"
                                                                                        ]))
                                                                                    )
                                                                                ])),
                                                                                (value!([
                                                                                    "emit" => "inline_sequence",
                                                                                    "row" => 358,
                                                                                    "col" => 29,
                                                                                    "children" => (
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 358,
                                                                                            "col" => 29,
                                                                                            "value" => "="
                                                                                        ]))
                                                                                    )
                                                                                ]))
                                                                            ])
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 358,
                                                                "col" => 34,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 358,
                                                                "col" => 36,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 358,
                                                                        "col" => 43,
                                                                        "value" => "HoldExpression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 358,
                                                                "col" => 59,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 358,
                                                                            "col" => 59,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 358,
                                                                            "col" => 63,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 358,
                                                                                    "col" => 63,
                                                                                    "value" => "assign"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 359,
                                                    "col" => 5,
                                                    "value" => "LogicalOr"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 364,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 364,
                                "col" => 1,
                                "value" => "Statement"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 364,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 364,
                                        "col" => 14,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 365,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 365,
                                                                "col" => 5,
                                                                "value" => "accept"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 365,
                                                                "col" => 14,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 365,
                                                                "col" => 35,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 365,
                                                                        "col" => 35,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 365,
                                                                "col" => 48,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 365,
                                                                            "col" => 48,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 365,
                                                                            "col" => 52,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 365,
                                                                                    "col" => 52,
                                                                                    "value" => "op_accept"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 366,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 366,
                                                                "col" => 5,
                                                                "value" => "break"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 366,
                                                                "col" => 13,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 366,
                                                                "col" => 34,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 366,
                                                                        "col" => 34,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 366,
                                                                "col" => 47,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 366,
                                                                            "col" => 47,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 366,
                                                                            "col" => 51,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 366,
                                                                                    "col" => 51,
                                                                                    "value" => "op_break"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 367,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 367,
                                                                "col" => 5,
                                                                "value" => "continue"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 367,
                                                                "col" => 16,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 367,
                                                                "col" => 37,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 367,
                                                                        "col" => 37,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 367,
                                                                "col" => 50,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 367,
                                                                            "col" => 50,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 367,
                                                                            "col" => 54,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 367,
                                                                                    "col" => 54,
                                                                                    "value" => "op_continue"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 368,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 368,
                                                                "col" => 5,
                                                                "value" => "exit"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 368,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 368,
                                                                "col" => 33,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 368,
                                                                        "col" => 33,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 368,
                                                                "col" => 46,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 368,
                                                                            "col" => 46,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 368,
                                                                            "col" => 50,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 368,
                                                                                    "col" => 50,
                                                                                    "value" => "op_exit"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 369,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 369,
                                                                "col" => 5,
                                                                "value" => "next"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 369,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 369,
                                                                "col" => 34,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 369,
                                                                            "col" => 34,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 369,
                                                                            "col" => 38,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 369,
                                                                                    "col" => 38,
                                                                                    "value" => "op_next"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 370,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 370,
                                                                "col" => 5,
                                                                "value" => "push"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 370,
                                                                "col" => 12,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 370,
                                                                "col" => 33,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 370,
                                                                        "col" => 33,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 370,
                                                                "col" => 46,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 370,
                                                                            "col" => 46,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 370,
                                                                            "col" => 50,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 370,
                                                                                    "col" => 50,
                                                                                    "value" => "op_push"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 371,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 371,
                                                                "col" => 5,
                                                                "value" => "reject"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 371,
                                                                "col" => 14,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 371,
                                                                "col" => 36,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 371,
                                                                            "col" => 36,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 371,
                                                                            "col" => 40,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 371,
                                                                                    "col" => 40,
                                                                                    "value" => "op_reject"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 372,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 372,
                                                                "col" => 5,
                                                                "value" => "repeat"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 372,
                                                                "col" => 14,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 372,
                                                                "col" => 35,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 372,
                                                                        "col" => 35,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 372,
                                                                "col" => 48,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 372,
                                                                            "col" => 48,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 372,
                                                                            "col" => 52,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 372,
                                                                                    "col" => 52,
                                                                                    "value" => "op_repeat"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 373,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 373,
                                                                "col" => 5,
                                                                "value" => "return"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 373,
                                                                "col" => 14,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_opt",
                                                                "row" => 373,
                                                                "col" => 35,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 373,
                                                                        "col" => 35,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 373,
                                                                "col" => 48,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 373,
                                                                            "col" => 48,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 373,
                                                                            "col" => 52,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 373,
                                                                                    "col" => 52,
                                                                                    "value" => "op_accept"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 374,
                                                    "col" => 5,
                                                    "value" => "Expression"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 379,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 379,
                                "col" => 1,
                                "value" => "Block"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 379,
                                "col" => 9,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 379,
                                        "col" => 10,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 380,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 380,
                                                                "col" => 5,
                                                                "value" => "{"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 380,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 380,
                                                                "col" => 11,
                                                                "value" => "___"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 380,
                                                                "col" => 15,
                                                                "value" => "}"
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 380,
                                                                "col" => 20,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 380,
                                                                            "col" => 20,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 380,
                                                                            "col" => 24,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 380,
                                                                                    "col" => 24,
                                                                                    "value" => "value_void"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 381,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 381,
                                                                "col" => 5,
                                                                "value" => "{"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 381,
                                                                "col" => 9,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_kle",
                                                                "row" => 381,
                                                                "col" => 11,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 381,
                                                                        "col" => 11,
                                                                        "value" => "Instruction"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 381,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 381,
                                                                "col" => 26,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "value_token_touch",
                                                                        "row" => 381,
                                                                        "col" => 33,
                                                                        "value" => "}"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 381,
                                                                "col" => 38,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 381,
                                                                            "col" => 38,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 381,
                                                                            "col" => 42,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 381,
                                                                                    "col" => 42,
                                                                                    "value" => "block"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 384,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 384,
                                "col" => 1,
                                "value" => "SequenceItem"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 384,
                                "col" => 16,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 384,
                                        "col" => 17,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 385,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 385,
                                                                "col" => 5,
                                                                "value" => "T_Alias"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 385,
                                                                "col" => 13,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 385,
                                                                "col" => 15,
                                                                "value" => "=>"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 385,
                                                                "col" => 20,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 385,
                                                                "col" => 22,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 385,
                                                                        "col" => 29,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 385,
                                                                "col" => 41,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 385,
                                                                            "col" => 41,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 385,
                                                                            "col" => 45,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 385,
                                                                                    "col" => 45,
                                                                                    "value" => "alias"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 386,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 386,
                                                                "col" => 5,
                                                                "value" => "Expression"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 386,
                                                                "col" => 16,
                                                                "value" => "=>"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 386,
                                                                "col" => 21,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 386,
                                                                "col" => 23,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 386,
                                                                        "col" => 30,
                                                                        "value" => "Expression"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 386,
                                                                "col" => 42,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 386,
                                                                            "col" => 42,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 386,
                                                                            "col" => 46,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 386,
                                                                                    "col" => 46,
                                                                                    "value" => "alias"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 387,
                                                    "col" => 5,
                                                    "value" => "Statement"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 390,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 390,
                                "col" => 1,
                                "value" => "Sequence"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 390,
                                "col" => 12,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 390,
                                        "col" => 13,
                                        "children" => (
                                            (value!([
                                                "emit" => "sequence",
                                                "row" => 391,
                                                "col" => 5,
                                                "children" => (
                                                    value!([
                                                        (value!([
                                                            "emit" => "op_mod_pos",
                                                            "row" => 391,
                                                            "col" => 5,
                                                            "children" => (
                                                                (value!([
                                                                    "emit" => "inline_sequence",
                                                                    "row" => 391,
                                                                    "col" => 6,
                                                                    "children" => (
                                                                        value!([
                                                                            (value!([
                                                                                "emit" => "identifier",
                                                                                "row" => 391,
                                                                                "col" => 6,
                                                                                "value" => "SequenceItem"
                                                                            ])),
                                                                            (value!([
                                                                                "emit" => "op_mod_opt",
                                                                                "row" => 391,
                                                                                "col" => 19,
                                                                                "children" => (
                                                                                    (value!([
                                                                                        "emit" => "inline_sequence",
                                                                                        "row" => 391,
                                                                                        "col" => 20,
                                                                                        "children" => (
                                                                                            value!([
                                                                                                (value!([
                                                                                                    "emit" => "value_token_touch",
                                                                                                    "row" => 391,
                                                                                                    "col" => 20,
                                                                                                    "value" => ","
                                                                                                ])),
                                                                                                (value!([
                                                                                                    "emit" => "identifier",
                                                                                                    "row" => 391,
                                                                                                    "col" => 24,
                                                                                                    "value" => "_"
                                                                                                ]))
                                                                                            ])
                                                                                        )
                                                                                    ]))
                                                                                )
                                                                            ]))
                                                                        ])
                                                                    )
                                                                ]))
                                                            )
                                                        ])),
                                                        (value!([
                                                            "emit" => "call",
                                                            "row" => 391,
                                                            "col" => 31,
                                                            "children" => (
                                                                value!([
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 391,
                                                                        "col" => 31,
                                                                        "value" => "ast"
                                                                    ])),
                                                                    (value!([
                                                                        "emit" => "callarg",
                                                                        "row" => 391,
                                                                        "col" => 35,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "value_string",
                                                                                "row" => 391,
                                                                                "col" => 35,
                                                                                "value" => "sequence"
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                ])
                                                            )
                                                        ]))
                                                    ])
                                                )
                                            ]))
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 394,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 394,
                                "col" => 1,
                                "value" => "Sequences"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 394,
                                "col" => 13,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 394,
                                        "col" => 14,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 395,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 395,
                                                                "col" => 5,
                                                                "value" => "Sequence"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_pos",
                                                                "row" => 395,
                                                                "col" => 14,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "block",
                                                                        "row" => 395,
                                                                        "col" => 14,
                                                                        "children" => (
                                                                            (value!([
                                                                                "emit" => "sequence",
                                                                                "row" => 395,
                                                                                "col" => 15,
                                                                                "children" => (
                                                                                    value!([
                                                                                        (value!([
                                                                                            "emit" => "value_token_touch",
                                                                                            "row" => 395,
                                                                                            "col" => 15,
                                                                                            "value" => "|"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 395,
                                                                                            "col" => 19,
                                                                                            "value" => "_"
                                                                                        ])),
                                                                                        (value!([
                                                                                            "emit" => "identifier",
                                                                                            "row" => 395,
                                                                                            "col" => 21,
                                                                                            "value" => "Sequence"
                                                                                        ]))
                                                                                    ])
                                                                                )
                                                                            ]))
                                                                        )
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 395,
                                                                "col" => 33,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 395,
                                                                            "col" => 33,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 395,
                                                                            "col" => 37,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 395,
                                                                                    "col" => 37,
                                                                                    "value" => "block"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 396,
                                                    "col" => 5,
                                                    "value" => "Sequence"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 399,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 399,
                                "col" => 1,
                                "value" => "Instruction"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 399,
                                "col" => 15,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 399,
                                        "col" => 16,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 400,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 400,
                                                                "col" => 5,
                                                                "value" => "begin"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 400,
                                                                "col" => 13,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 400,
                                                                "col" => 34,
                                                                "value" => "Sequences"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 400,
                                                                "col" => 44,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 400,
                                                                        "col" => 51,
                                                                        "value" => "T_EOL"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 400,
                                                                "col" => 58,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 400,
                                                                            "col" => 58,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 400,
                                                                            "col" => 62,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 400,
                                                                                    "col" => 62,
                                                                                    "value" => "begin"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 401,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 401,
                                                                "col" => 5,
                                                                "value" => "end"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 401,
                                                                "col" => 11,
                                                                "value" => "_SeparatedIdentifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 401,
                                                                "col" => 32,
                                                                "value" => "Sequences"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 401,
                                                                "col" => 42,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 401,
                                                                        "col" => 49,
                                                                        "value" => "T_EOL"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 401,
                                                                "col" => 56,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 401,
                                                                            "col" => 56,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 401,
                                                                            "col" => 60,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 401,
                                                                                    "col" => 60,
                                                                                    "value" => "end"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 402,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 402,
                                                                "col" => 5,
                                                                "value" => "T_Identifier"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 402,
                                                                "col" => 18,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "value_token_touch",
                                                                "row" => 402,
                                                                "col" => 20,
                                                                "value" => ":"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 402,
                                                                "col" => 24,
                                                                "value" => "_"
                                                            ])),
                                                            (value!([
                                                                "emit" => "block",
                                                                "row" => 402,
                                                                "col" => 26,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "sequence",
                                                                            "row" => 403,
                                                                            "col" => 9,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 403,
                                                                                        "col" => 9,
                                                                                        "value" => "Literal"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 403,
                                                                                        "col" => 17,
                                                                                        "value" => "_"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "op_mod_peek",
                                                                                        "row" => 403,
                                                                                        "col" => 19,
                                                                                        "children" => (
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 403,
                                                                                                "col" => 24,
                                                                                                "value" => "T_EOL"
                                                                                            ]))
                                                                                        )
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "sequence",
                                                                            "row" => 404,
                                                                            "col" => 9,
                                                                            "children" => (
                                                                                value!([
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 404,
                                                                                        "col" => 9,
                                                                                        "value" => "Token1"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "identifier",
                                                                                        "row" => 404,
                                                                                        "col" => 16,
                                                                                        "value" => "_"
                                                                                    ])),
                                                                                    (value!([
                                                                                        "emit" => "op_mod_peek",
                                                                                        "row" => 404,
                                                                                        "col" => 18,
                                                                                        "children" => (
                                                                                            (value!([
                                                                                                "emit" => "identifier",
                                                                                                "row" => 404,
                                                                                                "col" => 23,
                                                                                                "value" => "T_EOL"
                                                                                            ]))
                                                                                        )
                                                                                    ]))
                                                                                ])
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 405,
                                                                            "col" => 9,
                                                                            "value" => "Sequences"
                                                                        ]))
                                                                    ])
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 406,
                                                                "col" => 7,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 406,
                                                                        "col" => 14,
                                                                        "value" => "T_EOL"
                                                                    ]))
                                                                )
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 406,
                                                                "col" => 21,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 406,
                                                                            "col" => 21,
                                                                            "value" => "ast"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 406,
                                                                            "col" => 25,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 406,
                                                                                    "col" => 25,
                                                                                    "value" => "constant"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 407,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 407,
                                                                "col" => 5,
                                                                "value" => "Statement"
                                                            ])),
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 407,
                                                                "col" => 15,
                                                                "value" => "T_EOL"
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 408,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "identifier",
                                                                "row" => 408,
                                                                "col" => 5,
                                                                "value" => "Sequences"
                                                            ])),
                                                            (value!([
                                                                "emit" => "op_mod_expect",
                                                                "row" => 408,
                                                                "col" => 15,
                                                                "children" => (
                                                                    (value!([
                                                                        "emit" => "identifier",
                                                                        "row" => 408,
                                                                        "col" => 22,
                                                                        "value" => "T_EOL"
                                                                    ]))
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "identifier",
                                                    "row" => 409,
                                                    "col" => 5,
                                                    "value" => "T_EOL"
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 412,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 412,
                                "col" => 1,
                                "value" => "Nop"
                            ])),
                            (value!([
                                "emit" => "sequence",
                                "row" => 412,
                                "col" => 7,
                                "children" => (
                                    value!([
                                        (value!([
                                            "emit" => "identifier",
                                            "row" => 412,
                                            "col" => 7,
                                            "value" => "Void"
                                        ])),
                                        (value!([
                                            "emit" => "call",
                                            "row" => 412,
                                            "col" => 12,
                                            "children" => (
                                                value!([
                                                    (value!([
                                                        "emit" => "identifier",
                                                        "row" => 412,
                                                        "col" => 12,
                                                        "value" => "ast"
                                                    ])),
                                                    (value!([
                                                        "emit" => "callarg",
                                                        "row" => 412,
                                                        "col" => 16,
                                                        "children" => (
                                                            (value!([
                                                                "emit" => "value_string",
                                                                "row" => 412,
                                                                "col" => 16,
                                                                "value" => "op_nop"
                                                            ]))
                                                        )
                                                    ]))
                                                ])
                                            )
                                        ]))
                                    ])
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "constant",
                    "row" => 416,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 416,
                                "col" => 1,
                                "value" => "Tokay"
                            ])),
                            (value!([
                                "emit" => "value_parselet",
                                "row" => 416,
                                "col" => 9,
                                "children" => (
                                    (value!([
                                        "emit" => "block",
                                        "row" => 416,
                                        "col" => 10,
                                        "children" => (
                                            value!([
                                                (value!([
                                                    "emit" => "op_mod_pos",
                                                    "row" => 417,
                                                    "col" => 5,
                                                    "children" => (
                                                        (value!([
                                                            "emit" => "identifier",
                                                            "row" => 417,
                                                            "col" => 5,
                                                            "value" => "Instruction"
                                                        ]))
                                                    )
                                                ])),
                                                (value!([
                                                    "emit" => "sequence",
                                                    "row" => 418,
                                                    "col" => 5,
                                                    "children" => (
                                                        value!([
                                                            (value!([
                                                                "emit" => "value_token_any",
                                                                "row" => 418,
                                                                "col" => 5,
                                                                "value" => "."
                                                            ])),
                                                            (value!([
                                                                "emit" => "call",
                                                                "row" => 418,
                                                                "col" => 8,
                                                                "children" => (
                                                                    value!([
                                                                        (value!([
                                                                            "emit" => "identifier",
                                                                            "row" => 418,
                                                                            "col" => 8,
                                                                            "value" => "error"
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 418,
                                                                            "col" => 14,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_string",
                                                                                    "row" => 418,
                                                                                    "col" => 14,
                                                                                    "value" => "Parse error, unexpected token"
                                                                                ]))
                                                                            )
                                                                        ])),
                                                                        (value!([
                                                                            "emit" => "callarg",
                                                                            "row" => 418,
                                                                            "col" => 47,
                                                                            "children" => (
                                                                                (value!([
                                                                                    "emit" => "value_true",
                                                                                    "row" => 418,
                                                                                    "col" => 47,
                                                                                    "value" => "true"
                                                                                ]))
                                                                            )
                                                                        ]))
                                                                    ])
                                                                )
                                                            ]))
                                                        ])
                                                    )
                                                ]))
                                            ])
                                        )
                                    ]))
                                )
                            ]))
                        ])
                    )
                ])),
                (value!([
                    "emit" => "sequence",
                    "row" => 421,
                    "col" => 1,
                    "children" => (
                        value!([
                            (value!([
                                "emit" => "identifier",
                                "row" => 421,
                                "col" => 1,
                                "value" => "_"
                            ])),
                            (value!([
                                "emit" => "op_mod_opt",
                                "row" => 421,
                                "col" => 3,
                                "children" => (
                                    (value!([
                                        "emit" => "identifier",
                                        "row" => 421,
                                        "col" => 3,
                                        "value" => "Tokay"
                                    ]))
                                )
                            ])),
                            (value!([
                                "emit" => "op_mod_expect",
                                "row" => 421,
                                "col" => 10,
                                "children" => (
                                    (value!([
                                        "emit" => "identifier",
                                        "row" => 421,
                                        "col" => 17,
                                        "value" => "EOF"
                                    ]))
                                )
                            ])),
                            (value!([
                                "emit" => "call",
                                "row" => 421,
                                "col" => 22,
                                "children" => (
                                    value!([
                                        (value!([
                                            "emit" => "identifier",
                                            "row" => 421,
                                            "col" => 22,
                                            "value" => "ast"
                                        ])),
                                        (value!([
                                            "emit" => "callarg",
                                            "row" => 421,
                                            "col" => 26,
                                            "children" => (
                                                (value!([
                                                    "emit" => "value_string",
                                                    "row" => 421,
                                                    "col" => 26,
                                                    "value" => "main"
                                                ]))
                                            )
                                        ]))
                                    ])
                                )
                            ]))
                        ])
                    )
                ]))
            ])
        )
    ]))
    /*ETARENEG*/
}
