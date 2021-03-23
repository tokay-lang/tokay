use crate::error::Error;
use crate::value::{Dict, RefValue, Value};
use crate::vm::*;

type Builtin = fn(&mut Context, args: Vec<RefValue>, nargs: Option<Dict>) -> Result<Accept, Reject>;

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
    ("error", 1, false, |context, args, _| {
        Error::new(
            Some(context.runtime.reader.tell()),
            get_arg(&args, None, 0, None).unwrap().borrow().to_string(),
        )
        .into_reject()
    }),
    ("collect", 1, true, |context, args, nargs| {
        let emit = get_arg(&args, nargs.as_ref(), 0, Some("emit")).unwrap();
        let mut value = get_arg(&args, nargs.as_ref(), 1, Some("value"));

        // In case no value is set, collect them from the current context.
        if value.is_err() {
            if let Some(capture) = context.collect(context.capture_start, false, false, 0) {
                value = Ok(capture.as_value(context.runtime));
            }
        }

        let mut ret = Dict::new();

        ret.insert("emit".to_string(), emit.clone());

        // List or Dict values are classified as child nodes
        if let Ok(value) = value {
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
                get_arg(&args, None, i, None).unwrap().borrow().to_string()
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

/// Examine arguments from constant call.
pub fn get_arg(
    args: &Vec<RefValue>,
    nargs: Option<&Dict>,
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

    // Allow constant number of minimal parameters
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
