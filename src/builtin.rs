use crate::compiler::Compiler;
use crate::value::{Dict, List, RefValue, Value};
use crate::vm::*;

type Builtin = fn(&mut Context, args: usize, nargs: Option<Dict>) -> Result<Accept, Reject>;

static BUILTINS: &[(&'static str, i8, bool, Builtin)] = &[
    ("Integer", 0, false, |context, _args, _nargs| {
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
                Value::Integer(value).into_refvalue(),
                5,
            )))
        } else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    }),
    ("print", -1, false, |context, args, _| {
        if args == 0 {
            if let Some(capture) = context.get_capture(0) {
                println!("{:?}", capture);
            } else {
                print!("\n");
            }

            return Ok(Accept::Next);
        }

        for i in 0..args {
            if i > 0 {
                print!(" ");
            }

            print!(
                "{}",
                get_arg(context, args, None, i, None)
                    .unwrap()
                    .borrow()
                    .to_string()
            );
        }

        print!("\n");
        Ok(Accept::Next)
    }), /*
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
                Value::List(Box::new(flatten)).into_refvalue(),
            )))
        }),
        */
];

pub fn register(compiler: &mut Compiler) {
    for i in 0..BUILTINS.len() {
        compiler.set_constant(
            BUILTINS[i].0,
            compiler.define_static(Value::Builtin(i).into_refvalue()),
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

pub fn get_arg(
    context: &Context,
    args: usize,
    nargs: Option<Dict>,
    index: usize,
    name: Option<&'static str>,
) -> Result<RefValue, String> {
    // Try to retrieve argument by name
    if let Some(name) = name {
        if let Some(nargs) = nargs {
            if let Some(value) = nargs.get(name) {
                return Ok(value.clone());
            }
        }
    }

    if index >= args {
        Err(format!(
            "Function requires to access parameter {}, but only {} parameters where given",
            index, args
        ))
    } else {
        //println!("args = {} index = {}, peek = {}", args, index, args - index - 1);
        Ok(context.peek(args - index - 1).unwrap())
    }
}

pub fn call(
    builtin: usize,
    context: &mut Context,
    args: usize,
    mut nargs: Option<Dict>,
) -> Result<Accept, Reject> {
    let builtin = &BUILTINS[builtin];
    let mut result;

    // Allow constant number of minimal parameters
    if builtin.1 >= 0 && args < builtin.1 as usize {
        result = Err(Reject::Error(format!(
            "{} requires for at least {} arguments",
            builtin.0, builtin.1
        )));
    } else if builtin.1 == 0 && args > 0 {
        result = Err(Reject::Error(format!(
            "{} does not accept any sequencial arguments",
            builtin.0
        )));
    } else if nargs.is_some() && !builtin.2 {
        result = Err(Reject::Error(format!(
            "{} does not accept any named arguments",
            builtin.0
        )));
    } else {
        result = (builtin.3)(context, args, nargs);
    }

    context
        .runtime
        .stack
        .truncate(context.runtime.stack.len() - args);
    result
}
