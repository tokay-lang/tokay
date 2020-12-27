use crate::reader::Reader;
use crate::tokay::*;
use crate::value::Value;
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
        (Repeat::optional_silent(
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
        (Repeat::optional_silent(
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
        (Repeat::optional_silent(
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
        "\""
    ]
}),

(T_LightString = {
    [
        "\'",
        (Char::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
        "\'"
    ]
}),

(T_Integer = {
    // todo: implement as built-in Parselet
    [(Char::span(ccl!['0'..='9'])), (Op::Create("value_integer"))]
}),

(T_Float = {
    // todo: implement as built-in Parselet
    [(Char::span(ccl!['0'..='9'])), ".",
        (Repeat::optional_silent(Char::span(ccl!['0'..='9']))),
            (Op::Lexeme("value_float"))],
    [(Repeat::optional_silent(Char::span(ccl!['0'..='9']))),
        ".", (Char::span(ccl!['0'..='9'])),
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
    [T_Variable, _, (kle S_Tail), (Op::Create("lvalue"))],
    [S_Capture, (kle S_Tail), (Op::Create("lvalue"))]
}),

(S_Rvalue = {
    [T_Constant, _, (kle S_Tail), (Op::Create("rvalue"))],
    [T_Variable, _, (kle S_Tail), (Op::Create("rvalue"))],
    [S_Capture, (kle S_Tail), (Op::Create("rvalue"))]
}),

(S_CallParameters = {
    [S_CallParameters, _, ",", _, S_Expression],
    [S_Expression, _]
}),

(S_Call = {
    [S_Rvalue, "(", (opt S_CallParameters), ")", (Op::Create("call"))]
}),

(S_String = {
    [T_HeavyString, _, (Op::Create("value_string"))],
    [T_LightString, _, (Op::Create("value_string"))]
}),

(S_Value = {
    ["true", _, (Op::Create("value_true"))],
    ["false", _, (Op::Create("value_false"))],
    ["void", _, (Op::Create("value_void"))],
    ["unset", _, (Op::Create("value_unset"))],
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
    ["if", _, S_Expression, S_Expression, "else", _, S_Expression,
        (Op::Create("op_flow_ifelse"))],
    ["if", _, S_Expression, S_Expression, (Op::Create("op_flow_if"))],
    //fixme: below this, is it really an expression?
    ["return", _, S_Expression, (Op::Create("op_return"))],
    ["return", _, (Op::Create("op_flow_voidreturn"))],
    ["accept", _, S_Expression, (Op::Create("op_flow_accept"))],
    ["accept", _, (Op::Create("op_flow_voidaccept"))],
    ["reject", _, (Op::Create("op_flow_reject"))],
    //fixme: until here, see above.
    [T_Constant, _, "=", _, S_Value, (Op::Create("assign_constant"))],
    [S_Lvalue, _, "=", _, S_Expression, _, (Op::Create("assign"))], // fixme: a = b = c is possible here...
    S_Compare
}),

// Structure

(S_Parselet = {
    ["@", _, S_Block, (Op::Create("parselet"))]
}),

(S_Block = {
    ["{", _, S_Sequences, _,
        (Op::Expect(Box::new(Match::new_silent("}").into_op()))), _,
        (Op::Create("block"))],
    ["{", _, (Op::Expect(Box::new(Match::new_silent("}").into_op()))), _,
        (Op::Create("block"))]
}),

(S_Sequences = {
    (pos S_Sequence)
}),

(S_Sequence = {
    [(pos S_Item), (Op::Create("sequence"))],
    [T_EOL, (Op::Skip)]
}),

(S_Item = {
    // todo: Recognize aliases
    S_TokenModifier,
    S_Expression
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
                Char::new(ccl![
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

(S_ConstantCallParameter = {
    [S_TokenModifier, _],
    [S_Expression, _, (Op::Error("You may not use expressions here..."))] //fixme...
}),

(S_ConstantCallParameters = {
    [S_ConstantCallParameters, ",", _, S_ConstantCallParameter],
    [S_ConstantCallParameter]
}),

(S_ConstantCall = {
    [T_Constant, "(", _, S_ConstantCallParameters, ")",
        (Op::Create("call_constant"))],
    [T_Constant,
        (Op::Create("call_constant"))]
}),

(S_Token = {
    [T_HeavyString, (Op::Create("match"))],
    [T_LightString, (Op::Create("match_silent"))],
    [S_ConstantCall],
    [S_Parselet]
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

        if let Ok(accept) = res {
            if let Accept::Push(Capture::Value(value)) = accept {
                return Ok(Value::from_ref(value).unwrap());
            }
        }
        else {
            println!("Error: {:#?}", res.err());
        }

        return Err("Parse error".to_string())
    }
}
