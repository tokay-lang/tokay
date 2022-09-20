/*! Parser and grammar for Tokay, implemented using Tokay itself.

This module implements a Tokay parser in Tokay itself, using the compiler macros from the macros-module.
This is the general place to change syntax and modify the design of the abstract syntax tree.

There is also a grammar in examples/tokay.tok, which is this grammar expressed in Tokay.
When changing things here, do it there as well as this might become the reference parser someday.
*/
use charclass::charclass;

use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::tokay;
use crate::value::{Dict, RefValue, Str, Token};

pub struct Parser(Program);

impl Parser {
    pub fn new() -> Self {
        Self(tokay!({

        // ----------------------------------------------------------------------------

        // Whitespace & EOL

        (_ = {  // true whitespace is made of comments and escaped line-breaks as well
            [(token (Token::Chars(charclass![' ', '\t'])))],
            ["#", (opt (token (Token::Chars(charclass!['\n'].negate()))))],
            ["\\", (opt "\r"), "\n"]
        }),

        (___ = {  // optional line-breaks followed by whitespace
            (kle [T_EOL, _])
        }),

        (_SeparatedIdentifier = {  // helper parselet to ensure that identifiers are separated
            [(peek (not (token (Token::Char(charclass!['A' => 'Z', 'a' => 'z'] + charclass!['_']))))), _]
        }),

        (T_EOL = {  // end-of-line
            ["\n", _],  // unix/linux
            ["\r", (opt "\n"), _],  // classic mac & windows
            [";", _],  // allows for multiple lines in one row
            [(peek (token (Token::EOF))), (Op::Accept)],  // Input file may end without a new line
            (peek "}")  // peek for '}' to allow for blocks in one line.
        }),

        // Escape sequences

        (T_OctDigit = {  // T_OctDigit is used by T_EscapeSequence
            (token (Token::Char(charclass!['0' => '7'])))
        }),

        (T_HexDigit = {    // T_HexDigit is used by T_EscapeSequence
            (token (Token::Char(charclass!['0' => '9', 'A' => 'F', 'a' => 'f'])))
        }),

        (T_EscapeSequence = {   // Parsing escape sequences
            ["a", (value "\x07")],
            ["b", (value "\x08")],
            ["f", (value "\x0c")],
            ["n", (value "\n")],
            ["r", (value "\r")],
            ["t", (value "\t")],
            ["v", (value "\x0b")],
            [T_OctDigit, T_OctDigit, T_OctDigit,
                (Op::Rust(Rust(|context| code_to_char(context, 0, 8))))],
            ["x", T_HexDigit, T_HexDigit,
                (Op::Rust(Rust(|context| code_to_char(context, 1, 16))))],
            ["u", T_HexDigit, T_HexDigit, T_HexDigit, T_HexDigit,
                (Op::Rust(Rust(|context| code_to_char(context, 1, 16))))],
            ["U", T_HexDigit, T_HexDigit, T_HexDigit, T_HexDigit,
                T_HexDigit, T_HexDigit, T_HexDigit, T_HexDigit,
                (Op::Rust(Rust(|context| code_to_char(context, 1, 16))))],
            Any
        }),

        // Prime Tokens

        (T_Identifier = {  // any identifier
            (call ast[(value "identifier"), (call Ident[])])
        }),

        (T_Consumable = {  // consumable identifier
            [
                (token (Token::Char(charclass!['A' => 'Z'] + charclass!['_']))),
                (opt (token (Token::Chars(charclass!['A' => 'Z', 'a' => 'z', '0' => '9'] + charclass!['_'])))),
                (call ast[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_Alias = {  // T_Alias is an identifier treated as string value
            [
                (token (Token::Char(charclass!['A' => 'Z', 'a' => 'z'] + charclass!['_']))),
                (opt (token (Token::Chars(charclass!['A' => 'Z', 'a' => 'z', '0' => '9'] + charclass!['_'])))),
                (call ast[(value "value_string"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_String = {
            [
                "\"",  // a string
                (kle {
                    ["\\", T_EscapeSequence],
                    [(token (Token::Chars(charclass!['\\', '\"'].negate())))],
                    [EOF, (call error[(value "Unclosed string, expecting '\"'")])]
                }),
                (call str_join[(value ""), (Op::LoadFastCapture(2))]),
                (expect "\"")
            ]
        }),

        (T_Touch = {
            [
                "\'",  // a touch
                (kle {
                    ["\\", T_EscapeSequence],
                    [(token (Token::Chars(charclass!['\\', '\''].negate())))],
                    [EOF, (call error[(value "Unclosed match, expecting '\''")])]
                }),
                (call str_join[(value ""), (Op::LoadFastCapture(2))]),
                (expect "\'")
            ]
        }),

        (T_Integer = {
            [(call ast[(value "value_integer"), (call Int[])])]
        }),

        (T_Float = {
            [(call ast[(value "value_float"), (call Float[])])]
        }),

        // Character classes

        (CclChar = {
            ["\\", T_EscapeSequence],
            (token (Token::Char(charclass![']'].negate()))),
            [EOF, (call error[(value "Unclosed character-class, expecting ']'")])]
        }),

        (CclRange = {
            [CclChar, "-", CclChar,
                (call ast[
                    (value "range"),
                    [
                        (Op::LoadFastCapture(1)),
                        (Op::LoadFastCapture(3)),
                        (Op::BinaryOp("add"))
                    ]
                ])
            ],
            [CclChar, (call ast[(value "char")])]
        }),

        (Ccl = {
            ['^', (kle CclRange), (call ast[(value "ccl_neg")])],
            [(kle CclRange), (call ast[(value "ccl")])]
        }),

        // Statics, Variables, Loads

        (Subscript = {
            ["[", _, Expression, "]", _, (call ast[(value "index")])]
        }),

        (Attribute = {
            [".", _, (expect T_Alias), (call ast[(value "attribute")])]
        }),

        (Capture = {
            ["$", T_Alias, _, (call ast[(value "capture_alias")])],
            ["$", T_Integer, _, (call ast[(value "capture_index")])],
            ["$", "(", _, ___, Expression, ")", _, (call ast[(value "capture_expr")])],
            ["$", (call error[(value "'$...': Expecting identifier, integer or (expression)")])]
        }),

        (Variable = {
            T_Identifier,
            Capture
        }),

        (Lvalue = {
            [Variable, _, (kle {
                // Attribute,  // Attribute assignment not required for now.
                Subscript
            }), (call ast[(value "lvalue")])]
        }),

        (Load = {
            [Lvalue, "++", (call ast[(value "inplace_post_inc")])],
            [Lvalue, "--", (call ast[(value "inplace_post_dec")])],
            ["++", (expect Lvalue), (call ast[(value "inplace_pre_inc")])],
            ["--", (expect Lvalue), (call ast[(value "inplace_pre_dec")])],
            Variable
        }),

        // Parselet

        (InlineParselet = {
            // Inline parselet requires for an explicit block instead of an expression
            ["@", _, (opt ParseletGenerics), _, (opt ParseletArguments), (expect Block),
                (call ast[(value "value_parselet")])]

        }),

        (Parselet = {
            ["@", _, (opt ParseletGenerics), _, (opt ParseletArguments), (expect Expression),
                (call ast[(value "value_parselet")])]
        }),

        // Parselet: Generics

        (ParseletGeneric = {
            [T_Identifier, _, (opt [':', _, (expect Atomic)]), (call ast[(value "gen")])]
        }),

        (ParseletGenerics = {
            ["<", _, (kle [ParseletGeneric, _, (opt [',', _])]), (expect ">"), _]
        }),

        // Parselet: Arguments

        (ParseletArgument = {
            [T_Identifier, _, (opt ["=", _, (expect Expression)]), (call ast[(value "arg")])]
        }),

        (ParseletArguments = {
            (pos [ParseletArgument, (opt [",", _])])
        }),

        // Parselet: Instance

        (StaticParseletInstance = {
            T_Consumable,
            InlineParselet
        }),

        (ParseletInstanceArgument = {
            [T_Identifier, _, ":", _, (expect Atomic), _, (call ast[(value "genarg_named")])],
            [Atomic, _, (call ast[(value "genarg")])]
        }),

        (ParseletInstance = {
            [StaticParseletInstance, "<", _, (pos [
                ParseletInstanceArgument, (opt [",", _])
            ]), _, (expect ">"), _,  (call ast[(value "value_generic")])],
            StaticParseletInstance
        }),

        // Inline Blocks and Sequences

        (InlineSequenceItem = {
            [T_Alias, _, "=>", _, (expect Expression), (call ast[(value "alias")])],
            [Expression, "=>", _, (expect Expression), (call ast[(value "alias")])],
            Expression
        }),

        (InlineSequence = {
            // Special case: Expression followed by "," is considered as a list with a single item (syntactic sugar)
            [Expression, ___, ",", _, ___, (peek ")"), (call ast[(value "list")])],
            // A sequence is a list of items optionally separated by ","
            [(pos [InlineSequenceItem, ___, (opt [",", _]), ___]), (call ast[(value "inline_sequence")])],
            // The empty sequences generates an empty list
            [Void, (call ast[(value "list")])]
        }),

        (InlineBlock = {
            // Multiple sequences delimited by "|" are an alternative form of the block syntax
            ["(", _, ___, InlineSequence,
                (pos [___, "|", _, ___, InlineSequence]), (expect ")"),
                    (call ast[(value "block")])],
            // In case there's only a single sequence, handle it just as a sequence without a block
            ["(", _, ___, InlineSequence, ___, (expect ")")]
        }),

        // Call

        (CallArgument = {
            [T_Identifier, _, "=", _, (expect Expression), (call ast[(value "callarg_named")])],
            [Expression, (call ast[(value "callarg")])]
        }),

        (CallArguments = {
            (pos [CallArgument, (opt [",", _]), ___])
        }),

        // Tokens

        (TokenLiteral = {
            ["'", T_Touch, "'", (call ast[(value "value_token_match")])],
            [T_Touch, (call ast[(value "value_token_touch")])],
            [".", (call ast[(value "value_token_any")])],
            ['[', Ccl, ']', (call ast[(value "value_token_ccl")])]
        }),

        (TokenAtom = {
            TokenLiteral,
            [ParseletInstance, "(", _, ___, (opt CallArguments), ___, (expect ")"),
                (call ast[(value "call")])],
            ParseletInstance,
            InlineBlock,
            Block
        }),

        (Token = {
            // Token call modifiers
            [TokenAtom, "+", (call ast[(value "op_mod_pos")])],
            [TokenAtom, "*", (call ast[(value "op_mod_kle")])],
            [TokenAtom, "?", (call ast[(value "op_mod_opt")])],
            // todo: {min}, {min, max} maybe with expression?
            TokenAtom,
            ["peek", _SeparatedIdentifier, (expect Token), (call ast[(value "op_mod_peek")])],
            ["not", _SeparatedIdentifier, (expect Token), (call ast[(value "op_mod_not")])],
            ["expect", _SeparatedIdentifier, (expect Token), (call ast[(value "op_mod_expect")])]
        }),

        // Literals

        (Literal = {
            /* below calls to _SeparatedIdentifier avoid to wrongly interpret e.g. "truex" as "true" and "x" */
            ["true", _SeparatedIdentifier, (call ast[(value "value_true")])],
            ["false", _SeparatedIdentifier, (call ast[(value "value_false")])],
            ["void", _SeparatedIdentifier, (call ast[(value "value_void")])],
            ["null", _SeparatedIdentifier, (call ast[(value "value_null")])],
            [T_String, (call ast[(value "value_string")])],
            T_Float,
            T_Integer
        }),

        // Expression & Flow

        (Atomic = {
            ["(", _, ___, Expression, ___, ")"], // no expect ")" here!
            Literal,
            Token,

            // if
            ["if", _SeparatedIdentifier, Expression, ___, (expect Statement),
                (opt [___, "else", _SeparatedIdentifier, ___, (expect Statement)]),
                    (call ast[(value "op_if")])],

            // for
            //["for", _SeparatedIdentifier, T_Identifier, _, "in", _SeparatedIdentifier, Expression, Statement,
            //    (call ast[(value "op_for_in")])],
            ["for", _SeparatedIdentifier, StatementOrEmpty, ";", _, StatementOrEmpty, ";", _, StatementOrEmpty,
                StatementOrEmpty, (call ast[(value "op_for")])],
            ["for", _SeparatedIdentifier, (call error[(value "'for': Expecting start; condition; iter; statement")])],

            // loop
            ["loop", _SeparatedIdentifier, Expression, _, Statement, (call ast[(value "op_loop")])],
            ["loop", _SeparatedIdentifier, (expect Statement), (call ast[(value "op_loop")])],

            // standard load
            Load
        }),

        (Rvalue = {
            [Rvalue, "(", _, ___, (opt CallArguments), (expect ")"),
                (call ast[(value "call")])],
            [Rvalue, (kle {
                Attribute,
                Subscript
            }), (call ast[(value "rvalue")])],
            Atomic
        }),

        (Unary = {
            ["-", (not "-"), _, Unary, (call ast[(value "op_unary_neg")])],
            ["!", _, Unary, (call ast[(value "op_unary_not")])],
            [Rvalue, _]
        }),

        (MulDiv = {
            [MulDiv, "*", _, (expect Unary), (call ast[(value "op_binary_mul")])],
            [MulDiv, "/", _, (expect Unary), (call ast[(value "op_binary_div")])],
            Unary
        }),

        (AddSub = {
            [AddSub, "+", (not "+"), _, (expect MulDiv), // avoid matching "++"
                (call ast[(value "op_binary_add")])],
            [AddSub, "-", (not "-"), _, (expect MulDiv), // avoid matching "--"
                (call ast[(value "op_binary_sub")])],
            MulDiv
        }),

        (Compare = {
            [Compare, "==", _, (expect AddSub), (call ast[(value "op_compare_eq")])],
            [Compare, "!=", _, (expect AddSub), (call ast[(value "op_compare_neq")])],
            [Compare, "<=", _, (expect AddSub), (call ast[(value "op_compare_lteq")])],
            [Compare, ">=", _, (expect AddSub), (call ast[(value "op_compare_gteq")])],
            [Compare, "<", _, (expect AddSub), (call ast[(value "op_compare_lt")])],
            [Compare, ">", _, (expect AddSub), (call ast[(value "op_compare_gt")])],
            AddSub
        }),

        (LogicalAnd = {
            [LogicalAnd, "&&", _, (expect Compare), (call ast[(value "op_logical_and")])],
            Compare
        }),

        (LogicalOr = {
            [LogicalOr, "||", _, (expect LogicalAnd), (call ast[(value "op_logical_or")])],
            LogicalAnd
        }),

        (Expression = {

            // assignment
            [Lvalue, _, "=", (not {">", "="}), //avoid wrongly matching "=>" or "=="
                _, (expect Expression), (call ast[(value "assign_hold")])],
            [Lvalue, _, "+=", _, (expect Expression), (call ast[(value "assign_add_hold")])],
            [Lvalue, _, "-=", _, (expect Expression), (call ast[(value "assign_sub_hold")])],
            [Lvalue, _, "*=", _, (expect Expression), (call ast[(value "assign_mul_hold")])],
            [Lvalue, _, "/=", _, (expect Expression), (call ast[(value "assign_div_hold")])],

            // normal expression starting with LogicalOr
            LogicalOr
        }),

        // Statement and Assignment

        (StatementOrEmpty = {
            Statement,
            (call ast[(value "op_nop")])
        }),

        (Statement = {
            ["accept", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_accept")])],
            ["break", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_break")])],
            ["continue", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_continue")])],
            ["exit", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_exit")])],
            ["next", _SeparatedIdentifier, (call ast[(value "op_next")])],
            ["push", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_push")])],
            ["reject", _SeparatedIdentifier, (call ast[(value "op_reject")])],
            ["repeat", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_repeat")])],
            ["return", _SeparatedIdentifier, (opt Expression), (call ast[(value "op_accept")])],
            // todo: escape?

            [Lvalue, _, "=", (not {">", "="}), //avoid wrongly matching "=>" or "==" here
                _, (expect Expression), (call ast[(value "assign")])],
            [Lvalue, _, "+=", _, (expect Expression), (call ast[(value "assign_add")])],
            [Lvalue, _, "-=", _, (expect Expression), (call ast[(value "assign_sub")])],
            [Lvalue, _, "*=", _, (expect Expression), (call ast[(value "assign_mul")])],
            [Lvalue, _, "/=", _, (expect Expression), (call ast[(value "assign_div")])],

            Expression
        }),

        // Blocks and Sequences

        (Block = {
            ["{", _, ___, "}", (call ast[(value "value_void")])],
            ["{", _, (kle Instruction), _, (expect "}"), (call ast[(value "block")])]
        }),

        (SequenceItem = {
            [T_Alias, _, "=>", _, (expect Expression), (call ast[(value "alias")])],
            [Expression, "=>", _, (expect Expression), (call ast[(value "alias")])],
            Statement
        }),

        (Sequence = {
            [(pos [SequenceItem, (opt [",", _])]), (call ast[(value "sequence")])]
        }),

        (Sequences = {
            [Sequence, (pos ["|", _, Sequence]), (call ast[(value "block")])],
            Sequence
        }),

        (Instruction = {
            ["begin", _SeparatedIdentifier, Sequence, (expect T_EOL), (call ast[(value "begin")])],
            ["end", _SeparatedIdentifier, Sequence, (expect T_EOL), (call ast[(value "end")])],
            [T_Identifier, _, ":", _, {
                [Statement, (peek T_EOL)],
                Sequences
            }, (expect T_EOL), (call ast[(value "constant")])],
            [Sequences, (expect T_EOL)],
            T_EOL
        }),

        (Tokay = {
            (pos Instruction),
            [(token (Token::any())),
                (call error[(value "Parse error, unexpected token"), (value true)])]
        }),

        [_, (opt Tokay),
            (expect (token (Token::EOF)), "Parse error, expecting end-of-file"),
            (call ast[(value "main")])]

        // --- Test Environment -----------------------------------------------

        /*
        (T_Float = {
            [(token (Token::Chars(charclass!['0' => '9']))), ".", (opt (token (Token::Chars(charclass!['0' => '9'])))),
                (call ast[(value "value_float"), (Op::LoadFastCapture(0))])],
            [(opt (token (Token::Chars(charclass!['0' => '9'])))), ".", (token (Token::Chars(charclass!['0' => '9']))),
                (call ast[(value "value_float"), (Op::LoadFastCapture(0))])]
        }),

        T_Float

        (X = {
            [Y, (MATCH "c")]
        }),
        (Y = {
            [Z, (MATCH "b")]
        }),
        (Z = {
            X,
            Y,
            (MATCH "a")
        }),
        Z

        (A = {
            (MATCH 'x'),
            Void
        }),
        A
        */

        // ----------------------------------------------------------------------------
        }))
    }

    pub fn parse(&self, mut reader: Reader) -> Result<RefValue, Error> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        if let Ok(level) = std::env::var("TOKAY_PARSER_DEBUG") {
            runtime.debug = level.parse::<u8>().unwrap_or_default();
        } else {
            runtime.debug = 0;
        }

        match self.0.run(&mut runtime) {
            Ok(Some(ast)) => {
                if ast.borrow().object::<Dict>().is_some() {
                    Ok(ast)
                } else {
                    Err(Error::new(None, "Parse error".to_string()))
                }
            }
            Ok(None) => Ok(crate::value!(void)),
            Err(error) => Err(error),
        }
    }
}

fn code_to_char(context: &mut Context, skip: u8, base: u32) -> Result<Accept, Reject> {
    let value = context.get_capture(0).unwrap();
    let value = value.borrow();
    let slice = &value.object::<Str>().unwrap().as_str()[skip as usize..];

    let code = if slice.len() <= 2 {
        u8::from_str_radix(slice, base).unwrap_or_default() as char
    } else {
        std::char::from_u32(u32::from_str_radix(slice, base).unwrap_or_default())
            .unwrap_or_default()
    };

    Ok(Accept::Return(Some(RefValue::from(format!("{}", code)))))
}

#[test]
// EOL
fn parser_eol() {
    for eol in ["\n", "\r", "\r\n", ";"] {
        let tok = format!("a = 1{}a + 2", eol);
        println!("EOL test {:?}", tok);
        assert_eq!(crate::run(&tok, ""), Ok(Some(crate::value!(3))));
    }
}

// Tests for parsing and packrat features ---------------------------------------------------------

/*
    Below are some tests that provide indirect left-recursion.

    They currently don't work properly due to the following reason:
    For indirect left-recursion in packrat parsing, one rule in the
    grammar's graph must be declared as "leading", so that subsequent,
    even left-recursive parselets are considered as not left-recursive.


    An implementation of an solution for this issue can be found in
    the pegen parser generator from Python:

    https://github.com/python/cpython/blob/main/Tools/peg_generator/pegen/parser_generator.py

    Tokay won't take care of this right now as it is an edge-case
    and also more complex, as Tokay does not directly implements a
    grammar.
*/

#[test]
fn parser_indirectleftrec() {
    /*
        X: Y 'c'
        Y: Z 'b'
        Z: X | Y | 'a'
        Z
    */

    let program = tokay!({
        (X = {
            [Y, (MATCH "c")]
        }),
        (Y = {
            [Z, (MATCH "b")]
            //Void
        }),
        (Z = {
            X,
            Y,
            (MATCH "a")
        }),
        Z
    });

    println!("{:#?}", program.run_from_str("aaabc"));
}

#[test]
fn parser_leftrec() {
    /*
    let program = tokay!({
        (X = {
            [X, (MATCH "b")],
            (MATCH "a")
        }),

        X
    });
    */

    let program = tokay!({
        (Y = {
            X,
            (MATCH "a")
        }),
        (X = {
            [Y, (MATCH "b")]
        }),
        X
    });

    /*
    let program = tokay!({
        (Factor = {
            ["(", (pos [Expression]), ")"],
            (token (Token::Chars(charclass!['0'..='9'])))
        }),
        (Expression = {
            [Expression, "+", Expression],
            Factor
        }),
        Expression
    });
    */

    println!("{:#?}", program.run_from_str("abb"));
}
