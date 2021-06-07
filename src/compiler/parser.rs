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
            [";", _, (Op::Skip)],
            [(token (Token::EOF)), (Op::Skip)],
            [(peek "}"), (Op::Skip)]
        }),

        (T_NoIdentifier = {
            (not (token (Token::Char(ccl!['A'..='Z', 'a'..='z', '_'..='_']))))
        }),

        // Prime Tokens (might probably be replaced by something native, pluggable one)

        (T_Identifier = {
            [
                (token (Token::Char(ccl!['A'..='Z', 'a'..='z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call ast[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_Consumable = {
            [
                (token (Token::Char(ccl!['A'..='Z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call ast[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_Alias = {  // T_Alias is an identifier treated as string value
            [
                (token (Token::Char(ccl!['A'..='Z', 'a'..='z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call ast[(value "value_string"), (Op::LoadFastCapture(0))])
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
            [(token (Token::Chars(ccl!['0'..='9']))), (call ast[(value "value_integer")])]
        }),

        (T_Float = {
            // todo: implement as built-in Parselet
            [(token (Token::Chars(ccl!['0'..='9']))), ".", (opt (token (Token::Chars(ccl!['0'..='9'])))),
                (call ast[(value "value_float"), (Op::LoadFastCapture(0))])],
            [(opt (token (Token::Chars(ccl!['0'..='9'])))), ".", (token (Token::Chars(ccl!['0'..='9']))),
                (call ast[(value "value_float"), (Op::LoadFastCapture(0))])]
        }),

        // Character classes

        (CclChar = {
            [EOF, (call error[(value "Unclosed character-class, expecting ']'")])],
            ["\\", (token (Token::Any))],
            (token (Token::char_except(']')))
        }),

        (CclRange = {
            [CclChar, "-", CclChar,
                (call ast[(value "range"), [(Op::LoadFastCapture(1)), (Op::LoadFastCapture(3)), (Op::Add)]])],
            [CclChar, (call ast[(value "char")])]
        }),

        (Ccl = {
            ['^', (kle CclRange), (call ast[(value "ccl_neg")])],
            [(kle CclRange), (call ast[(value "ccl")])]
        }),

        // Statics, Variables & Constants

        (Subscript = {
            [".", T_Identifier, _, (call ast[(value "attribute")])],
            ["[", _, Expression, "]", _, (call ast[(value "index")])]
        }),

        (Capture = {
            ["$", T_Alias, _, (call ast[(value "capture_alias")])],
            ["$", T_Integer, _, (call ast[(value "capture_index")])],
            ["$", "(", _, Expression, ")", _, (call ast[(value "capture_expr")])],
            ["$", (call error[(value "'$': Expecting identifier, integer or (expression)")])]
        }),

        (Variable = {
            T_Identifier,
            Capture
        }),

        (Lvalue = {
            [Variable, (kle Subscript), _, (call ast[(value "lvalue")])]
        }),

        (Load = {
            [Lvalue, "++", (call ast[(value "inplace_post_inc")])],
            [Lvalue, "--", (call ast[(value "inplace_post_dec")])],
            ["++", (expect Lvalue), (call ast[(value "inplace_pre_inc")])],
            ["--", (expect Lvalue), (call ast[(value "inplace_pre_dec")])],
            Variable
        }),

        (CallParameter = {
            [T_Identifier, _, "=", _, (expect Expression), (call ast[(value "param_named")])],
            [Expression, (call ast[(value "param")])]
        }),

        (CallParameters = {
            (pos [CallParameter, (opt [",", _])])
        }),

        (Literal = {
            /* below calls to (peek T_NoIdentifier) avoid to wrongly interpret
                e.g. "truex" as "true" and "x" */
            ["true", (peek T_NoIdentifier), (call ast[(value "value_true")])],
            ["false", (peek T_NoIdentifier), (call ast[(value "value_false")])],
            ["void", (peek T_NoIdentifier), (call ast[(value "value_void")])],
            ["null", (peek T_NoIdentifier), (call ast[(value "value_null")])],
            [T_String, (call ast[(value "value_string")])],
            T_Float,
            T_Integer
        }),

        // Tokens

        (TokenLiteral = {
            ["'", T_Match, "'", (call ast[(value "value_token_match")])],
            [T_Match, (call ast[(value "value_token_touch")])],
            [".", (call ast[(value "value_token_any")])],
            ['[', Ccl, ']', (call ast[(value "value_token_ccl")])]
        }),

        (TokenCall = {
            TokenLiteral,
            [T_Consumable, "(", _, (opt CallParameters), (expect ")"),
                (call ast[(value "call")])],
            [T_Consumable, (call ast[(value "call")])]
        }),

        (Token = {
            // Token call modifiers
            [TokenCall, "+", (call ast[(value "op_mod_pos")])],
            [TokenCall, "*", (call ast[(value "op_mod_kle")])],
            [TokenCall, "?", (call ast[(value "op_mod_opt")])],
            // todo: {min}, {min, max} maybe with expression?
            TokenCall,
            ["peek", _, (expect Token), (call ast[(value "op_mod_peek")])],
            ["not", _, (expect Token), (call ast[(value "op_mod_not")])],
            ["expect", _, (expect Token), (call ast[(value "op_mod_expect")])]
        }),

        // Expression & Flow

        (CollectionItem = {
            [T_Alias, _, "=>", _, (expect Expression), (call ast[(value "alias")])],
            [Expression, "=>", _, (expect Expression), (call ast[(value "alias")])],
            Expression
        }),

        (Collection = {
            ["(", _, (pos [Expression, (opt [",", _])]), ")", // no expect ")" here!
                (call ast[(value "collection")])],
            ["(", _, (pos [CollectionItem, (opt [",", _])]), (expect ")"),
                (call ast[(value "collection")])]
        }),

        (Atomic = {
            ["(", _, Expression, ")"], // no expect ")" here!
            Collection,
            Literal,
            Token,
            Load,
            Block,
            Parselet
        }),

        (Rvalue = {
            [Rvalue, "(", _, (opt CallParameters), (expect ")"), _,
                (call ast[(value "call")])],
            [Atomic, (pos Subscript), (call ast[(value "rvalue")])],
            Atomic
        }),

        (Unary = {
            ["-", _, Unary, (call ast[(value "op_unary_neg")])],
            ["!", _, Unary, (call ast[(value "op_unary_not")])],
            [Rvalue, _]
        }),

        // todo: & and |

        (MulDiv = {
            [MulDiv, "*", _, (expect Unary), (call ast[(value "op_binary_mul")])],
            [MulDiv, "/", _, (expect Unary), (call ast[(value "op_binary_div")])],
            // todo: ^ (pow)
            Unary
        }),

        (AddSub = {
            [AddSub, "+", _, MulDiv, // no expect(MulDiv) here because of pre-increment fallback
                (call ast[(value "op_binary_add")])],
            [AddSub, "-", _, MulDiv, // no expect(MulDiv) here because of pre-decrement fallback
                (call ast[(value "op_binary_sub")])],
            MulDiv
        }),

        (Compare = {
            [Compare, "==", _, (expect AddSub), (call ast[(value "op_compare_equal")])],
            [Compare, "!=", _, (expect AddSub), (call ast[(value "op_compare_unequal")])],
            [Compare, "<=", _, (expect AddSub), (call ast[(value "op_compare_lowerequal")])],
            [Compare, ">=", _, (expect AddSub), (call ast[(value "op_compare_greaterequal")])],
            [Compare, "<", _, (expect AddSub), (call ast[(value "op_compare_lower")])],
            [Compare, ">", _, (expect AddSub), (call ast[(value "op_compare_greater")])],
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

        (ExpressionOrVoid = {
            Expression,
            (call ast[(value "value_void")])
        }),

        (Expression = {
            // if
            ["if", _, Expression, Statement, "else", _, Statement, (call ast[(value "op_ifelse")])],
            ["if", _, Expression, Statement, (call ast[(value "op_if")])],
            ["if", _, (call error[(value "'if': Expecting condition and statement")])],

            // for
            ["for", _, T_Identifier, _, "in", _, Expression, Statement, (call ast[(value "op_for_in")])],
            ["for", _, StatementOrVoid, ";", _, StatementOrVoid, ";", _, StatementOrVoid, (opt T_EOL), _,
                StatementOrVoid, (call ast[(value "op_for")])],
            ["for", _, (call error[(value "'for': Expecting start; condition; iter; statement")])],

            // assignment
            [Lvalue, "=", (not {">", "="}), //avoid wrongly matching "=>" or "=="
                _, (expect Expression), (call ast[(value "assign_hold")])],
            [Lvalue, "+=", _, (expect Expression), (call ast[(value "assign_add_hold")])],
            [Lvalue, "-=", _, (expect Expression), (call ast[(value "assign_sub_hold")])],
            [Lvalue, "*=", _, (expect Expression), (call ast[(value "assign_mul_hold")])],
            [Lvalue, "/=", _, (expect Expression), (call ast[(value "assign_div_hold")])],

            // normal expression starting with LogicalOr
            LogicalOr
        }),

        (StatementOrVoid = {
            Statement,
            (call ast[(value "value_void")])
        }),

        (Statement = {
            ["accept", _, ExpressionOrVoid, (call ast[(value "op_accept")])],
            ["return", _, ExpressionOrVoid, (call ast[(value "op_accept")])],
            ["repeat", _, ExpressionOrVoid, (call ast[(value "op_repeat")])],
            ["reject", _, (call ast[(value "op_reject")])],
            // todo: report, escape, repeat

            [Lvalue, "=", (not {">", "="}), //avoid wrongly matching "=>" or "=="
                _, (expect Expression), (call ast[(value "assign")])],
            [Lvalue, "+=", _, (expect Expression), (call ast[(value "assign_add")])],
            [Lvalue, "-=", _, (expect Expression), (call ast[(value "assign_sub")])],
            [Lvalue, "*=", _, (expect Expression), (call ast[(value "assign_mul")])],
            [Lvalue, "/=", _, (expect Expression), (call ast[(value "assign_div")])],

            Expression
        }),

        // Parselet

        (Argument = {
            [T_Identifier, _, (opt ["=", _, (opt Expression)]), (call ast[(value "arg")])]
        }),

        (Arguments = {
            (pos [Argument, (opt [",", _])])
        }),

        (Parselet = {
            ["@", _, (opt Arguments), Block, (call ast[(value "value_parselet")])],
            ["@", _, (opt Arguments), Token, (call ast[(value "value_parselet")])]
        }),

        (Block = {
            ["{", _, (pos Instruction), _, (expect "}"), _, (opt T_EOL), (call ast[(value "block")])],
            ["{", _, (expect "}"), _, (opt T_EOL), (Op::PushVoid), (call ast[(value "block")])]
        }),

        // Sequences

        (SequenceItem = {
            [T_Alias, _, "=>", _, (expect Expression), (call ast[(value "alias")])],
            [Expression, "=>", _, (expect Expression), (call ast[(value "alias")])],
            Statement
        }),

        (Instruction = {
            ["begin", _, Block, _, (call ast[(value "begin")])],
            ["begin", _, Statement, (expect T_EOL), (call ast[(value "begin")])],
            ["end", _, Block, _, (call ast[(value "end")])],
            ["end", _, Statement, (expect T_EOL), (call ast[(value "end")])],

            [T_Identifier, _, ":", _, (expect Expression), (expect T_EOL),
                (call ast[(value "constant")])],
            [(pos SequenceItem), (call ast[(value "sequence")])],
            [T_EOL, (Op::Skip)]
        }),

        (Tokay = {
            (pos Instruction),
            [(token (Token::Any)),
                (call error[(value "Parse error, unexpected token"), (value true)])]
        }),

        [_, Tokay, (call ast[(value "main")])]

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
}
