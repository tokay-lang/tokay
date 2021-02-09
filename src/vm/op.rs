use std::collections::HashMap;
use std::iter::FromIterator;

use super::*;
use crate::reader::{Range, Reader};
use crate::value::{Dict, List, RefValue, Value};

// --- Op ----------------------------------------------------------------------

/**
Atomic operations.

Specifies atomic level operations like running a parsable structure or running
VM code.
*/
#[derive(Debug)]
pub enum Op {
    Nop,
    Usage(usize), // (yet) unresolved usage

    // Parsing constructs
    Empty, // The empty word

    Scanable(Box<dyn Scanable>), // Scanable item
    Runable(Box<dyn Runable>),   // Runable item

    Peek(Box<Op>), // Peek-operation (todo: this should be a Runable)
    Not(Box<Op>),  // Not-predicate (todo: this should be a Runable)

    // Call
    TryCall,
    Call,
    CallStatic(usize),

    // Debuging and error reporting
    Print,               // todo: make this a builtin
    Debug(&'static str), // todo: make this a builtin
    Error(&'static str), // todo: make this a builtin
    Expect(Box<Op>),     // todo: make this a builtin

    // AST construction
    Create(&'static str), // todo: make this a builtin
    Lexeme(&'static str), // todo: make this a builtin

    // Interrupts
    Skip,
    LoadAccept,
    Reject,

    // Constants
    LoadStatic(usize),
    PushTrue,
    PushFalse,
    PushVoid,

    // Variables & Values
    LoadGlobal(usize),
    LoadFast(usize),
    StoreGlobal(usize),
    StoreFast(usize),
    LoadFastCapture(usize),
    LoadCapture,
    StoreFastCapture(usize),
    StoreCapture,

    // Operations
    Add,
    Sub,
    Div,
    Mul,
}

impl Op {
    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn into_kleene(self) -> Self {
        Repeat::kleene(self)
    }

    pub fn into_positive(self) -> Self {
        Repeat::positive(self)
    }

    pub fn into_optional(self) -> Self {
        Repeat::optional(self)
    }

    /*
        Utility function to replace an Op during tranformation
        either by another Op or by a sequence.
    */
    pub(super) fn replace_usage(&mut self, usages: &mut Vec<Vec<Op>>) {
        if let Op::Usage(usage) = self {
            match usages[*usage].len() {
                0 => panic!("Invalid"),
                1 => {
                    *self = usages[*usage].drain(..).next().unwrap();
                }
                _ => {
                    // This could be an error, but we can also
                    // make a sequence from it...
                    *self =
                        Sequence::new(usages[*usage].drain(..).map(|item| (item, None)).collect());
                }
            }
        }
    }
}

impl Runable for Op {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        //println!("RUN {:?}", self);

        match self {
            Op::Nop => Ok(Accept::Next),

            Op::Usage(_) => panic!(
                "{:?} can't be run; Trying to run an unresolved program?",
                self
            ),

            Op::Empty => Ok(Accept::Push(Capture::Empty)),

            Op::Scanable(scanable) => scanable.scan(&mut context.runtime.reader),
            Op::Runable(runable) => runable.run(context),

            Op::TryCall => {
                let value = context.pop();
                if value.borrow().is_callable() {
                    value.borrow().call(context)
                } else {
                    Ok(Accept::Push(Capture::Value(value.clone(), 1)))
                }
            }

            Op::Call => {
                let value = context.pop();
                let value = value.borrow();
                value.call(context)
            }

            Op::CallStatic(addr) => context.runtime.program.statics[*addr]
                .borrow()
                .call(context),

            Op::Peek(p) => {
                let reader_start = context.runtime.reader.tell();
                let ret = p.run(context);
                context.runtime.reader.reset(reader_start);
                ret
            }

            Op::Not(p) => {
                if p.run(context).is_ok() {
                    Err(Reject::Next)
                } else {
                    Ok(Accept::Next)
                }
            }

            Op::Print => {
                let value = context.collect(context.capture_start, true, false);

                if value.is_some() {
                    println!("{:?}", value.unwrap());
                }

                Ok(Accept::Next)
            }

            Op::Debug(s) => {
                println!("{}", s);
                Ok(Accept::Next)
            }

            Op::Error(s) => Err(Reject::Error(s.to_string())),

            Op::Expect(op) => op
                .run(context)
                .or_else(|_| Err(Reject::Error(format!("Expecting {}", op)))),

            Op::Create(emit) => {
                /*
                println!("Create {} from {:?}",
                    emit, &context.runtime.stack[context.capture_start..]
                );
                */

                let value = match context.collect(context.capture_start, false, false) {
                    Some(capture) => {
                        let value = capture.as_value(context.runtime);
                        let mut ret = Dict::new();

                        ret.insert(
                            "emit".to_string(),
                            Value::String(emit.to_string()).into_ref(),
                        );

                        // List or Dict values are classified as child nodes
                        if value.borrow().get_list().is_some()
                            || value.borrow().get_dict().is_some()
                        {
                            ret.insert("children".to_string(), value);
                        } else {
                            ret.insert("value".to_string(), value);
                        }

                        Value::Dict(Box::new(ret)).into_ref()
                    }
                    None => Value::String(emit.to_string()).into_ref(),
                };

                //println!("Create {} value = {:?}", emit, value);

                Ok(Accept::Return(Some(value)))
            }

            Op::Lexeme(emit) => {
                let value = Value::String(
                    context
                        .runtime
                        .reader
                        .extract(&context.runtime.reader.capture_from(context.reader_start)),
                );

                let mut ret = Dict::new();

                ret.insert(
                    "emit".to_string(),
                    Value::String(emit.to_string()).into_ref(),
                );

                ret.insert("value".to_string(), value.into_ref());

                Ok(Accept::Return(Some(Value::Dict(Box::new(ret)).into_ref())))
            }

            Op::Skip => Ok(Accept::Skip),

            Op::LoadAccept => {
                let value = context.pop();
                Ok(Accept::Return(Some(value.clone())))
            }

            /*
            Op::Accept(value) => {
                Ok(Accept::Return(value.clone()))
            },

            Op::Repeat(value) => {
                Ok(Accept::Repeat(value.clone()))
            },
            */
            Op::Reject => Err(Reject::Return),

            Op::LoadStatic(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.program.statics[*addr].clone(),
                5,
            ))),

            Op::PushTrue => Ok(Accept::Push(Capture::Value(Value::True.into_ref(), 5))),
            Op::PushFalse => Ok(Accept::Push(Capture::Value(Value::False.into_ref(), 5))),
            Op::PushVoid => Ok(Accept::Push(Capture::Value(Value::Void.into_ref(), 5))),

            Op::LoadGlobal(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.stack[*addr].as_value(&context.runtime),
                5,
            ))),

            Op::LoadFast(addr) => Ok(Accept::Push(Capture::Value(
                context.runtime.stack[context.stack_start + *addr].as_value(&context.runtime),
                5,
            ))),

            Op::StoreGlobal(addr) => {
                // todo: bounds checking?
                context.runtime.stack[*addr] = Capture::Value(context.pop(), 10);

                Ok(Accept::Next)
            }

            Op::StoreFast(addr) => {
                // todo: bounds checking?
                context.runtime.stack[context.stack_start + *addr] =
                    Capture::Value(context.pop(), 10);

                Ok(Accept::Next)
            }

            Op::LoadFastCapture(index) => {
                let value = context
                    .get_capture(*index)
                    .unwrap_or(Value::Void.into_ref());

                Ok(Accept::Push(Capture::Value(value, 10)))
            }

            Op::LoadCapture => {
                let index = context.pop();
                let index = index.borrow();

                match *index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        Op::LoadFastCapture(index.to_addr()).run(context)
                    }

                    _ => {
                        unimplemented!("//todo")
                    }
                }
            }

            Op::StoreFastCapture(index) => {
                let value = context.pop();

                context.set_capture(*index, value);
                Ok(Accept::Next)
            }

            Op::StoreCapture => {
                let index = context.pop();
                let index = index.borrow();

                match *index {
                    Value::Addr(_) | Value::Integer(_) | Value::Float(_) => {
                        Op::StoreFastCapture(index.to_addr()).run(context)
                    }

                    _ => {
                        unimplemented!("//todo")
                    }
                }
            }

            Op::Add | Op::Sub | Op::Div | Op::Mul => {
                let b = context.pop();
                let a = context.pop();

                /*
                println!("{:?}", self);
                println!("a = {:?}", a);
                println!("b = {:?}", b);
                */

                let c = match self {
                    Op::Add => (&*a.borrow() + &*b.borrow()).into_ref(),
                    Op::Sub => (&*a.borrow() - &*b.borrow()).into_ref(),
                    Op::Div => (&*a.borrow() / &*b.borrow()).into_ref(),
                    Op::Mul => (&*a.borrow() * &*b.borrow()).into_ref(),
                    _ => unimplemented!("Unimplemented operator"),
                };

                Ok(Accept::Push(Capture::Value(c, 10)))
            }
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        usages: &mut Vec<Vec<Op>>,
        leftrec: &mut bool,
        nullable: &mut bool,
    ) {
        match self {
            Op::Scanable(_) => {
                *nullable = false;
            }

            Op::Runable(runable) => {
                runable.finalize(statics, usages, leftrec, nullable);
            }

            Op::Usage(_) => self.replace_usage(usages),

            Op::CallStatic(addr) => {
                if let Value::Parselet(parselet) = &*statics[*addr].borrow() {
                    if let Ok(mut parselet) = parselet.try_borrow_mut() {
                        let mut call_leftrec = parselet.leftrec;
                        let mut call_nullable = parselet.nullable;

                        parselet.body.finalize(
                            statics,
                            usages,
                            &mut call_leftrec,
                            &mut call_nullable,
                        );

                        parselet.leftrec = call_leftrec;
                        parselet.nullable = call_nullable;

                        *nullable = parselet.nullable;
                    } else {
                        *leftrec = true;
                    }
                }
            }

            _ => {}
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            //Op::Parsable(p) => write!(f, "{}", p),
            _ => write!(f, "Op #todo"),
        }
    }
}

