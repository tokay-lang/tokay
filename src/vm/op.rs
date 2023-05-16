use super::*;
use crate::error::Error;
use crate::reader::Offset;
use crate::value;
use crate::value::{Dict, List, Object, Str, Value};
use std::io;
use std::io::prelude::*;
use std::rc::Rc;

// --- Op ----------------------------------------------------------------------

/**
Atomic operations.

Specifies all atomic level VM code operations to run the Tokay VM.
*/
#[derive(Debug, Clone)]
pub(crate) enum Op {
    Nop,
    Offset(Box<Offset>), // Source offset position for debugging

    // Capture frames
    Frame(usize), // Start new frame with optional relative forward address fuse
    Capture,      // Reset frame capture to current stack size, saving captures
    Extend,       // Extend frame's reader to current position
    Reset,        // Reset frame
    Close,        // Close frame
    Collect,      // Collect stack values from current frame
    InCollect,    // Same as collect, but degrate the parselet level (5) (fixme: This is temporary!)
    Fuse(usize),  // Set frame fuse to relative forward address

    // Loop frames
    Loop(usize), // Loop frame
    Break,       // Ok(Accept::Break)
    LoadBreak,   // Ok(Accept::Break) with value
    Continue,    // Ok(Accept::Continue)

    // Conditional jumps
    ForwardIfTrue(usize),     // Jump forward when TOS is true
    ForwardIfFalse(usize),    // Jump forward when TOS is false
    ForwardIfNotVoid(usize),  // Jump forward when TOS is not void
    ForwardIfConsumed(usize), // Jump forward when frame consumed input

    // Direct jumps
    Forward(usize),  // Jump forward
    Backward(usize), // Jump backward

    // Interrupts
    Skip,                  // Err(Reject::Skip)
    Next,                  // Err(Reject::Next)
    Push,                  // Ok(Accept::Push)
    LoadPush,              // Ok(Accept::Push) with value
    Accept,                // Ok(Accept::Return)
    LoadAccept,            // Ok(Accept::Return) with value
    Repeat,                // Ok(Accept::Repeat)
    LoadRepeat,            // Ok(Accept::Repeat) with value
    Reject,                // Ok(Err::Reject)
    LoadExit,              // Exit with errorcode
    Exit,                  // Exit with 0
    Error(Option<String>), // Error with optional error message (otherwise its expected on stack)

    // Call
    CallOrCopy,          // Load and eventually call stack element without parameters
    Call,                // Call stack element without parameters
    CallArg(usize),      // Call stack element with sequential parameters
    CallArgNamed(usize), // Call stack element with sequential and named parameters
    CallStatic(usize),   // Call static element without parameters
    CallStaticArg(Box<(usize, usize)>), // Call static element with sequential parameters
    CallStaticArgNamed(Box<(usize, usize)>), // Call static element with sequential and named parameters

    // Constants
    LoadStatic(usize), // Push a constant from the statics
    Push0,             // Push Integer(0)
    Push1,             // Push Integer(1)
    PushVoid,          // Push Void
    PushNull,          // Push Null
    PushTrue,          // Push True
    PushFalse,         // Push False

    // Variables, Values and Captures
    LoadGlobal(usize),           // Load global variable
    LoadFast(usize),             // Load local variable by current context
    LoadFastCapture(usize),      // Load capture by known index
    LoadCapture,                 // Load capture by evaluated index
    LoadItem,                    // Load item
    LoadAttr,                    // Load attr
    StoreGlobal(usize),          // Store global variable
    StoreGlobalHold(usize),      // Store global variable and keep tos
    StoreFast(usize),            // Store local variable
    StoreFastHold(usize),        // Store local variable and keep tos
    StoreFastCapture(usize),     // Store capture with known index
    StoreFastCaptureHold(usize), // Store capture with known index and keep tos
    StoreCapture,                // Store capture with evaluated index
    StoreCaptureHold,            // Store capture with evaluated index and keep tos
    StoreItem,                   // Store item
    StoreItemHold,               // Store item and push item reference to tos

    MakeAlias,       // Make key-value-Capture from last two stack items
    MakeList(usize), // Make a List from specified amount of items on stack
    MakeDict(usize), // Make a Dict from specified amount of key-value-pairs on the stack

    // Operations
    Drop,        // drop TOS
    Inv,         // invalidate TOS to empty capture
    Sep,         // separate, ensure TOS value is not shared
    Dup,         // duplicate TOS (creates a new object)
    Copy(usize), // copy indexed element as TOS
    Swap(usize), // swap indexed element with TOS

