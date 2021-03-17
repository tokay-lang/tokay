use crate::compiler::*;
use crate::reader::Reader;
use crate::value::Value;
use crate::vm::*;
use crate::{ccl, compile, compile_item, value};

/**
This module implements a tokay parser in tokay itself, using the tokay-compiler macros.
This is the general place to change syntax and modify the design of the abstract syntax tree.
*/

pub struct Parser(Program);

impl Parser {
    pub fn new() -> Self {
        Self(compile!({
        // ----------------------------------------------------------------------------

        // Whitespace & EOL

        (_ = {
            [" "],
            ["#", (Chars::until('\n')), "\n"],
            ["\\", "\n"]
        }),

        (T_EOL = {
            [(Chars::char('\n')), _, (Op::Skip)],
            [(Chars::char(';')), _, (Op::Skip)]
        }),

        // Prime Tokens (might probably be replaced by something native, pluggable one)

        (T_Identifier = {
            [
                (Chars::new(ccl!['A'..='Z', 'a'..='z', '_'..='_'])),
                (Repeat::optional_silent(
                    Chars::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
                )),
                (call leaf[(value "identifier")])
            ]
        }),

        (T_String = {
            [
                "\"",
                (Chars::until('"')),     //fixme: Escape sequences (using Until built-in parselet)
                "\""
            ]
        }),

        (T_Match = {
            [
                "\'",
                (Chars::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
                "\'"
            ]
        }),

        (T_Integer = {
            // todo: implement as built-in Parselet
            [(Chars::span(ccl!['0'..='9'])), (call node[(value "value_integer")])]
        }),

        (T_Float = {
            // todo: implement as built-in Parselet
            [(Chars::span(ccl!['0'..='9'])), ".",
                (Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
                    (call leaf[(value "value_float")])],
            [(Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
                ".", (Chars::span(ccl!['0'..='9'])),
                    (call leaf[(value "value_float")])]
        }),

        // Statics, Variables & Constants

        (S_Tail = {
            [".", _, T_Identifier, _, (call node[(value "attribute")])],
            ["[", _, S_Expression, "]", _, (call node[(value "index")])]
        }),

        (S_Capture = {
            ["$", T_Identifier, _, (call node[(value "capture_alias")])],
            ["$", T_Integer, _, (call node[(value "capture_index")])],
            ["$", "(", _, S_Expression, ")", _, (call node[(value "capture")])],
            ["$", (call error[(value "Either use $int or $name for captures, thanks")])]
        }),

        (S_Variable = {
            T_Identifier,
            S_Capture
        }),

        (S_Lvalue = {
            [S_Variable, _, (kle S_Tail), (call node[(value "lvalue")])]
        }),

        (S_Inplace = {
            /* todo: drafted support for inplace increment and decrement operators,
            these are not supported by the compiler, yet. */

            [S_Lvalue, "++", (call node[(value "inplace_post_inc")])],
            [S_Lvalue, "--", (call node[(value "inplace_post_dec")])],
            ["++", S_Lvalue, (call node[(value "inplace_pre_inc")])],
            ["--", S_Variable, (call node[(value "inplace_pre_dec")])],
            S_Variable
        }),

        (S_Rvalue = {
            [S_Inplace, _, (kle S_Tail), (call node[(value "rvalue")])]
        }),

        (S_Parameter = {
            [T_Identifier, _, "=", _, S_Expression, (call node[(value "param_named")])],
            [S_Expression, (call node[(value "param")])]
        }),

        (S_Parameters = {
            (pos [S_Parameter, (opt [",", _])])
        }),

        (S_Call = {
            [T_Identifier, "(", _, (opt S_Parameters), ")", _, (call node[(value "call_identifier")])]
            //[S_Rvalue, "(", _, (opt S_Parameters), ")", _, (call node[(value "call_rvalue")])]
        }),

        (S_Literal = {
            ["true", _, (call node[(value "value_true")])],
            ["false", _, (call node[(value "value_false")])],
            ["void", _, (call node[(value "value_void")])],
            ["null", _, (call node[(value "value_null")])],
            [T_String, _, (call node[(value "value_string")])],
            [T_Float, _],
            [T_Integer, _]
        }),

        (S_Token = {
            ["peek", _, S_Token, (call node[(value "mod_peek")])],
            ["not", _, S_Token, (call node[(value "mod_not")])],
            ["pos", _, S_Token, (call node[(value "mod_positive")])],      // fixme: not final!
            ["kle", _, S_Token, (call node[(value "mod_kleene")])],        // fixme: not final!
            ["opt", _, S_Token, (call node[(value "mod_optional")])],      // fixme: not final!
            ["'", T_Match, "'", _, (call node[(value "match")])],
            [T_Match, _, (call node[(value "touch")])]
            // fixme: consumable token identifiers?
        }),

        (S_Value = {
            S_Literal,
            S_Parselet
        }),

        // Expression & Flow

        (S_Atomic = {
            ["(", _, S_Expression, (expect ")"), _],
            S_Literal,
            S_Token,
            S_Call,
            S_Rvalue,
            S_Block,
            S_Parselet
        }),

        (S_Unary = {
            ["-", _, S_Atomic, (call node[(value "op_unary_sub")])],
            ["+", _, S_Atomic, (call node[(value "op_unary_add")])],
            ["!", _, S_Atomic, (call node[(value "op_unary_not")])],
            S_Atomic
        }),

        (S_MulDiv = {
            [S_MulDiv, "*", _, (expect S_Unary), (call node[(value "op_binary_mul")])],
            [S_MulDiv, "/", _, (expect S_Unary), (call node[(value "op_binary_div")])],
            S_Unary
        }),

        (S_AddSub = {
            [S_AddSub, "+", _, (expect S_MulDiv), (call node[(value "op_binary_add")])],
            [S_AddSub, "-", _, (expect S_MulDiv), (call node[(value "op_binary_sub")])],
            S_MulDiv
        }),

        (S_Compare = {
            [S_Compare, "==", _, (expect S_AddSub), (call node[(value "op_compare_equal")])],
            [S_Compare, "!=", _, (expect S_AddSub), (call node[(value "op_compare_unequal")])],
            [S_Compare, "<=", _, (expect S_AddSub), (call node[(value "op_compare_lowerequal")])],
            [S_Compare, ">=", _, (expect S_AddSub), (call node[(value "op_compare_greaterequal")])],
            [S_Compare, "<", _, (expect S_AddSub), (call node[(value "op_compare_lower")])],
            [S_Compare, ">", _, (expect S_AddSub), (call node[(value "op_compare_greater")])],
            S_AddSub
        }),

        (S_Assign = {
            [S_Lvalue, "=", _, S_Expression, (call node[(value "assign")])] // fixme: a = b = c is possible here...
            // todo: add operators "+="", "-="", "*="", "/=" here as well
        }),

        (S_Expression = {
            ["if", _, S_Expression, S_Statement, "else", _, S_Statement,
                (call node[(value "op_ifelse")])],
            ["if", _, S_Expression, S_Statement, (call node[(value "op_if")])],
            S_Compare
        }),

        (S_Statement = {
            ["return", _, S_Expression, (call node[(value "op_return")])],
            ["return", _, (call node[(value "op_returnvoid")])],
            ["accept", _, S_Expression, (call node[(value "op_accept")])],
            ["accept", _, (call node[(value "op_acceptvoid")])],
            ["reject", _, (call node[(value "op_reject")])],
            S_Assign,
            S_Expression
        }),

        // Parselet

        (S_Argument = {
            //[T_Identifier, _, ":", _, (opt S_Value), (call node[(value "arg_constant")])],  // todo: later...
            [T_Identifier, _, (opt ["=", _, (opt S_Value)]), (call node[(value "arg")])]
        }),

        (S_Arguments = {
            (pos [S_Argument, (opt [",", _])])
        }),

        (S_Parselet = {
            ["@", _, (opt S_Arguments), S_Block, (call node[(value "value_parselet")])],
            ["@", _, S_Sequence, (call node[(value "value_parselet")])]
        }),

        (S_Block = {
            ["{", _, S_Sequences, _, (expect "}"), _, (call node[(value "block")])],
            ["{", _, (expect "}"), _, (Op::PushVoid), (call node[(value "block")])]
        }),

        // Sequences

        (S_Sequences = {
            (pos S_Sequence)
        }),

        (S_Sequence = {
            ["begin", _, S_Statement, (call node[(value "begin")])],
            ["end", _, S_Statement, (call node[(value "end")])],
            [(pos S_Item), (call node[(value "sequence")])],
            [T_EOL, (Op::Skip)]
        }),

        (S_Item = {
            // todo: Recognize aliases
            [T_Identifier, _, ":", _, S_Value, T_EOL, (call node[(value "assign_constant")])],
            S_Statement
        }),

        /*
        (S_TokenModifier = {
            ["!", S_TokenModifier, (call node[(value "mod_not")])],
            ["~", S_TokenModifier, (call node[(value "mod_peek")])],
            [S_Token, "+", _, (call node[(value "mod_positive")])],
            [S_Token, "*", _, (call node[(value "mod_kleene")])],
            [S_Token, "?", _, (call node[(value "mod_optional")])],
            [
                S_Token, _,
                (Op::Peek(
                    Op::Not(
                        Chars::new(ccl![
                            '='..='=',
                            '+'..='+',
                            '-'..='-',
                            '*'..='*',
                            '/'..='/'
                            // todo: More to come?
                        ]).into_box()
                    ).into_box()
                ))
            ]
        }),

        (S_Token = {
            [T_String, (call node[(value "match")])],
            [T_LightString, (call node[(value "touch")])],
            [".", _, (call node[(value "any")])],
            S_Call,
            [T_Identifier, (call node[(value "call_or_load")])],
            S_Parselet
        }),
        */

        (S_Tokay = {
            S_Sequences
        }),

        [_, S_Tokay, (call node[(value "main")])]

        // ----------------------------------------------------------------------------
                    }))
    }

    pub fn parse(&self, mut reader: Reader) -> Result<Value, String> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        match self.0.run(&mut runtime) {
            Ok(Some(ast)) => {
                let ast = Value::from_ref(ast).unwrap();

                if ast.get_dict().is_some() {
                    Ok(ast)
                } else {
                    Err("Parse error".to_string())
                }
            }
            Ok(None) => Ok(Value::Void),
            Err(Some(error)) => Err(error),
            Err(None) => Err("Parse error".to_string()),
        }
    }
}