// --- Rust --------------------------------------------------------------------

/** This is allows to run any Rust code in position as Parsable. */
/*
pub struct Rust(pub fn(&mut Context) -> Result<Accept, Reject>);

impl Parsable for Rust {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        self.0(context)
    }
}

impl std::fmt::Debug for Rust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rust-function}}")
    }
}

impl std::fmt::Display for Rust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rust-function}}")
    }
}
*/
// --- Capture -----------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Capture {
    Empty,                       // Empty capture
    Range(Range, u8),            // Captured range from the input & severity
    Value(RefValue, u8),         // Captured value & severity
    Named(Box<Capture>, String), // Named
}

impl Capture {
    pub fn as_value(&self, runtime: &Runtime) -> RefValue {
        match self {
            Capture::Empty => Value::Void.into_ref(),

            Capture::Range(range, _) => Value::String(runtime.reader.extract(range)).into_ref(),

            Capture::Value(value, _) => value.clone(),

            Capture::Named(capture, _) => capture.as_value(runtime),
        }
    }
}

// --- Accept ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Accept {
    Next,
    Skip,
    Push(Capture),
    Repeat(Option<RefValue>),
    Return(Option<RefValue>),
}

// --- Reject ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Reject {
    Next,
    Return,
    Main,
    Error(String),
}

