use crate::reader::Reader;
use crate::vm::*;
use crate::value::Value;
use crate::compiler::*;
use crate::{compile, compile_item, ccl};


/** This implements a tokay parser in tokay itself,
    using the tokay-compiler macros. This is the
    general place to change syntax and modify the
    design of the abstract syntax tree. */

pub struct Parser(Program);

impl Parser {
    pub fn new() -> Self {
        Self(
            compile!({
// ----------------------------------------------------------------------------

// Whitespace & EOL

(_ = {
    [" "],
    ["#", (Chars::until('\n'))],
    ["\\", "\n"]
}),

(T_EOL = {
    [
        (Chars::char('\n')),
        _,
        (Op::Skip)
    ]
}),

// Prime Tokens (might probably be replaced by something native, pluggable one)

(T_Identifier = {
    [
        (Chars::new(ccl!['A'..='Z', 'a'..='z', '_'..='_'])),
        (Repeat::optional_silent(
            Chars::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::Lexeme("identifier"))
    ]
}),

(T_HeavyString = {
    [
        "\"",
        (Chars::until('"')),     //fixme: Escape sequences (using Until built-in parselet)
        "\""
    ]
}),

(T_LightString = {
    [
        "\'",
        (Chars::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
        "\'"
    ]
}),

(T_Integer = {
    // todo: implement as built-in Parselet
    [(Chars::span(ccl!['0'..='9'])), (Op::Create("value_integer"))]
}),

(T_Float = {
    // todo: implement as built-in Parselet
    [(Chars::span(ccl!['0'..='9'])), ".",
        (Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
            (Op::Lexeme("value_float"))],
    [(Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
        ".", (Chars::span(ccl!['0'..='9'])),
            (Op::Lexeme("value_float"))]
}),

// Statics, Variables & Constants

(S_Tail = {
    [".", _, T_Identifier, _, (Op::Create("attribute"))],
    ["[", _, S_Expression, "]", _, (Op::Create("index"))]
}),

(S_Capture = {
    ["$", T_Identifier, _, (Op::Create("capture_alias"))],
    ["$", T_Integer, _, (Op::Create("capture_index"))],
    ["$", "(", _, S_Expression, ")", _, (Op::Create("capture"))],
    ["$", (Op::Error("Either use $int or $name for captures, thanks"))]
}),

(S_Lvalue = {
    [T_Identifier, _, (kle S_Tail), (Op::Create("lvalue"))],
    [S_Capture, (kle S_Tail), (Op::Create("lvalue"))]
}),

(S_Rvalue = {
    [T_Identifier, _, (kle S_Tail), (Op::Create("rvalue"))],
    [S_Capture, (kle S_Tail), (Op::Create("rvalue"))]
}),

(S_CallParameters = {
    [S_CallParameters, _, ",", _, S_Expression],
    [S_Expression, _]
}),

(S_Call = {
    [T_Identifier, "(", (opt S_CallParameters), ")", (Op::Create("call_identifier"))],
    [S_Rvalue, "(", (opt S_CallParameters), ")", (Op::Create("call_rvalue"))]
}),

(S_String = {
    [T_HeavyString, _, (Op::Create("value_string"))],
    [T_LightString, _, (Op::Create("value_string"))]
}),

(S_Value = {
    ["true", _, (Op::Create("value_true"))],
    ["false", _, (Op::Create("value_false"))],
    ["void", _, (Op::Create("value_void"))],
    ["null", _, (Op::Create("value_null"))],
    [S_String, _],
    [T_Float, _],
    [T_Integer, _],
    [S_Parselet, _]
}),

// Expression & Flow

(S_Atomic = {
    ["(", _, S_Expression, ")", _],
    S_Call,
    S_Rvalue,
    S_Value,
    S_Block
}),

(S_Unary = {
    ["-", _, S_Atomic, (Op::Create("op_unary_sub"))],
    ["+", _, S_Atomic, (Op::Create("op_unary_add"))],
    ["!", _, S_Atomic, (Op::Create("op_unary_not"))],
    S_Atomic
}),

(S_MulDiv = {
    [S_MulDiv, "*", _, S_Unary, (Op::Create("op_binary_mul"))],
    [S_MulDiv, "/", _, S_Unary, (Op::Create("op_binary_div"))],
    S_Unary
}),

(S_AddSub = {
    [S_AddSub, "+", _, S_MulDiv, (Op::Create("op_binary_add"))],
    [S_AddSub, "-", _, S_MulDiv, (Op::Create("op_binary_sub"))],
    S_MulDiv
}),

(S_Compare = {
    [S_Compare, "==", _, S_AddSub, (Op::Create("op_compare_equal"))],
    [S_Compare, "!=", _, S_AddSub, (Op::Create("op_compare_unequal"))],
    [S_Compare, "<=", _, S_AddSub, (Op::Create("op_compare_lowerequal"))],
    [S_Compare, ">=", _, S_AddSub, (Op::Create("op_compare_greaterequal"))],
    [S_Compare, "<", _, S_AddSub, (Op::Create("op_compare_lower"))],
    [S_Compare, ">", _, S_AddSub, (Op::Create("op_compare_greater"))],
    S_AddSub
}),

(S_Expression = {
    ["if", _, S_Expression, S_Statement, "else", _, S_Statement,
        (Op::Create("op_ifelse"))],
    ["if", _, S_Expression, S_Statement, (Op::Create("op_if"))],
    [S_Lvalue, _, "=", _, S_Expression, _, (Op::Create("assign"))], // fixme: a = b = c is possible here...
    S_Compare
}),

(S_Statement = {
    ["return", _, S_Expression, (Op::Create("op_return"))],
    ["return", _, (Op::Create("op_returnvoid"))],
    ["accept", _, S_Expression, (Op::Create("op_accept"))],
    ["accept", _, (Op::Create("op_acceptvoid"))],
    ["reject", _, (Op::Create("op_reject"))],
    S_Expression
}),

// Structure

(S_Argument = {
    [T_Identifier, _, "=", S_Value],
    [T_Identifier, _]
}),

(S_Arguments = {
    [S_Arguments, ",", _, S_Argument],
        //(Op::CallStatic(builtin::get("flatten").unwrap()))],
    S_Argument
}),

(S_Parselet = {
    ["@", _, (opt S_Arguments), S_Block, (Op::Create("value_parselet"))]
}),

(S_Block = {
    ["{", _, S_Sequences, _,
        (Op::Expect(Box::new(Match::new_silent("}")))), _,
        (Op::Create("block"))],
    ["{", _, (Op::Expect(Box::new(Match::new_silent("}")))), _,
        (Op::PushVoid), (Op::Create("block"))]
}),

(S_Sequences = {
    (pos S_Sequence)
}),

(S_Sequence = {
    [T_Identifier, _, ":", _, S_Value, (Op::Create("assign_constant"))],
    [(pos S_Item), (Op::Create("sequence"))],
    [T_EOL, (Op::Skip)]
}),

(S_Item = {
    // todo: Recognize aliases
    S_TokenModifier,
    S_Statement
}),

(S_TokenModifier = {
    ["!", S_TokenModifier, (Op::Create("mod_not"))],
    ["~", S_TokenModifier, (Op::Create("mod_peek"))],
    [S_Token, "+", _, (Op::Create("mod_positive"))],
    [S_Token, "*", _, (Op::Create("mod_kleene"))],
    [S_Token, "?", _, (Op::Create("mod_optional"))],
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
    [T_HeavyString, (Op::Create("match"))],
    [T_LightString, (Op::Create("match_silent"))],
    S_Call,
    [T_Identifier, (Op::Create("call_or_load"))],
    S_Parselet
}),

(S_Tokay = {
    S_Sequences
}),

[S_Tokay, (Op::Create("main"))]

// ----------------------------------------------------------------------------
            })
        )
    }

    pub fn parse(&self, mut reader: Reader) -> Result<Value, String> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        let res = self.0.run(&mut runtime);

        if let Ok(Some(ast)) = res {
            return Ok(Value::from_ref(ast).unwrap())
        }
        else {
            println!("Error: {:#?}", res.err());
        }


        return Err("Parse error".to_string())
    }
}
