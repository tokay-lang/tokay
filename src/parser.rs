use crate::reader::Reader;
use crate::tokay::*;
use crate::value::{RefValue, Value};
use crate::compiler::Compiler;
use crate::{tokay, tokay_item, ccl};


pub struct TokayParser(Program);

macro_rules! emit {
    ( $string:literal ) => {
        Op::Push(Value::String($string.to_string()).into_ref()).into_box()
    }
}

impl TokayParser {
    pub fn new() -> Self {
        Self(
            tokay!({
// ----------------------------------------------------------------------------

(_ = {
    [" "],
    ["#", (Char::until('\n'))]
}),

(T_EOL = {
    [
        (Char::char('\n')),
        _,
        (Op::Skip)
    ]
}),

// Basic Tokens (might probably be replaced by something native, pluggable one)

/*
(Identifier = {
    [
        (Char::new(
            ccl!['A'..='Z', 'a'..='z', '_'..='_'])).into_box()
        ),
        (Op::Token(Chars::new(
            ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        ).into_box())
    ]
}),
*/

(T_Variable = {
    [
        (Char::new(ccl!['a'..='z'])),
        (Repeat::muted_optional(
            Char::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::LoadCapture(0)),
        (Op::Create("variable"))
    ]
}),

(T_Constant = {
    [
        (Char::new(ccl!['A'..='Z', '_'..='_'])),
        (Repeat::muted_optional(
            Char::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        )),
        (Op::LoadCapture(0)),
        (Op::Create("constant"))
    ]
}),

(T_HeavyString = {
    [
        "\"",
        (Char::until('"')),     //fixme: Escape sequences (using Until built-in parselet)
        "\"",
        (Op::LoadCapture(2)),
        (Op::Create("string"))
    ]
}),

(T_LightString = {
    [
        "\'",
        (Char::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
        "\'",
        (Op::LoadCapture(2)),
        (Op::Create("string"))
    ]
}),

// Structure

(S_Parselet = {
    ["@", _, S_Block, (Op::Create("parselet"))]
}),

(S_Block = {
    ["{", _, S_Sequences, _, "}", _, (Op::Create("block"))]
}),

(S_Sequences = {
    [S_Sequences, S_Sequence],
    [S_Sequence]
}),

(S_Sequence = {
    (S_Sequence1 = {
        [S_Sequence1, S_Atomic],
        [S_Atomic]
    }),

    [T_EOL, (Op::Skip)],
    [T_Constant, _, "=", _, S_Parselet, _, (Op::Create("assign_constant"))],
    [S_Sequence1, (Op::Create("sequence"))]
}),

(S_Atomic = {
    [T_HeavyString, _],
    [T_LightString, _],
    [T_Constant, _]
}),

[S_Sequence]

// ----------------------------------------------------------------------------
            })
        )
    }

    pub fn parse(&self, code: &'static str) -> Result<Value, String> {
        //self.0.dump();

        let mut reader = Reader::new(
            Box::new(std::io::Cursor::new(code))
        );

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
