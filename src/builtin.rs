use crate::compiler::Compiler;
use crate::error::Error;
use crate::value::{Dict, RefValue, Value};
use crate::vm::*;

type Builtin = fn(&mut Context, args: Vec<RefValue>, nargs: Option<Dict>) -> Result<Accept, Reject>;

static BUILTINS: &[(&'static str, i8, bool, Builtin)] = &[
    ("Integer", 0, false, |context, _args, _nargs| {
        let mut neg = false;
        let mut value: i64 = 0;

        // Sign
        if let Some(ch) = context.runtime.reader.peek() {
            if ch == '-' || ch == '+' {
                neg = ch == '-';
                context.runtime.reader.next();
            }
        }

        let start = context.runtime.reader.tell();

        // Digits
        while let Some(ch) = context.runtime.reader.peek() {
            if ch < '0' || ch > '9' {
                break;
            }

            value = value * 10 + ch.to_digit(10).unwrap() as i64;
            context.runtime.reader.next();
        }

        if start.offset < context.runtime.reader.tell().offset {
            if neg {
                value = -value;
            }

            Ok(Accept::Push(Capture::Value(
                Value::Integer(value).into_refvalue(),
                5,
            )))
        } else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    }),
    ("Name", 0, false, |context, _args, _nargs| {
        let mut count: usize = 0;

        while let Some(ch) = context.runtime.reader.peek() {
            if !ch.is_alphanumeric() {
                break;
            }

            context.runtime.reader.next();
            count += 1;
        }

        if count > 0 {
            Ok(Accept::Push(Capture::Range(
                context.runtime.reader.capture_last(count),
                5,
            )))
        } else {
            Err(Reject::Next)
        }
    }),
    ("Cname", 0, false, |context, _args, _nargs| {
        if let Some(ch) = context.runtime.reader.peek() {
            if !ch.is_alphabetic() && ch != '_' {
                return Err(Reject::Next);
            }

            context.runtime.reader.next();
        } else {
            return Err(Reject::Next);
        }

        let mut count: usize = 1;

        while let Some(ch) = context.runtime.reader.peek() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }

            context.runtime.reader.next();
            count += 1;
        }

        if count > 0 {
            Ok(Accept::Push(Capture::Range(
                context.runtime.reader.capture_last(count),
                5,
            )))
        } else {
            Err(Reject::Next)
        }
    }),
    ("Whitespace", 0, false, |context, _args, _nargs| {
        let mut count: usize = 0;

        while let Some(ch) = context.runtime.reader.peek() {
            if !ch.is_whitespace() {
                break;
            }

            context.runtime.reader.next();
            count += 1;
        }

        if count > 0 {
            Ok(Accept::Push(Capture::Range(
                context.runtime.reader.capture_last(count),
                5,
            )))
        } else {
            Err(Reject::Next)
        }
    }),
    ("error", 1, false, |context, args, _| {
        Error::new(
            Some(context.runtime.reader.tell()),
            get_arg(&args, None, Some(0), None)
                .unwrap()
                .borrow()
                .to_string(),
        )
        .into_reject()
    }),
    ("collect", -1, true, |context, args, nargs| {
        let emit = get_arg(&args, nargs.as_ref(), Some(0), Some("emit"));
        let mut value = get_arg(&args, nargs.as_ref(), Some(1), Some("value"));

        // In case no value is set, collect them from the current context.
        if value.is_err() {
            if let Some(capture) = context.collect(context.capture_start, false, false, 0) {
                value = Ok(capture.as_value(context.runtime));
            }
        }

        let mut ret = Dict::new();

        if let Ok(emit) = emit {
            ret.insert("emit".to_string(), emit.clone());
        }

        if let Ok(value) = value {
            // List or Dict values are classified as child nodes
            if value.borrow().get_list().is_some() || value.borrow().get_dict().is_some() {
                ret.insert("children".to_string(), value);
            } else {
                ret.insert("value".to_string(), value);
            }
        }

        ret.insert(
            "offset".to_string(),
            Value::Addr(context.reader_start.offset).into_refvalue(),
        );
        ret.insert(
            "row".to_string(),
            Value::Addr(context.reader_start.row as usize).into_refvalue(),
        );
        ret.insert(
            "col".to_string(),
            Value::Addr(context.reader_start.col as usize).into_refvalue(),
        );

        /*
        let current = context.runtime.reader.tell();

        ret.insert(
            "end_offset".to_string(),
            Value::Addr(current.offset).into_refvalue(),
        );
        ret.insert(
            "end_row".to_string(),
            Value::Addr(current.row as usize).into_refvalue(),
        );
        ret.insert(
            "end_col".to_string(),
            Value::Addr(current.col as usize).into_refvalue(),
        );
        */

        Ok(Accept::Return(Some(
            Value::Dict(Box::new(ret)).into_refvalue(),
        )))
    }),
    ("print", -1, false, |context, args, _| {
        if args.len() == 0 {
            if let Some(capture) = context.get_capture(0) {
                println!("{}", capture.borrow());
            } else {
                print!("\n");
            }

            return Ok(Accept::Next);
        }

        for i in 0..args.len() {
            if i > 0 {
                print!(" ");
            }

            print!(
                "{}",
                get_arg(&args, None, Some(i), None)
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

/// Retrieve builtin by name.
pub fn get(ident: &str) -> Option<usize> {
    for i in 0..BUILTINS.len() {
        if BUILTINS[i].0 == ident {
            return Some(i);
        }
    }

    None
}

/// Check if specific builtin is consumable by identifier
pub fn is_consumable(builtin: usize) -> bool {
    Compiler::identifier_is_consumable(BUILTINS[builtin].0)
}

/// Examine arguments from constant call.
pub fn get_arg(
    args: &Vec<RefValue>,
    nargs: Option<&Dict>,
    index: Option<usize>,
    name: Option<&'static str>,
) -> Result<RefValue, String> {
    // Try to retrieve argument by name
    if let Some(name) = name {
        if let Some(nargs) = nargs {
            if let Some(value) = nargs.get(name) {
                return Ok(value.clone());
            }
        }

        if index.is_none() {
            return Ok(Value::Void.into_refvalue());
        }
    }

    let index = index.unwrap();

    if index >= args.len() {
        Err(format!(
            "Function requires to access parameter {}, but only {} parameters where given",
            index,
            args.len()
        ))
    } else {
        //println!("args = {} index = {}, peek = {}", args, index, args - index - 1);
        Ok(args[index].clone())
    }
}

// Call builtin from the VM.
pub fn call(
    builtin: usize,
    context: &mut Context,
    args: usize,
    nargs: Option<Dict>,
) -> Result<Accept, Reject> {
    let builtin = &BUILTINS[builtin];

    // First, collect all arguments and turn them into RefValues
    let args: Vec<Capture> = context
        .runtime
        .stack
        .drain(context.runtime.stack.len() - args..)
        .collect();
    let args: Vec<RefValue> = args
        .into_iter()
        .map(|item| item.as_value(context.runtime))
        .collect();

    let result;

    // Require constant number of minimal parameters
    if builtin.1 >= 0
        && (args.len() < builtin.1 as usize
            && (!builtin.2
                || builtin.2 && {
                    if let Some(nargs) = nargs.as_ref() {
                        nargs.len()
                    } else {
                        0
                    }
                } < builtin.1 as usize))
    {
        result = Error::new(
            None,
            format!(
                "'{}' requires for at least {} arguments",
                builtin.0, builtin.1
            ),
        )
        .into_reject();
    } else if builtin.1 == 0 && args.len() > 0 {
        result = Error::new(
            None,
            format!("'{}' does not accept any sequencial arguments", builtin.0),
        )
        .into_reject();
    } else if nargs.is_some() && !builtin.2 {
        result = Error::new(
            None,
            format!("'{}' does not accept any named arguments", builtin.0),
        )
        .into_reject();
    } else {
        result = (builtin.3)(context, args, nargs);
    }

    result
}

// Checks if builtin is callable with or without arguments
pub fn is_callable(builtin: usize, with_arguments: bool) -> bool {
    let builtin = &BUILTINS[builtin];

    (!with_arguments && builtin.1 < 0 || (builtin.1 == 0 && !builtin.2))
        || with_arguments && (builtin.1 != 0 || builtin.2)
}
