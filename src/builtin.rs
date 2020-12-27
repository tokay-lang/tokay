use crate::tokay::*;
use crate::value::Value;
use crate::compiler::Compiler;

static BUILTINS: &[(&'static str, fn(&mut Context) -> Result<Accept, Reject>)]
= &[
    ("Integer", |context| {
        let mut value: i64 = 0;
        let start = context.runtime.reader.tell();

        while let Some(ch) = context.runtime.reader.peek() {
            if ch < '0' || ch > '9' {
                break;
            }

            value = value * 10 + ch.to_digit(10).unwrap() as i64;
            context.runtime.reader.next();
        }

        if start < context.runtime.reader.tell() {
            Ok(
                Accept::Push(
                    Capture::Value(
                        Value::Integer(value).into_ref()
                    )
                )
            )
        }
        else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    })
];


pub fn register(compiler: &mut Compiler) {
    for i in 0..BUILTINS.len() {
        compiler.set_constant(
            BUILTINS[i].0, Value::Builtin(i).into_ref()
        );
    }
}

pub fn call(builtin: usize, context: &mut Context) -> Result<Accept, Reject> {
    BUILTINS[builtin].1(context)
}