// --- Context -----------------------------------------------------------------

pub struct Context<'runtime, 'program, 'reader> {
    pub runtime: &'runtime mut Runtime<'program, 'reader>, // fixme: Temporary pub?

    pub(super) stack_start: usize,
    pub(super) capture_start: usize,
    pub(super) reader_start: usize,
}

impl<'runtime, 'program, 'reader> Context<'runtime, 'program, 'reader> {
    pub fn new(runtime: &'runtime mut Runtime<'program, 'reader>, preserve: usize) -> Self {
        let stack_start = runtime.stack.len();

        runtime
            .stack
            .resize(stack_start + preserve + 1, Capture::Empty);

        Self {
            stack_start,
            capture_start: stack_start + preserve,
            reader_start: runtime.reader.tell(),
            runtime: runtime,
        }
    }

    // Push value onto the stack
    pub fn push(&mut self, value: RefValue) {
        self.runtime.stack.push(Capture::Value(value, 10))
    }

    /// Pop value off the stack.
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.pop().unwrap();
        capture.as_value(self.runtime)
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&self, pos: usize) -> Option<RefValue> {
        if self.capture_start + pos >= self.runtime.stack.len() {
            return None;
        }

        if pos == 0 {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, _) = &self.runtime.stack[self.capture_start] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(
                Value::String(
                    self.runtime
                        .reader
                        .extract(&(self.reader_start..self.runtime.reader.tell())),
                )
                .into_ref(),
            )
        } else {
            Some(self.runtime.stack[self.capture_start + pos].as_value(&self.runtime))
        }
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&self, name: &str) -> Option<RefValue> {
        // fixme: Should be examined in reversed order
        for capture in self.runtime.stack[self.capture_start..].iter() {
            if let Capture::Named(capture, alias) = &capture {
                if alias == name {
                    return Some(capture.as_value(&self.runtime));
                }
            }
        }

        None
    }

    /** Set a capture to a RefValue by index. */
    pub fn set_capture(&mut self, pos: usize, value: RefValue) {
        let pos = self.capture_start + pos;

        if pos >= self.runtime.stack.len() {
            return;
        }

        self.runtime.stack[pos] = Capture::Value(value, 5)
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        // fixme: Should be examined in reversed order
        for capture in self.runtime.stack[self.capture_start..].iter_mut() {
            if let Capture::Named(capture, alias) = capture {
                if alias == name {
                    *capture = Box::new(Capture::Value(value, 5));
                    break;
                }
            }
        }
    }

    /** Get slice of all captures from current context */
    pub fn get_captures(&self) -> &[Capture] {
        &self.runtime.stack[self.capture_start..]
    }

    /** Drain all captures from current context */
    pub fn drain_captures(&mut self) -> Vec<Capture> {
        self.runtime.stack.drain(self.capture_start..).collect()
    }

    /** Helper function to collect captures from a capture_start and turn
    them either into a dict or list object capture or take them as is.

    This function is internally used for automatic AST construction and value
    inheriting.
    */
    pub(super) fn collect(
        &mut self,
        capture_start: usize,
        copy: bool,
        single: bool,
    ) -> Option<Capture> {
        // Eiter copy or drain captures from stack
        let mut captures: Vec<Capture> = if copy {
            Vec::from_iter(
                self.runtime.stack[capture_start..]
                    .iter()
                    .filter(|item| !(matches!(item, Capture::Empty)))
                    .cloned(),
            )
        } else {
            self.runtime
                .stack
                .drain(capture_start..)
                .filter(|item| !(matches!(item, Capture::Empty)))
                .collect()
        };

        //println!("captures = {:?}", captures);

        if captures.len() == 0 {
            None
        } else if single && captures.len() == 1 && !matches!(captures[0], Capture::Named(_, _)) {
            Some(captures.pop().unwrap())
        } else {
            let mut list = List::new();
            let mut dict = Dict::new();
            let mut max = 0;

            // Collect any significant captures and values
            for capture in captures.into_iter() {
                match capture {
                    Capture::Range(range, severity) if severity >= max => {
                        if severity > max {
                            max = severity;
                            list.clear();
                        }

                        list.push(Value::String(self.runtime.reader.extract(&range)).into_ref());
                    }

                    Capture::Value(value, severity) if severity >= max => {
                        if severity > max {
                            max = severity;
                            list.clear();
                        }

                        list.push(value);
                    }

                    Capture::Named(capture, alias) => {
                        // Named capture becomes dict key
                        dict.insert(alias, capture.as_value(self.runtime));
                    }

                    _ => continue,
                };
            }

            //println!("list = {:?}", list);
            //println!("dict = {:?}", dict);

            if dict.len() == 0 {
                if list.len() > 1 {
                    return Some(Capture::Value(Value::List(Box::new(list)).into_ref(), 5));
                } else if list.len() == 1 {
                    return Some(Capture::Value(list[0].clone(), 5));
                }

                None
            } else {
                for (i, item) in list.into_iter().enumerate() {
                    dict.insert(i.to_string(), item);
                }

                if dict.len() == 1 {
                    return Some(Capture::Value(dict.values().next().unwrap().clone(), 5));
                }

                Some(Capture::Value(Value::Dict(Box::new(dict)).into_ref(), 5))
            }
        }
    }
}

impl<'runtime, 'program, 'reader> Drop for Context<'runtime, 'program, 'reader> {
    fn drop(&mut self) {
        self.runtime.stack.truncate(self.stack_start);
    }
}

// --- Runtime -----------------------------------------------------------------

pub struct Runtime<'program, 'reader> {
    program: &'program Program,
    pub(crate) reader: &'reader mut Reader, // temporary pub

    pub(super) memo: HashMap<(usize, usize), (usize, Result<Accept, Reject>)>,
    pub(super) stack: Vec<Capture>,
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            memo: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}
