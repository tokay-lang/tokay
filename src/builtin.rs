use crate::compiler::Compiler;
use crate::value::{Dict, List, Value};
use crate::vm::*;

static BUILTINS: &[(&'static str, fn(&mut Context) -> Result<Accept, Reject>)] = &[
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

        if start.offset < context.runtime.reader.tell().offset {
            Ok(Accept::Push(Capture::Value(
                Value::Integer(value).into_ref(),
                5,
            )))
        } else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    }),
    ("flatten", |context| {
        let mut max = 0;
        let mut flatten = List::new();

        for capture in context.drain_captures() {
            match capture {
                Capture::Named(_, _) => {}
                Capture::Range(_, severity) | Capture::Value(_, severity) if severity >= max => {
                    if severity > max {
                        max = severity;
                        flatten.clear();
                    }
                }
                _ => continue,
            }

            let value = capture.as_value(context.runtime);
            let peek = value.borrow();

            if let Value::List(list) = &*peek {
                flatten.extend(list.iter().cloned());
            } else {
                flatten.push(value.clone());
            }
        }

        Ok(Accept::Return(Some(
            Value::List(Box::new(flatten)).into_ref(),
        )))
    }),
];

pub fn register(compiler: &mut Compiler) {
    for i in 0..BUILTINS.len() {
        compiler.set_constant(
            BUILTINS[i].0,
            compiler.define_static(Value::Builtin(i).into_ref()),
        );
    }
}

pub fn get(ident: &'static str) -> Option<usize> {
    for i in 0..BUILTINS.len() {
        if BUILTINS[i].0 == ident {
            return Some(i);
        }
    }

    None
}

pub fn call(
    builtin: usize,
    context: &mut Context,
    args: usize,
    _nargs: Option<Dict>,
) -> Result<Accept, Reject> {
    if args > 0 {
        unimplemented!("Builtins with parameters are yet unimplemented")
    }

    BUILTINS[builtin].1(context)
}
