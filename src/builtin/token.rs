//! Built-in hard-coded tokens

use super::Builtin;
use crate::value::Value;
use crate::vm::{Accept, Capture, Reject};

inventory::submit! {
    Builtin {
        name: "Identifier", // Matching C-style identifiers
        required: 0,
        signature: "",
        func: |context, _args| {
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
                    None,
                    5,
                )))
            } else {
                Err(Reject::Next)
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "Integer", // Matching 64-bit integers directly
        required: 0,
        signature: "",
        func: |context, _args| {
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
                    None,
                    5,
                )))
            } else {
                context.runtime.reader.reset(start);
                Err(Reject::Next)
            }
        },
    }
}

inventory::submit! {
    Builtin {
        name: "Word", // Matching words made of letters
        required: 0,
        signature: "min max",
        func: |context, args| {
            let min = &args[0];
            let max = &args[1];

            let mut count: usize = 0;

            while let Some(ch) = context.runtime.reader.peek() {
                if !ch.is_alphabetic() {
                    break;
                }

                context.runtime.reader.next();
                count += 1;
            }

            if count > 0 {
                if let Some(min) = min {
                    if count < min.borrow().to_addr() {
                        count = 0;
                    }
                }

                if let Some(max) = max {
                    if count > max.borrow().to_addr() {
                        count = 0;
                    }
                }
            }

            if count > 0 {
                Ok(Accept::Push(Capture::Range(
                    context.runtime.reader.capture_last(count),
                    None,
                    5,
                )))
            } else {
                Err(Reject::Next)
            }
        },
    }
}
