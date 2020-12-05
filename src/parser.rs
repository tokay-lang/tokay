use crate::reader::Reader;
use crate::tokay::*;
use crate::value::{RefValue, Value};
use crate::compiler::Compiler;
use crate::{tokay, tokay_item, ccl};


pub struct TokayParser(Program);

impl TokayParser {
    pub fn new() -> Self {
        Self(
            tokay!({
// ----------------------------------------------------------------------------

// Whitespace & EOL

(_ = {
    [" "],
    ["#", (Char::until('\n'))],
    ["\\", "\n"]
}),

(T_EOL = {
    [
        (Char::char('\n')),
        _,
        (Op::Skip)
    ]
}),

// Prime Tokens (might probably be replaced by something native, pluggable one)

(T_Identifier = {
    [
        (Char::new(ccl!['A'..='Z', 'a'..='z', '_'..='_'])),
        (Repeat::muted_optional(
            Char::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::PushAddr(0)),
        (Op::LoadCapture),
        (Op::Lexeme("identifier"))
    ]
}),

(T_Variable = {
    [
        (Char::new(ccl!['a'..='z'])),
        (Repeat::muted_optional(
            Char::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::PushAddr(0)),
        (Op::LoadCapture),
        (Op::Lexeme("variable"))
    ]
}),

(T_Constant = {
    [
        (Char::new(ccl!['A'..='Z', '_'..='_'])),
        (Repeat::muted_optional(
            Char::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::PushAddr(0)),
        (Op::LoadCapture),
        (Op::Lexeme("constant"))
    ]
}),

(T_HeavyString = {
    [
        "\"",
        (Char::until('"')),     //fixme: Escape sequences (using Until built-in parselet)
        "\"",
        (Op::PushAddr(2)),
        (Op::LoadCapture),
        (Op::Create("string"))
    ]
}),

(T_LightString = {
    [
        "\'",
        (Char::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
        "\'",
        (Op::PushAddr(2)),
        (Op::LoadCapture),
        (Op::Create("string"))
    ]
}),

(T_Integer = {
    // todo: implement as built-in Parselet
    [(Char::span(ccl!['0'..='9'])), (Op::Create("integer"))]
}),

(T_Float = {
    // todo: implement as built-in Parselet
    [(Char::span(ccl!['0'..='9'])), ".",
        (Repeat::muted_optional(Char::span(ccl!['0'..='9']))),
            (Op::Lexeme("float"))],
    [(Repeat::muted_optional(Char::span(ccl!['0'..='9']))),
        ".", (Char::span(ccl!['0'..='9'])),
            (Op::Lexeme("float"))]
}),

// Statics, Variables & Constants

(S_Tail = {
    [".", _, T_Identifier, _, (Op::Create("attribute"))],
    ["[", _, S_Expression, _, "]", _, (Op::Create("index"))]
}),

(S_Capture = {
    ["$", T_Identifier, _, (Op::Create("capture_named"))],
    ["$", T_Integer, _, (Op::Create("capture"))],
    ["$", (Op::Error("Either use $int or $name for captures, thanks"))]
}),

(S_Lvalue = {
    [T_Variable, _, (kle S_Tail), (Op::Create("lvalue"))],
    [S_Capture, _, (kle S_Tail), (Op::Create("lvalue"))]
}),

(S_Rvalue = {
    [T_Constant, (Op::Create("rvalue"))],
    [T_Variable, _, (kle S_Tail), (Op::Create("rvalue"))],
    [S_Capture, _, (kle S_Tail), (Op::Create("rvalue"))]
}),

(S_CallParameters = {
    [S_CallParameters, _, ",", _, S_Expression],
    [S_Expression, _]
}),

(S_Call = {
    [S_Rvalue, _, "(", (opt S_CallParameters), ")", (Op::Create("call"))]
}),

(S_String = {
    [T_HeavyString, _],
    [T_LightString, _]
}),

(S_Literal = {
    ["true", _, (Op::Create("true"))],
    ["false", _, (Op::Create("false"))],
    ["void", _, (Op::Create("void"))],
    ["unset", _, (Op::Create("unset"))],
    [T_Float, _],
    [T_Integer, _]
}),

// Expression & Flow

(S_Atomic = {
    ["(", _, S_Expression, ")", _],
    S_Literal,
    S_String,
    S_Call,
    S_Rvalue,
    S_Block,
    S_Parselet
}),

(S_Unary = {
    ["-", _, S_Atomic, (Op::Create("unary_sub"))],
    ["+", _, S_Atomic, (Op::Create("unary_add"))],
    ["!", _, S_Atomic, (Op::Create("unary_not"))],
    S_Atomic
}),

(S_MulDiv = {
    [S_MulDiv, "*", _, S_Unary, (Op::Create("binary_mul"))],
    [S_MulDiv, "/", _, S_Unary, (Op::Create("binary_div"))],
    S_Unary
}),

(S_AddSub = {
    [S_AddSub, "+", _, S_MulDiv, (Op::Create("binary_add"))],
    [S_AddSub, "-", _, S_MulDiv, (Op::Create("binary_sub"))],
    S_MulDiv
}),

(S_Compare = {
    [S_Compare, "==", _, S_AddSub, (Op::Create("compare_equal"))],
    [S_Compare, "!=", _, S_AddSub, (Op::Create("compare_unequal"))],
    [S_Compare, "<=", _, S_AddSub, (Op::Create("compare_lowerequal"))],
    [S_Compare, ">=", _, S_AddSub, (Op::Create("compare_greaterequal"))],
    [S_Compare, "<", _, S_AddSub, (Op::Create("compare_lower"))],
    [S_Compare, ">", _, S_AddSub, (Op::Create("compare_greater"))],
    S_AddSub
}),

(S_Expression = {
    ["if", _, S_Expression, S_Expression, "else", _, S_Expression,
        (Op::Create("if_else"))],
    ["if", _, S_Expression, S_Expression, (Op::Create("if"))],
    ["return", _, S_Expression, (Op::Create("return"))],
    ["return", _, (Op::Create("return_void"))],
    ["accept", _, S_Expression, (Op::Create("accept"))],
    ["accept", _, (Op::Create("accept_void"))],
    ["reject", _, (Op::Create("reject"))],
    [S_Lvalue, _, "=", _, S_Expression, _, (Op::Create("assign"))],
    S_Compare
}),

// Structure

(S_Parselet = {
    ["@", _, S_Block, (Op::Create("parselet"))]
}),

(S_Block = {
    ["{", _, S_Sequences, _,
        (Op::Expect(Box::new(Match::new("}").into_op()))), _,
        (Op::Create("block"))],
    ["{", _, "}", _, (Op::PushVoid), (Op::Create("block"))]
}),

(S_Sequences = {
    (pos S_Sequence)
}),

(S_Sequence = {
    (S_Sequence1 = {
        [S_Sequence1, S_Item],
        [S_Item]
    }),

    [T_Constant, _, "=", _, S_Parselet, _, (Op::Create("assign_constant"))],
    [S_Sequence1, (Op::Create("sequence"))],
    [T_EOL, (Op::Skip)]
}),

(S_Item = {
    S_TokenModifier,
    S_Expression
}),

(S_TokenModifier = {
    [S_Token, "+", _, (Op::Create("mod_positive"))],
    [S_Token, "*", _, (Op::Create("mod_kleene"))],
    [S_Token, "?", _, (Op::Create("mod_optional"))],
    [S_Token, _]
}),

(S_ConstantCallParameter = {
    [S_TokenModifier, _],
    [S_Expression, _, (Op::Error("You may not use expressions here..."))]
}),

(S_ConstantCallParameters = {
    [S_ConstantCallParameters, _, ",", _, S_ConstantCallParameter],
    [S_ConstantCallParameter]
}),

(S_ConstantCall = {
    [T_Constant, _, "(", _, S_ConstantCallParameters, ")", _,
        (Op::Create("call_constant"))],
    [T_Constant, _, (Op::Create("call_constant"))]
}),

(S_Token = {
    [T_HeavyString, _],
    [T_LightString, _],
    [S_ConstantCall, _],
    [S_Parselet, _]
}),

(S_Tokay = {
    S_Sequences
}),

[S_Tokay, (Op::Create("tokay"))]

// ----------------------------------------------------------------------------
            })
        )
    }

    pub fn parse(&self, mut reader: Reader) -> Result<Value, String> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        let res = self.0.run(&mut runtime);

        if let Ok(accept) = res {
            if let Accept::Push(Capture::Value(value)) = accept {
                return Ok(RefValue::into_value(value).unwrap());
            }
        }
        else {
            println!("Error: {:#?}", res.err());
        }

        return Err("Parse error".to_string())
    }
}