    UnaryOp(&'static str),  // Operation with one operand
    BinaryOp(&'static str), // Operation with two operands
}

impl Op {
    pub fn execute(ops: &[Op], context: &mut Context) -> Result<Accept, Reject> {
        if ops.len() == 0 {
            return Ok(Accept::Next);
            //return Ok(Accept::Push(Capture::Empty));
        }

        fn dump(ops: &[Op], context: &Context, ip: usize) {
            for (i, op) in ops.iter().enumerate() {
                context.log(&format!(
                    "{}{:03} {:?}",
                    if i == ip { ">" } else { " " },
                    i,
                    op
                ));
            }
        }

        assert!(context.frames.len() == 0);

        // ---------------------------------------------------------------------

        let mut ip = 0; // Instruction pointer
        let mut state = Ok(Accept::Next);

        while ip < ops.len() {
            let op = &ops[ip];

            // Debug
            if context.runtime.debug == 3 {
                context.log(&format!("{:03}:{:?}", ip, op));
            } else if context.runtime.debug > 3 {
                if context.runtime.debug > 5 {
                    // Skip any Nop-Operations
                    if matches!(op, Op::Nop | Op::Offset(_)) {
                        ip += 1;
                        continue;
                    }
                }

                // Dump entire code
                context.log("--- Code ---");
                dump(ops, context, ip);

                // Dump stack and frames
                if context.runtime.debug > 4 {
                    context.log("--- Stack ---");
                    for i in 0..context.runtime.stack.len() {
                        context.log(&format!(" {:03} {:?}", i, context.runtime.stack[i]));
                    }

                    context.log("--- Frames ---");
                    for i in 0..context.frames.len() {
                        context.log(&format!(" {:03} {}", i, context.frames[i]));
                    }

                    context.log(&format!(" {:03} {}", context.frames.len(), context.frame));
                }

                // Step-by-step
                if context.runtime.debug > 5 {
                    let _ = io::stdin().read(&mut [0u8]).unwrap();
                }
            }

            // Execute instruction
            state = match op {
                Op::Nop => Ok(Accept::Next),

                Op::Offset(offset) => {
                    context.source_offset = Some(**offset);
                    Ok(Accept::Next)
                }

                // Frames
                Op::Frame(fuse) => {
                    context.frames.push(context.frame);
                    context.frame = Frame {
                        fuse: if *fuse > 0 { Some(ip + *fuse) } else { None },
                        capture_start: context.runtime.stack.len(),
                        reader_start: context.runtime.reader.tell(),
                    };

                    Ok(Accept::Next)
                }

                Op::Capture => {
                    context.frame.capture_start = context.runtime.stack.len();
                    Ok(Accept::Next)
                }

                Op::Extend => {
                    context.frame.reader_start = context.runtime.reader.tell();
                    Ok(Accept::Next)
                }

                Op::Reset => {
                    context.runtime.stack.truncate(context.frame.capture_start);
                    context.runtime.reader.reset(context.frame.reader_start);
                    Ok(Accept::Next)
                }

                Op::Close => {
                    context.frame = context.frames.pop().unwrap();
                    Ok(Accept::Next)
                }

                Op::Collect => Ok(Accept::Push(context.collect(
                    context.frame.capture_start,
                    false,
                    context.runtime.debug > 5,
                ))),

                Op::InCollect => {
                    let mut capture = context.collect(
                        context.frame.capture_start,
                        false,
                        context.runtime.debug > 5,
                    );

                    if capture.get_severity() > 5 {
                        capture.set_severity(5);
                    }

                    Ok(Accept::Push(capture))
                }

                Op::Fuse(addr) => {
                    context.frame.fuse = Some(ip + *addr);
                    Ok(Accept::Next)
                }

                // Loops
                Op::Loop(size) => {
                    context.loops.push(Loop {
                        frames: context.frames.len(),
                        start: ip + 1,
                        end: ip + *size,
                    });
                    Ok(Accept::Next)
                }

                Op::Break | Op::LoadBreak => {
                    let current = context.loops.pop().unwrap();

                    // Save value?
                    let value = if matches!(op, Op::LoadBreak) {
                        Some(context.pop())
                    } else {
                        None
                    };

                    // Discard all open frames inside current loop.
                    while context.frames.len() > current.frames {
                        context.frame = context.frames.pop().unwrap();
                    }

                    context.runtime.stack.truncate(context.frame.capture_start);

                    // Jump behind loop
                    ip = current.end;

                    // Break will always leave a value, either defined or empty capture
                    Ok(if let Some(value) = value {
                        Accept::Push(Capture::Value(value, None, 10))
                    } else {
                        context.runtime.stack.push(Capture::Empty);
                        Accept::Hold
                    })
                }

                Op::Continue => {
                    let current = context
                        .loops
                        .last()
                        .expect("Op::Continue used outside of a loop frame");

                    // Discard all open frames inside current loop.
                    while context.frames.len() > current.frames {
                        context.frame = context.frames.pop().unwrap();
                    }

                    context.runtime.stack.truncate(context.frame.capture_start);

                    // Jump to loop start.
                    ip = current.start;

                    Ok(Accept::Hold)
                }

                // Conditional jumps
                Op::ForwardIfTrue(goto) => {
                    if context.pop().is_true() {
                        ip += goto;
                    } else {
                        ip += 1;
                    }

                    Ok(Accept::Hold)
                }

                Op::ForwardIfFalse(goto) => {
                    if !context.pop().is_true() {
                        ip += goto;
                    } else {
                        ip += 1;
                    }

                    Ok(Accept::Hold)
                }

                Op::ForwardIfNotVoid(goto) => {
                    if !context.pop().is_void() {
                        ip += goto;
                    } else {
                        ip += 1;
                    }

                    Ok(Accept::Hold)
                }

                Op::ForwardIfConsumed(goto) => {
                    if context.frame.reader_start != context.runtime.reader.tell() {
                        ip += goto;
                        Ok(Accept::Hold)
                    } else {
                        Ok(Accept::Next)
                    }
                }

                Op::Forward(goto) => {
                    ip += goto;
                    Ok(Accept::Hold)
                }

                Op::Backward(goto) => {
                    ip -= goto;
                    Ok(Accept::Hold)
                }

                // Interrupts
                Op::Skip => Err(Reject::Skip), // currently not used.
                Op::Next => Err(Reject::Next),

                Op::Push => Ok(Accept::Push(Capture::Empty)),
                Op::LoadPush => {
                    let value = context.pop();
                    Ok(Accept::Push(Capture::Value(value, None, 15))) // high severity for override required here
                }

                Op::Accept => Ok(Accept::Return(None)),
                Op::LoadAccept => {
                    let value = context.pop();
                    Ok(Accept::Return(Some(value)))
                }
                Op::Repeat => Ok(Accept::Repeat(None)),
                Op::LoadRepeat => {
                    let value = context.pop();
                    Ok(Accept::Repeat(Some(value)))
                }
                Op::Reject => Err(Reject::Return),
                Op::LoadExit => {
                    std::process::exit(context.pop().to_i64()? as i32);
                }
                Op::Exit => std::process::exit(0),

                Op::Error(msg) => {
                    if let Some(msg) = msg {
                        Error::new(Some(context.frame.reader_start), msg.clone()).into()
                    } else {
                        Error::new(Some(context.frame.reader_start), context.pop().to_string())
                            .into()
                    }
                }

                // Calls
                Op::CallOrCopy => {
                    let value = context.pop();

                    if false && context.runtime.debug > 3 {
                        println!(
                            "CallOrCopy is_callable={:?} is_mutable={:?}",
                            value.is_callable(true),
                            value.is_mutable()
                        )
                    }

                    if value.is_callable(true) {
                        // Call the value without parameters
                        value.call_direct(context, 0, None)
                    } else if value.is_mutable() {
                        // Push a reference to the value
                        context.push(value)
                    } else {
                        // Push a copy of the value
                        context.push(value.borrow().clone().into())
                    }
                }

                Op::Call => {
                    let target = context.pop();
                    target.call_direct(context, 0, None)
                }

                Op::CallArg(args) => {
                    let target = context.pop();
                    target.call_direct(context, *args, None)
                }

                Op::CallArgNamed(args) => {
                    let target = context.pop();
                    let nargs = Value::from(context.pop());

                    if let Some(nargs) = nargs.into_object::<Dict>() {
                        target.call_direct(context, *args, Some(nargs))
                    } else {
                        panic!("nargs operand required to be dict")
                    }
                }

                Op::CallStatic(addr) => {
                    context.program.statics[*addr].call_direct(context, 0, None)
                }

                Op::CallStaticArg(addr_args) => {
                    context.program.statics[addr_args.0].call_direct(context, addr_args.1, None)
                    //println!("CallStaticArg returns {:?}", ret);
                }

                Op::CallStaticArgNamed(addr_args) => {
                    let nargs = Value::from(context.pop());

                    if let Some(nargs) = nargs.into_object::<Dict>() {
                        context.program.statics[addr_args.0].call_direct(
                            context,
                            addr_args.1,
                            Some(nargs),
                        )
                    } else {
                        panic!("nargs operand required to be dict")
                    }
                }

                // Variables and values
                Op::LoadStatic(addr) => {
                    let value = &context.program.statics[*addr];
                    context.push(value.borrow().clone().into())
                }
                Op::Push0 => context.push(value!(0i64)),
                Op::Push1 => context.push(value!(1i64)),
                Op::PushVoid => context.push(value!(void)),
                Op::PushNull => context.push(value!(null)),
                Op::PushTrue => context.push(value!(true)),
                Op::PushFalse => context.push(value!(false)),

                Op::LoadGlobal(addr) => context.load(*addr),
                Op::LoadFast(addr) => context.load(context.stack_start + *addr),

                Op::LoadFastCapture(index) => {
                    let value = context.get_capture(*index).unwrap_or(value!(void));
                    context.push(value)
                }

                Op::LoadCapture => {
                    let index = context.pop();
                    let index = index.borrow();

                    let value = if let Some(alias) = index.object::<Str>() {
                        context
                            .get_capture_by_name(alias.as_str())
                            .unwrap_or(value!(void))
                    } else {
                        context
                            .get_capture(index.to_usize()?)
                            .unwrap_or(value!(void))
                    };

                    context.push(value)
                }

                Op::LoadItem => {
                    let item = context.pop();
                    let object = context.pop();

                    match object.call_method("get_item", Some(context), vec![item]) {
                        Ok(Some(value)) => context.push(value),
                        Ok(None) => Ok(Accept::Next),
                        Err(msg) => Err(Reject::from(msg)),
                    }
                }

                Op::LoadAttr => {
                    let attr = context.pop();
                    let attr = attr.borrow();
                    let value = context.pop();

                    match value.create_method(attr.object::<Str>().unwrap().as_str()) {
                        Ok(value) => context.push(value),
                        Err(err) => err.into(),
                    }
                }

                Op::StoreGlobal(addr) => {
                    // todo: bounds checking?
                    let value = context.pop().ref_or_copy();
                    context.runtime.stack[*addr] = Capture::Value(value, None, 0);
                    Ok(Accept::Push(Capture::Empty))
                }

                Op::StoreGlobalHold(addr) => {
                    // todo: bounds checking?
                    let value = context.peek().ref_or_copy();
                    context.runtime.stack[*addr] = Capture::Value(value, None, 0);
                    Ok(Accept::Next)
                }

                Op::StoreFast(addr) => {
                    // todo: bounds checking?
                    let value = context.pop().ref_or_copy();
                    context.runtime.stack[context.stack_start + *addr] =
                        Capture::Value(value, None, 0);
                    Ok(Accept::Push(Capture::Empty))
                }

                Op::StoreFastHold(addr) => {
                    // todo: bounds checking?
                    let value = context.peek().ref_or_copy();
                    context.runtime.stack[context.stack_start + *addr] =
                        Capture::Value(value, None, 0);
                    Ok(Accept::Next)
                }

                Op::StoreFastCapture(index) => {
                    let value = context.pop().ref_or_copy();

                    context.set_capture(*index, value);
                    Ok(Accept::Push(Capture::Empty))
                }

                Op::StoreFastCaptureHold(index) => {
                    let value = context.peek().ref_or_copy();

                    context.set_capture(*index, value);
                    Ok(Accept::Next)
                }

                Op::StoreCapture | Op::StoreCaptureHold => {
                    let index = context.pop();
                    let index = index.borrow();

                    if let Some(alias) = index.object::<Str>() {
                        if matches!(op, Op::StoreCapture) {
                            let value = context.pop().ref_or_copy();
                            context.set_capture_by_name(alias, value);
                            Ok(Accept::Push(Capture::Empty))
                        } else {
                            let value = context.peek().ref_or_copy();
                            context.set_capture_by_name(alias, value);
                            Ok(Accept::Next)
                        }
                    } else {
                        if matches!(op, Op::StoreCapture) {
                            let value = context.pop().ref_or_copy();
                            context.set_capture(index.to_usize()?, value);
                            Ok(Accept::Push(Capture::Empty))
                        } else {
                            let value = context.peek().ref_or_copy();
                            context.set_capture(index.to_usize()?, value);
                            Ok(Accept::Next)
                        }
                    }
                }

                Op::StoreItem | Op::StoreItemHold => {
                    let item = context.pop();
                    let object = context.pop();
                    let value = context.pop();

                    match object.call_method("set_item", Some(context), vec![item, value]) {
                        Ok(value) => {
                            let value = value.unwrap(); // setitem must always return a value!

                            if matches!(op, Op::StoreItemHold) {
                                context.push(value)
                            } else {
                                Ok(Accept::Next)
                            }
                        }
                        Err(msg) => Err(Reject::from(msg)),
                    }
                }

                Op::MakeAlias => {
                    let name = context.pop();

                    match context.runtime.stack.last_mut().unwrap() {
                        Capture::Range(_, alias, ..) | Capture::Value(_, alias, ..) => {
                            *alias = Some(name);
                        }

                        empty => {
                            *empty = Capture::Value(value!(void), Some(name), 0);
                        }
                    }

                    Ok(Accept::Next)
                }

                Op::MakeList(count) => {
                    let mut list = List::new();

                    for _ in 0..*count {
                        list.insert(0, context.pop());
                    }

                    context.push(RefValue::from(list))
                }

                Op::MakeDict(count) => {
                    let mut dict = Dict::new();

                    for _ in 0..*count {
                        let key = context.pop();
                        let value = context.pop();
                        dict.insert(key, value);
                    }

                    context.push(RefValue::from(dict))
                }

                // Operations
                Op::Drop => {
                    context.pop();
                    Ok(Accept::Next)
                }

                Op::Inv => {
                    context.pop();
                    Ok(Accept::Push(Capture::Empty))
                }

                Op::Sep => {
                    let mut value = context.pop();

                    if Rc::strong_count(&value) > 1 {
                        value = RefValue::from({
                            let inner = value.borrow();
                            inner.clone()
                        });
                    }

                    context.push(value)
                }

                Op::Dup => {
                    let value = context.peek();
                    let value = value.borrow();
                    context.push(value.clone().into())
                }

                Op::Copy(index) => {
                    assert!(*index > 0);

                    let index = context.runtime.stack.len() - index;
                    context
                        .runtime
                        .stack
                        .push(context.runtime.stack[index].clone());

                    Ok(Accept::Next)
                }

                Op::Swap(index) => {
                    assert!(*index > 1);

                    let index = context.runtime.stack.len() - index;
                    let tos = context.runtime.stack.pop().unwrap();

                    context
                        .runtime
                        .stack
                        .push(context.runtime.stack[index].clone());
                    context.runtime.stack[index] = tos;

                    Ok(Accept::Next)
                }

                Op::UnaryOp(op) => {
                    let value = context.pop();
                    context.push(value.unary_op(op)?)
                }

                Op::BinaryOp(op) => {
                    let last = context.pop();
                    let first = context.pop();
                    context.push(first.binary_op(last, op)?)
                }
            };

            // Debug
            if context.runtime.debug > 3 {
                context.log(&format!("ip = {} state = {:?}", ip, state));
            }

            match state {
                Ok(Accept::Hold) => state = Ok(Accept::Next),
                Ok(Accept::Next) => ip += 1,
                Ok(Accept::Push(capture)) if ip + 1 < ops.len() => {
                    context.runtime.stack.push(capture);
                    state = Ok(Accept::Next);
                    ip += 1;
                }
                Err(Reject::Next) if context.frames.len() > 0 => loop {
                    context.runtime.stack.truncate(context.frame.capture_start);
                    context.runtime.reader.reset(context.frame.reader_start);

                    if let Some(fuse) = context.frame.fuse {
                        if fuse > ip {
                            ip = fuse;
                            break;
                        }
                    }

                    if context.frames.len() == 0 {
                        return Err(Reject::Next);
                    }

                    context.frame = context.frames.pop().unwrap();
                },
                Err(Reject::Error(ref mut err)) => {
                    // Patch context source position on error,
                    // if no other position already set
                    if let Some(source_offset) = context.source_offset {
                        err.patch_offset(source_offset);
                    }

                    break;
                }
                _ => break,
            }
        }

        // Clear all frames except the base frame
        if !context.frames.is_empty() {
            context.frames.truncate(1);
            context.frame = context.frames.pop().unwrap();
        }

        // In case state is Accept::Next, try to return a capture
        if matches!(state, Ok(Accept::Next)) {
            // Either take $0 when set
            if let Capture::Value(value, ..) =
                &mut context.runtime.stack[context.frame.capture_start - 1]
            {
                state = Ok(Accept::Push(Capture::Value(
                    value.clone(),
                    None,
                    context.parselet.severity,
                )));
            // Otherwise, push last value
            } else if context.runtime.stack.len() > context.frame.capture_start {
                state = Ok(Accept::Push(context.runtime.stack.pop().unwrap())
                    .into_push(context.parselet.severity));
            }
            /*
            // Else, push empty capture
            else {
                state = Ok(Accept::Push(Capture::Empty))
            }
            */
        }

        if context.runtime.debug > 3 {
            context.log(&format!("exit state = {:?}", state));
        }

        state
    }
}
