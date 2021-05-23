use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::token::Token;
use crate::value::Value;
use crate::vm::*;
use crate::{ccl, value};
use crate::{tokay_embed, tokay_embed_item};

/**
Implements a Tokay parser in Tokay itself, using the compiler macros from the macros-module.
This is the general place to change syntax and modify the design of the abstract syntax tree.
*/

pub struct Parser(Program);

impl Parser {
    pub fn new() -> Self {
        Self(tokay_embed!({
        // ----------------------------------------------------------------------------

        // Whitespace & EOL

        (_ = {
            [" "],
            ["#", (token (Token::chars_until('\n')))],
            ["\\", "\n"]
        }),

        (T_EOL = {
            ["\n", _, (Op::Skip)],
            [";", _, (Op::Skip)]
        }),

        // Prime Tokens (might probably be replaced by something native, pluggable one)

        (T_Identifier = {
            [
                (token (Token::Char(ccl!['A'..='Z', 'a'..='z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call collect[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_Consumable = {
            [
                (token (Token::Char(ccl!['A'..='Z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call collect[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_String = {
            [
                "\"",
                (token (Token::chars_until('\"'))),     //fixme: Escape sequences (using Until built-in parselet)
                "\""
            ]
        }),

        (T_Match = {
            [
                "\'",
                (token (Token::chars_until('\''))),    //fixme: Escape sequences (using Until built-in parselet)
                "\'"
            ]
        }),

        (T_Integer = {
            // todo: implement as built-in Parselet
            [(token (Token::Chars(ccl!['0'..='9']))), (call collect[(value "value_integer")])]
        }),

        (T_Float = {
            // todo: implement as built-in Parselet
            [(token (Token::Chars(ccl!['0'..='9']))), ".", (opt (token (Token::Chars(ccl!['0'..='9'])))),
                (call collect[(value "value_float"), (Op::LoadFastCapture(0))])],
            [(opt (token (Token::Chars(ccl!['0'..='9'])))), ".", (token (Token::Chars(ccl!['0'..='9']))),
                (call collect[(value "value_float"), (Op::LoadFastCapture(0))])]
        }),

        // Character classes

        (CclChar = {
            [EOF, (call error[(value "Unclosed character-class, expecting ']'")])],
            ["\\", (token (Token::Any))],
            (token (Token::char_except(']')))
        }),

        (CclRange = {
            [CclChar, "-", CclChar,
                (call collect[(value "range"), [(Op::LoadFastCapture(1)), (Op::LoadFastCapture(3)), (Op::Add)]])],
            [CclChar, (call collect[(value "char")])]
        }),

        (Ccl = {
            ['^', (kle CclRange), (call collect[(value "ccl_neg")])],
            [(kle CclRange), (call collect[(value "ccl")])]
        }),

        // Statics, Variables & Constants

        (Tail = {
            [".", _, T_Identifier, _, (call collect[(value "attribute")])],
            ["[", _, Expression, "]", (call collect[(value "index")])]
        }),

        (Capture = {
            ["$", T_Identifier, _, (call collect[(value "capture_alias")])],
            ["$", T_Integer, _, (call collect[(value "capture_index")])],
            ["$", "(", _, Expression, ")", _, (call collect[(value "capture_expr")])],
            ["$", (call error[(value "'$': Expecting identifier, integer or (expression)")])]
        }),

        (Variable = {
            T_Identifier,
            Capture
        }),

        (Lvalue = {
            [Variable, (kle Tail), _, (call collect[(value "lvalue")])]
        }),

        (Inplace = {
            [Lvalue, "++", (call collect[(value "inplace_post_inc")])],
            [Lvalue, "--", (call collect[(value "inplace_post_dec")])],
            ["++", (expect Lvalue), (call collect[(value "inplace_pre_inc")])],
            ["--", (expect Lvalue), (call collect[(value "inplace_pre_dec")])],
            Variable
        }),

        (Rvalue = {
            [Inplace, (kle Tail), _, (call collect[(value "rvalue")])]
        }),

        (CallParameter = {
            [T_Identifier, _, "=", _, Expression, (call collect[(value "param_named")])],
            [Expression, (call collect[(value "param")])]
        }),

        (CallParameters = {
            (pos [CallParameter, (opt [",", _])])
        }),

        (Call = {
            [T_Identifier, "(", _, (opt CallParameters), (expect ")"), _,
                (call collect[(value "call_identifier")])]
            //[Rvalue, "(", _, (opt Parameters), ")", _, (call collect[(value "call_rvalue")])]
        }),

        (Literal = {
            ["true", _, (call collect[(value "value_true")])],
            ["false", _, (call collect[(value "value_false")])],
            ["void", _, (call collect[(value "value_void")])],
            ["null", _, (call collect[(value "value_null")])],
            [T_String, _, (call collect[(value "value_string")])],
            [T_Float, _],
            [T_Integer, _]
        }),

        // Tokens

        (TokenLiteral = {
            ["'", T_Match, "'", (call collect[(value "value_token_match")])],
            [T_Match, (call collect[(value "value_token_touch")])],
            [".", (call collect[(value "value_token_any")])],
            ['[', Ccl, ']', (call collect[(value "value_token_ccl")])]
        }),

        (TokenCall = {
            TokenLiteral,
            [T_Consumable, "(", _, (opt CallParameters), (expect ")"),
                (call collect[(value "call_identifier")])],
            [T_Consumable, (call collect[(value "rvalue")])]
        }),

        (Token = {
            // Token call modifiers
            [TokenCall, "+", _, (call collect[(value "op_mod_pos")])],
            [TokenCall, "*", _, (call collect[(value "op_mod_kle")])],
            [TokenCall, "?", _, (call collect[(value "op_mod_opt")])],
            // todo: {min}, {min, max} maybe with expression?
            [TokenCall, _],
            ["peek", _, (expect Token, "Token"), (call collect[(value "op_mod_peek")])],
            ["not", _, (expect Token, "Token"), (call collect[(value "op_mod_not")])],
            ["expect", _, (expect Token, "Token"), (call collect[(value "op_mod_expect")])]
        }),

        // Expression & Flow

        (CollectionItem = {
            [T_Identifier, _, "=>", _, Expression, (call collect[(value "alias")])],
            [Expression, "=>", _, Expression, (call collect[(value "alias")])],
            Expression
        }),

        (Atomic = {
            ["(", _, Expression, ")", _], // no expect ")" here!
            ["(", _, (pos [Expression, (opt [",", _])]), ")", _, // no expect ")" here!
                (call collect[(value "collection")])],
            ["(", _, (pos [CollectionItem, (opt [",", _])]), (expect ")"), _,
                (call collect[(value "collection")])],
            Literal,
            Token,
            Call,
            Rvalue,
            Block,
            Parselet
        }),

        (Unary = {
            ["-", _, Unary, (call collect[(value "op_unary_neg")])],
            ["!", _, Unary, (call collect[(value "op_unary_not")])],
            Atomic
        }),

        // todo: & and |

        (MulDiv = {
            [MulDiv, "*", _, (expect Unary), (call collect[(value "op_binary_mul")])],
            [MulDiv, "/", _, (expect Unary), (call collect[(value "op_binary_div")])],
            // todo: ^ (pow)
            Unary
        }),

        (AddSub = {
            [AddSub, "+", _, MulDiv, // no expect(MulDiv) here because of pre-increment fallback
                (call collect[(value "op_binary_add")])],
            [AddSub, "-", _, MulDiv, // no expect(MulDiv) here because of pre-decrement fallback
                (call collect[(value "op_binary_sub")])],
            MulDiv
        }),

        (Compare = {
            [Compare, "==", _, (expect AddSub), (call collect[(value "op_compare_equal")])],
            [Compare, "!=", _, (expect AddSub), (call collect[(value "op_compare_unequal")])],
            [Compare, "<=", _, (expect AddSub), (call collect[(value "op_compare_lowerequal")])],
            [Compare, ">=", _, (expect AddSub), (call collect[(value "op_compare_greaterequal")])],
            [Compare, "<", _, (expect AddSub), (call collect[(value "op_compare_lower")])],
            [Compare, ">", _, (expect AddSub), (call collect[(value "op_compare_greater")])],
            AddSub
        }),

        (LogicalAnd = {
            [LogicalAnd, "&&", _, (expect Compare), (call collect[(value "op_logical_and")])],
            Compare
        }),

        (LogicalOr = {
            [LogicalOr, "||", _, (expect LogicalAnd), (call collect[(value "op_logical_or")])],
            LogicalAnd
        }),


        (Assign = {
            [Lvalue, "=", _, Expression, (call collect[(value "assign")])] // fixme: a = b = c is possible here...
            // todo: add operators "+="", "-="", "*="", "/=" here as well
        }),

        (ExpressionOrVoid = {
            Expression,
            (call collect[(value "value_void")])
        }),

        (Expression = {
            // if
            ["if", _, Expression, Statement, "else", _, Statement, (call collect[(value "op_ifelse")])],
            ["if", _, Expression, Statement, (call collect[(value "op_if")])],
            ["if", _, (call error[(value "'if': Expecting condition and statement")])],

            // while
            ["while", _, Expression, (kle T_EOL), Statement, (call collect[(value "op_while")])],
            ["while", _, (call error[(value "'while': Expecting end-condition and statement")])],

            // for
            ["for", _, T_Identifier, _, "in", _, Expression, Statement, (call collect[(value "op_for_in")])],
            ["for", _, StatementOrVoid, ";", _, StatementOrVoid, ";", _, StatementOrVoid, (opt T_EOL), _, StatementOrVoid, (call collect[(value "op_for")])],
            ["for", _, (call error[(value "'for': Expecting start; condition; iter; statement")])],

            // assignment
            [Lvalue, "=", _, Expression, (call collect[(value "assign_hold")])],
            [Lvalue, "+=", _, Expression, (call collect[(value "assign_add_hold")])],
            [Lvalue, "-=", _, Expression, (call collect[(value "assign_sub_hold")])],
            [Lvalue, "*=", _, Expression, (call collect[(value "assign_mul_hold")])],
            [Lvalue, "/=", _, Expression, (call collect[(value "assign_div_hold")])],

            // normal expression starting with LogicalOr
            LogicalOr
        }),

        (StatementOrVoid = {
            Statement,
            (call collect[(value "value_void")])
        }),

        (Statement = {
            ["accept", _, ExpressionOrVoid, (call collect[(value "op_accept")])],
            ["return", _, ExpressionOrVoid, (call collect[(value "op_accept")])],
            ["repeat", _, ExpressionOrVoid, (call collect[(value "op_repeat")])],
            ["reject", _, (call collect[(value "op_reject")])],

            // todo: report, escape, repeat
            [Lvalue, "=", _, Expression, (call collect[(value "assign")])],
            [Lvalue, "+=", _, Expression, (call collect[(value "assign_add")])],
            [Lvalue, "-=", _, Expression, (call collect[(value "assign_sub")])],
            [Lvalue, "*=", _, Expression, (call collect[(value "assign_mul")])],
            [Lvalue, "/=", _, Expression, (call collect[(value "assign_div")])],

            Expression
        }),

        // Parselet

        (Argument = {
            [T_Identifier, _, (opt ["=", _, (opt Expression)]), (call collect[(value "arg")])]
        }),

        (Arguments = {
            (pos [Argument, (opt [",", _])])
        }),

        (Parselet = {
            ["@", _, (opt Arguments), Block, (call collect[(value "value_parselet")])],
            ["@", _, (opt Arguments), Token, (call collect[(value "value_parselet")])]
        }),

        (Block = {
            ["{", _, (pos Instruction), _, (expect "}"), _, (call collect[(value "block")])],
            ["{", _, (expect "}"), _, (Op::PushVoid), (call collect[(value "block")])]
        }),

        // Sequences

        (SequenceItem = {
            [T_Identifier, _, "=>", _, Expression, (call collect[(value "alias")])],
            [Expression, _, "=>", _, Expression, (call collect[(value "alias")])],
            Statement
        }),

        (Sequence = {
            [(pos SequenceItem), (call collect[(value "sequence")])]
        }),

        // Instructions

        (Instruction = {
            ["begin", _, Block, (call collect[(value "begin")])],
            ["begin", _, Statement, (expect T_EOL), (call collect[(value "begin")])],
            ["end", _, Block, (call collect[(value "end")])],
            ["end", _, Statement, (expect T_EOL), (call collect[(value "end")])],

            [T_Identifier, _, ":", _, (expect Expression), (expect T_EOL),
                (call collect[(value "constant")])],
            Sequence,
            [T_EOL, (Op::Skip)]
        }),

        (Tokay = {
            (pos Instruction)
        }),

        [_, Tokay, (call collect[(value "main")])]

        // ----------------------------------------------------------------------------
                    }))
    }

    pub fn parse(&self, mut reader: Reader) -> Result<Value, Error> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        match self.0.run(&mut runtime) {
            Ok(Some(ast)) => {
                let ast = Value::from_ref(ast).unwrap();

                if ast.get_dict().is_some() {
                    Ok(ast)
                } else {
                    Err(Error::new(None, "Parse error".to_string()))
                }
            }
            Ok(None) => Ok(Value::Void),
            Err(error) => Err(error),
        }
    }

    pub fn print(ast: &Value) {
        fn print(value: &Value, indent: usize) {
            match value {
                Value::Dict(d) => {
                    let emit = d["emit"].borrow();
                    let emit = emit.get_string().unwrap();

                    let row = d.get("row").and_then(|row| Some(row.borrow().to_addr()));
                    let col = d.get("col").and_then(|col| Some(col.borrow().to_addr()));
                    let end_row = d
                        .get("end_row")
                        .and_then(|row| Some(row.borrow().to_addr()));
                    let end_col = d
                        .get("end_col")
                        .and_then(|col| Some(col.borrow().to_addr()));

                    let value = d.get("value");
                    let children = d.get("children");

                    if let (Some(row), Some(col), Some(end_row), Some(end_col)) =
                        (row, col, end_row, end_col)
                    {
                        print!(
                            "{:indent$}{} [{}:{} - {}:{}]",
                            "",
                            emit,
                            row,
                            col,
                            end_row,
                            end_col,
                            indent = indent
                        );
                    } else if let (Some(row), Some(col)) = (row, col) {
                        print!("{:indent$}{} [{}:{}]", "", emit, row, col, indent = indent);
                    } else {
                        print!("{:indent$}{}", "", emit, indent = indent);
                    }

                    if let Some(value) = value {
                        print!(" {:?}", value.borrow());
                    }
                    print!("\n");

                    if let Some(children) = children {
                        print(&children.borrow(), indent + 1);
                    }
                }

                Value::List(l) => {
                    for item in l.iter() {
                        print(&item.borrow(), indent);
                    }
                }

                other => unimplemented!("{:?} is not implemented", other),
            }
        }

        print(ast, 0);
    }
}
