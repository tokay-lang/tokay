//! Built-in standard functions

use super::Builtin;
use crate::compiler;
use crate::error::Error;
use crate::value::{Dict, Value};
use crate::vm::{Accept, Capture};

inventory::submit! {
    Builtin {
        name: "ast",
        required: 1,
        signature: "emit value",
        func: |context, args| {
            let emit = args[0].as_ref().unwrap();

            let mut ret = Dict::new();
            ret.insert("emit".to_string(), emit.clone());

            let value = match &args[1] {
                Some(value) => Some(value.clone()),
                None => context
                    .collect(context.capture_start, false, true, false, 0)
                    .unwrap_or(None),
            };

            if let Some(value) = value {
                // List or Dict values are classified as child nodes
                if value.borrow().get_list().is_some() || value.borrow().get_dict().is_some() {
                    ret.insert("children".to_string(), value.clone());
                } else {
                    ret.insert("value".to_string(), value.clone());
                }
            }

            // Store positions of reader start
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

            // Store positions of reader stop
            let current = context.runtime.reader.tell();

            ret.insert(
                "stop_offset".to_string(),
                Value::Addr(current.offset).into_refvalue(),
            );
            ret.insert(
                "stop_row".to_string(),
                Value::Addr(current.row as usize).into_refvalue(),
            );
            ret.insert(
                "stop_col".to_string(),
                Value::Addr(current.col as usize).into_refvalue(),
            );

            Ok(Accept::Return(Some(
                Value::Dict(Box::new(ret)).into_refvalue(),
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "ast_print",
        required: 1,
        signature: "ast",
        func: |_, args| {
            compiler::ast::print(&args[0].as_ref().unwrap().borrow());
            Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "chr",
        required: 1,
        signature: "i",
        func: |_context, args| {
            let i = args[0].as_ref().unwrap().borrow().to_addr();
            println!("i = {}", i);

            Ok(Accept::Push(Capture::Value(
                Value::String(format!("{}", std::char::from_u32(i as u32).unwrap()))
                    .into_refvalue(),
                None,
                10,
            )))
        },
    }
}

inventory::submit! {
    Builtin {
        name: "error",
        required: 1,
        signature: "msg collect",
        func: |context, args| {
            let msg = args[0].as_ref().unwrap();
            let collect = args[1]
                .as_ref()
                .map_or(false, |value| value.borrow().is_true());

            let mut msg = msg.borrow().to_string();

            if collect {
                if let Ok(Some(value)) =
                    context.collect(context.capture_start, false, true, false, 0)
                {
                    let value = value.borrow();

                    if let Value::String(s) = &*value {
                        msg.push_str(&format!(": '{}'", s))
                    } else {
                        msg.push_str(&format!(": {}", value.repr()))
                    }
                }
            }

            Error::new(Some(context.runtime.reader.tell()), msg).into_reject()
        },
    }
}

inventory::submit! {
    Builtin {
        name: "ord",
        required: 1,
        signature: "c",
        func: |_context, args| {
            let c = args[0].as_ref().unwrap().borrow().to_string();
            if c.chars().count() != 1 {
                Error::new(
                    None,
                    format!(
                        "ord() expected single character, but received string of length {}",
                        c.len()
                    ),
                )
                .into_reject()
            } else {
                let c = c.chars().next().unwrap();

                Ok(Accept::Push(Capture::Value(
                    Value::Addr(c as usize).into_refvalue(),
                    None,
                    10,
                )))
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "print",
        required: -1,
        signature: "",
        func: |context, args| {
            //println!("args = {:?}", args);
            if args.len() == 0 {
                if let Some(capture) = context.get_capture(0) {
                    print!("{}", capture.borrow());
                }
            } else {
                for i in 0..args.len() {
                    if i > 0 {
                        print!(" ");
                    }

                    print!("{}", args[i].as_ref().unwrap().borrow().to_string());
                }
            }

            print!("\n");
            Ok(Accept::Push(Capture::Value(
                Value::Void.into_refvalue(),
                None,
                10,
            )))
        },
    }
}
