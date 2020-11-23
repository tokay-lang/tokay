use std::collections::HashMap;
use std::cell::RefCell;
use std::iter::FromIterator;

use crate::ccl::Ccl;
use crate::value::{Complex, Value, RefValue};
use crate::reader::{Reader, Range};
use crate::compiler::Compiler;
use crate::ccl;


#[derive(Debug, Clone)]
pub enum Accept {
    Next,
    Skip,
    Push(Capture),
    Repeat(Option<RefValue>),
    Return(Option<RefValue>)
}


#[derive(Debug, Clone)]
pub enum Reject {
    Next,
    Return,
    Main,
    Error(String)
}


/** Parser trait */
pub trait Parser: std::fmt::Debug + std::fmt::Display {
    /** Perform a parse on a given context.

    A parse may either Accept or Reject, with a given severity.
    */
    fn run(&self, context: &mut Context) -> Result<Accept, Reject>;

    /** Finalize according grammar view;

    This function is called from top of each parselet to detect
    both left-recursive and nullable (=no input consuming) structures. */
    fn finalize(
        &mut self,
        _parselets: &Vec<RefCell<Parselet>>,
        _leftrec: &mut bool,
        _nullable: &mut bool)
    {
        // default is: just do nothing ;)
    }

    /** Resolve is called by the compiler to resolve unresolved symbols
    inside or below a program structure */
    fn resolve(&mut self, _compiler: &Compiler, _strict: bool)
    {
        // default is: just do nothing ;)
    }

    /** Convert parser object into boxed dyn Parser Op */
    fn into_op(self) -> Op
        where Self: std::marker::Sized + 'static
    {
        Op::Parser(Box::new(self))
    }
}


// --- Op ----------------------------------------------------------------------

/**
Atomic operations.

Specifies atomic level operations like running a parser or running VM code.
*/
#[derive(Debug)]
pub enum Op {

    // Parser
    Parser(Box<dyn Parser>),
    Empty,

    // Debuging and error reporting
    Print,
    Debug(&'static str),
    Error(&'static str),
    Expect(Box<Op>),

    // AST construction
    Create(&'static str),
    Lexeme(&'static str),

    // Interrupts
    Skip,
    Accept(Option<RefValue>),
    Repeat(Option<RefValue>),
    Reject,

    // Call
    Call(usize),
    TryCall,
    Name(String),

    // Constants
    LoadStatic(usize),
    PushAddr(usize),
    PushInt(i64),
    PushFloat(f64),
    PushTrue,
    PushFalse,
    PushVoid,

    // Variables
    LoadGlobal(usize),
    LoadFast(usize),
    StoreGlobal(usize),
    StoreFast(usize),
    LoadCaptureFast(usize),
    LoadCapture

    /*
    And(Op),
    Not(Op)
    */
}

impl Op {
    pub fn into_kleene(self) -> Self {
        Repeat::kleene(self)
    }

    pub fn into_positive(self) -> Self {
        Repeat::positive(self)
    }

    pub fn into_optional(self) -> Self {
        Repeat::optional(self)
    }
}

impl Parser for Op {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            Op::Parser(p) => p.run(context),

            Op::Empty => {
                Ok(Accept::Push(Capture::Empty))
            },

            Op::Print => {
                let value = context.collect(
                    context.capture_start, true, false
                );
                println!("{:?}", value);
                Ok(Accept::Next)
            },

            Op::Debug(s) => {
                println!("{}", s);
                Ok(Accept::Next)
            },

            Op::Error(s) => {
                Err(Reject::Error(s.to_string()))
            },

            Op::Expect(op) => {
                op.run(context).or_else(|_| {
                    Err(
                        Reject::Error(
                            format!("Expecting {}", op)
                        )
                    )
                })
            },

            Op::Create(emit) => {
                /*
                println!("Create {} from {:?}",
                    emit, &context.runtime.stack[context.capture_start..]
                );
                */

                let value = match context.collect(
                    context.capture_start, false, false)
                {
                    Some(value) => {
                        let mut ret = Complex::new();
                        ret.push_key_value(
                            emit.to_string(),
                            value
                        );

                        Value::Complex(Box::new(ret)).into_ref()
                    }
                    None => {
                        Value::String(emit.to_string()).into_ref()
                    }
                };

                //println!("Create {} value = {:?}", emit, value);

                Ok(Accept::Return(Some(value)))
            },

            Op::Lexeme(emit) => {
                let value = Value::String(
                    context.runtime.reader.extract(
                        &context.runtime.reader.capture_from(
                            context.reader_start
                        )
                    )
                );

                let mut ret = Complex::new();
                ret.push_key_value(
                    emit.to_string(),
                    value.into_ref()
                );

                Ok(
                    Accept::Return(
                        Some(Value::Complex(Box::new(ret)).into_ref())
                    )
                )
            },

            Op::Skip => {
                Ok(Accept::Skip)
            },

            Op::Accept(value) => {
                Ok(Accept::Return(value.clone()))
            },

            Op::Repeat(value) => {
                Ok(Accept::Repeat(value.clone()))
            },

            Op::Reject => {
                Err(Reject::Return)
            },

            Op::Call(parselet) => {
                context.runtime.program.parselets[*parselet].run(
                    context.runtime
                )
            },

            Op::TryCall => {
                let value = context.pop();

                match *value.borrow() {

                    Value::Parselet(p) => {
                        return context.runtime.program.parselets[p].run(
                            context.runtime
                        )
                    },

                    _ => {}
                }

                Ok(Accept::Push(Capture::Value(value)))
            }

            Op::Name(_) => panic!("{:?} cannot be executed", self),

            Op::LoadStatic(addr) => {
                Ok(Accept::Push(Capture::Value(
                    context.runtime.program.statics[*addr].clone()
                )))
            },
            Op::PushAddr(a) => {
                Ok(Accept::Push(Capture::Value(Value::Addr(*a).into_ref())))
            },
            Op::PushInt(i) => {
                Ok(Accept::Push(Capture::Value(Value::Integer(*i).into_ref())))
            },
            Op::PushFloat(f) => {
                Ok(Accept::Push(Capture::Value(Value::Float(*f).into_ref())))
            },
            Op::PushTrue => {
                Ok(Accept::Push(Capture::Value(Value::True.into_ref())))
            },
            Op::PushFalse => {
                Ok(Accept::Push(Capture::Value(Value::False.into_ref())))
            },
            Op::PushVoid => {
                Ok(Accept::Push(Capture::Value(Value::Void.into_ref())))
            },

            Op::LoadGlobal(addr) => {
                Ok(Accept::Push(
                    Capture::Value(
                        context.runtime.stack[*addr].0
                            .as_value(&context.runtime)
                    )
                ))
            },

            Op::LoadFast(addr) => {
                Ok(Accept::Push(
                    Capture::Value(
                        context.runtime.stack[
                            context.stack_start + addr
                        ].0.as_value(&context.runtime)
                    )
                ))
            },

            Op::StoreGlobal(addr) => {
                // todo
                Ok(Accept::Next)
            },

            Op::StoreFast(addr) => {
                context.runtime.stack[context.stack_start + addr].0 =
                    Capture::Value(context.pop());
                Ok(Accept::Next)
            },

            Op::LoadCaptureFast(index) => {
                let value = context.get_capture(*index).unwrap_or(
                    Value::Void.into_ref()
                );
                context.push(value);

                Ok(Accept::Next)
            },

            Op::LoadCapture => {
                if let Value::Addr(index) = *context.pop().borrow() {
                    Op::LoadCaptureFast(index).run(context);
                    Ok(Accept::Next)
                }
                else {
                    Err(Reject::Error("Internal".to_string()))
                }
            }
        }
    }

    fn finalize(
        &mut self,
        parselets: &Vec<RefCell<Parselet>>,
        leftrec: &mut bool,
        nullable: &mut bool)
    {
        match self {
            Op::Parser(parser) => parser.finalize(parselets, leftrec, nullable),

            Op::Name(name) => panic!("OH no, there is Name({}) still!", name),

            Op::Call(idx) => {
                if let Ok(mut parselet) = parselets[*idx].try_borrow_mut() {
                    let mut my_leftrec = parselet.leftrec;
                    let mut my_nullable = parselet.nullable;

                    parselet.body.finalize(
                        parselets,
                        &mut my_leftrec,
                        &mut my_nullable,
                    );

                    parselet.leftrec = my_leftrec;
                    parselet.nullable = my_nullable;

                    *nullable = parselet.nullable;
                }
                else {
                    *leftrec = true;
                }
            },

            _ => {}
        }
    }

    fn resolve(&mut self, compiler: &Compiler, strict: bool)
    {
        match self {
            Op::Parser(parser) => parser.resolve(compiler, strict),

            Op::Name(name) => {
                if let Some(value) = compiler.get_constant(name) {
                    match &*value.borrow() {
                        Value::Parselet(p) => {
                            //println!("resolved {:?} as {:?}", name, *p);
                            *self = Op::Call(*p);
                            return;
                        },
                        Value::String(s) => {
                            *self = Match::new(&s.clone()).into_op();
                            return;
                        },
                        _ => {
                            unimplemented!("Cannot resolve {:?}", value);
                        }
                    }
                }
                else if strict {
                    panic!("Cannot resolve {:?}", name);
                }
            },

            _ => {}
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Parser(p) => write!(f, "{}", p),
            _ => write!(f, "Op #todo")
        }
    }
}

// --- Rust -------------------------------------------------------------------
//fixme: This should not be implement as Parser.

pub struct Rust(pub fn(&mut Context) -> Result<Accept, Reject>);

impl Parser for Rust {
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

// --- Char -------------------------------------------------------------------

#[derive(Debug)]
pub struct Char {
    accept: Ccl,
    repeats: bool
}

impl Char {
    fn _new(accept: Ccl, repeats: bool) -> Op {
        Self{
            accept,
            repeats
        }.into_op()
    }

    pub fn new(accept: Ccl) -> Op {
        Self::_new(accept, false)
    }

    pub fn any() -> Op {
        let mut any = Ccl::new();
        any.negate();

        Self::new(any)
    }

    pub fn char(ch: char) -> Op {
        Self::new(ccl![ch..=ch])
    }

    pub fn span(ccl: Ccl) -> Op {
        Self::_new(ccl, true)
    }

    pub fn until(ch: char) -> Op {
        let mut other = ccl![ch..=ch];
        other.negate();

        Self::span(other)
    }
}

impl Parser for Char {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let start = context.runtime.reader.tell();

        while let Some(ch) = context.runtime.reader.peek() {
            if !self.accept.test(&(ch..=ch)) {
                break;
            }

            context.runtime.reader.next();

            if !self.repeats {
                break;
            }
        }

        if start < context.runtime.reader.tell() {
            Ok(
                Accept::Push(
                    Capture::Range(
                        context.runtime.reader.capture_from(start)
                    )
                )
            )
        }
        else {
            context.runtime.reader.reset(start);
            Err(Reject::Next)
        }
    }

    fn finalize(
        &mut self,
        _parselets: &Vec<RefCell<Parselet>>,
        _leftrec: &mut bool,
        nullable: &mut bool)
    {
        *nullable = false;
    }
}

impl std::fmt::Display for Char {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Char #todo")
    }
}

// --- Match ------------------------------------------------------------------

#[derive(Debug)]
pub struct Match(String);

impl Match {
    pub fn new(string: &str) -> Op {
        Self(string.to_string()).into_op()
    }
}

impl Parser for Match {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let start = context.runtime.reader.tell();

        for ch in self.0.chars() {
            if let Some(c) = context.runtime.reader.next() {
                if c != ch {
                    // fixme: Optimize me!
                    context.runtime.reader.reset(start);
                    return Err(Reject::Next);
                }
            }
            else {
                // fixme: Optimize me!
                context.runtime.reader.reset(start);
                return Err(Reject::Next);
            }
        }

        Ok(
            Accept::Push(
                Capture::Range(
                    context.runtime.reader.capture_last(self.0.len())
                )
            )
        )
    }

    fn finalize(
        &mut self,
        _parselets: &Vec<RefCell<Parselet>>,
        _leftrec: &mut bool,
        nullable: &mut bool)
    {
        *nullable = false;
    }
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

// --- Repeat ------------------------------------------------------------------

/** Repeating parser.

This is a simple programmtic repetition. For several reasons, repetitions can
also be expressed on a specialized token-level or by the grammar using left-
or right-recursive structures.
*/

#[derive(Debug)]
pub struct Repeat {
    parser: Op,
    min: usize,
    max: usize,
    mute: bool
}

impl Repeat {
    pub fn new(parser: Op, min: usize, max: usize, mute: bool) -> Op
    {
        assert!(max == 0 || max >= min);

        Self{
            parser,
            min,
            max,
            mute
        }.into_op()
    }

    pub fn kleene(parser: Op) -> Op {
        Self::new(parser, 0, 0, false)
    }

    pub fn positive(parser: Op) -> Op {
        Self::new(parser, 1, 0, false)
    }

    pub fn optional(parser: Op) -> Op {
        Self::new(parser, 0, 1, false)
    }

    pub fn muted_kleene(parser: Op) -> Op {
        Self::new(parser, 0, 0, true)
    }

    pub fn muted_positive(parser: Op) -> Op {
        Self::new(parser, 1, 0, true)
    }

    pub fn muted_optional(parser: Op) -> Op {
        Self::new(parser, 0, 1, true)
    }
}

impl Parser for Repeat {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Remember capturing positions
        let capture_start = context.runtime.stack.len();
        let reader_start = context.runtime.reader.tell();

        let mut count: usize = 0;

        loop {
            match self.parser.run(context) {
                Err(Reject::Next) => break,

                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject)
                },

                Ok(Accept::Next) => {},

                Ok(Accept::Push(capture)) => {
                    if !self.mute {
                        context.runtime.stack.push((capture, None))
                    }
                },

                Ok(accept) => {
                    return Ok(accept)
                }
            }

            count += 1;

            if self.max > 0 && count == self.max {
                break
            }
        }

        if count < self.min {
            context.runtime.stack.truncate(capture_start);
            context.runtime.reader.reset(reader_start);
            Err(Reject::Next)
        }
        else {
            // Push collected captures, if any
            if let Some(value) = context.collect(capture_start, false, false)
            {
                Ok(Accept::Push(Capture::Value(value)))
            }
            // Otherwiese, push a capture of consumed range
            else if reader_start < context.runtime.reader.tell() {
                Ok(
                    Accept::Push(
                        Capture::Range(
                            context.runtime.reader.capture_from(reader_start)
                        )
                    )
                )
            }
            // Else, just accept next
            else {
                Ok(Accept::Next)
            }
        }
    }

    fn finalize(
        &mut self,
        parselets: &Vec<RefCell<Parselet>>,
        leftrec: &mut bool,
        nullable: &mut bool)
    {
        self.parser.finalize(parselets, leftrec, nullable);

        if self.min == 0 {
            *nullable = true;
        }
    }

    fn resolve(&mut self, compiler: &Compiler, strict: bool)
    {
        self.parser.resolve(compiler, strict);
    }
}

impl std::fmt::Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Repeat #todo")
    }
}

// --- Sequence ----------------------------------------------------------------

/** Sequence parser.

This parser collects a sequence of sub-parsers. According to the sub-parsers
semantics, or when an entire sequence was completely recognized, the sequence
is getting accepted. Incomplete sequences are rejected.
*/

#[derive(Debug)]
pub struct Sequence {
    leftrec: bool,
    nullable: bool,
    items: Vec<(Op, Option<String>)>
}

impl Sequence {
    pub fn new(items: Vec<(Op, Option<String>)>) -> Op
    {
        Self{
            leftrec: false,
            nullable: true,
            items
        }.into_op()
    }
}

impl Parser for Sequence {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Empty sequence?
        if self.items.len() == 0 {
            return Ok(Accept::Next);
        }

        // Remember capturing positions
        let capture_start = context.runtime.stack.len();
        let reader_start = context.runtime.reader.tell();

        // Iterate over sequence
        for (item, alias) in &self.items {
            match item.run(context) {
                Err(reject) => {
                    context.runtime.stack.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => {
                    context.runtime.stack.push((Capture::Empty, alias.clone()))
                },

                Ok(Accept::Push(capture)) => {
                    context.runtime.stack.push((capture, alias.clone()))
                },

                Ok(Accept::Return(value)) if value.is_none() => {
                    return Ok(
                        Accept::Return(
                            context.collect(capture_start, false, true)
                        )
                    )
                },

                Ok(Accept::Repeat(value)) if value.is_none() => {
                    return Ok(
                        Accept::Repeat(
                            context.collect(capture_start, false, true)
                        )
                    )
                },

                other => {
                    return other
                }
            }
        }

        /*
            When no explicit Return is performed, first try to collect any
            significant captures.
        */
        //println!("Sequence {:?}", &context.runtime.capture[capture_start..]);

        if let Some(value) = context.collect(capture_start, false, true) {
            Ok(Accept::Push(Capture::Value(value)))
        }
        /*
            When this even fails, push a range of the current sequence in
            case than any input was consumed.

            fixme:
            Maybe this could be pushed with severity 0, and later on collected?
        */
        else if reader_start < context.runtime.reader.tell() {
            Ok(
                Accept::Push(
                    Capture::Range(
                        context.runtime.reader.capture_from(reader_start)
                    )
                )
            )
        }
        /*
            Otherwise, just return Next.
        */
        else {
            Ok(Accept::Next)
        }
    }

    fn finalize(
        &mut self,
        parselets: &Vec<RefCell<Parselet>>,
        leftrec: &mut bool,
        nullable: &mut bool)
    {
        for (item, _) in self.items.iter_mut() {
            item.finalize(
                parselets,
                &mut self.leftrec,
                &mut self.nullable
            );

            if !self.nullable {
                break
            }
        }

        *leftrec = self.leftrec;
        *nullable = self.nullable;
    }

    fn resolve(&mut self, compiler: &Compiler, strict: bool)
    {
        for (item, _) in self.items.iter_mut() {
            item.resolve(compiler, strict);
        }
    }

}

impl std::fmt::Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sequence #todo")
    }
}

// --- Block -------------------------------------------------------------------

/** Block parser.

A block parser defines either an alternation of sequences or a grouped sequence
of VM instructions. The compiler has to guarantee for correct usage of the block
parser.

Block parsers support static program constructs being left-recursive, and extend
the generated parse tree automatically until no more input can be consumed.
*/

#[derive(Debug)]
pub struct Block {
    leftrec: bool,
    items: Vec<(Op, bool)>
}

impl Block {
    pub fn new(items: Vec<Op>) -> Op {
        Self{
            items: items.into_iter().map(|item| (item, false)).collect(),
            leftrec: false
        }.into_op()
    }
}

impl Parser for Block {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject>
    {
        // Internal Block run function
        fn run(block: &Block, context: &mut Context, leftrec: bool)
                -> Result<Accept, Reject>
        {
            let mut res = Ok(Accept::Next);
            let reader_start = context.runtime.reader.tell();

            for (item, item_leftrec) in &block.items {
                // Skip over parsers that don't match leftrec configuration
                if *item_leftrec != leftrec {
                    continue;
                }

                res = item.run(context);

                // Generally break on anything which is not Next.
                if !matches!(&res, Ok(Accept::Next) | Err(Reject::Next)) {

                    // Push only accepts when input was consumed, otherwise the
                    // push value is just discarded, except for the last item
                    // being executed.
                    if let Ok(Accept::Push(_)) = res {
                        // No consuming, no breaking!
                        if reader_start == context.runtime.reader.tell() {
                            continue
                        }
                    }

                    break
                }
            }

            res
        }

        // Create a unique block id from the Block's address
        let id = self as *const Block as usize;

        // Check for an existing memo-entry, and return it in case of a match
        if let Some((reader_end, result)) =
            context.runtime.memo.get(&(context.reader_start, id))
        {
            context.runtime.reader.reset(*reader_end);
            return result.clone();
        }

        if self.leftrec {
            //println!("Leftrec {:?}", self);

            // Left-recursive blocks are called in a loop until no more input
            // is consumed.

            let mut reader_end = context.reader_start;
            let mut result = Err(Reject::Next);

            // Insert a fake memo entry to avoid endless recursion

            /* info: removing this fake entry does not affect program run!

            This is because of the leftrec parameter to internal run(),
            which only accepts non-left-recursive calls on the first run.
            As an additional fuse, this fake memo entry should anyway be kept.
            */
            context.runtime.memo.insert(
                (context.reader_start, id),
                (reader_end, result.clone())
            );

            let mut loops = 0;

            loop {
                let res = run(self, context, loops > 0);

                match res {
                    // Hard reject
                    Err(Reject::Main) | Err(Reject::Error(_)) => {
                        return res;
                    },

                    // Soft reject
                    Err(_) => {
                        if loops == 0 {
                            return res
                        }
                        else {
                            break
                        }
                    },

                    _ => {}
                }

                // Stop also when no more input was consumed
                if context.runtime.reader.tell() <= reader_end {
                    break;
                }

                result = res;

                reader_end = context.runtime.reader.tell();
                context.runtime.memo.insert(
                    (context.reader_start, id),
                    (reader_end, result.clone())
                );

                context.runtime.reader.reset(context.reader_start);
                context.runtime.stack.truncate(context.capture_start);
                loops += 1;
            }

            context.runtime.reader.reset(reader_end);
            result
        }
        else {
            // Non-left-recursive block can be called directly.
            run(self, context, false)
        }
    }

    fn finalize(
        &mut self,
        parselets: &Vec<RefCell<Parselet>>,
        leftrec: &mut bool,
        nullable: &mut bool)
    {
        *nullable = false;

        for (item, item_leftrec) in self.items.iter_mut() {
            *item_leftrec = false;
            let mut my_nullable = true;

            item.finalize(
                parselets,
                item_leftrec,
                &mut my_nullable
            );

            if my_nullable {
                *nullable = true;
            }

            if *item_leftrec {
                self.leftrec = true;
            }
        }

        *leftrec = self.leftrec;
    }

    fn resolve(&mut self, compiler: &Compiler, strict: bool)
    {
        for (item, _) in self.items.iter_mut() {
            item.resolve(compiler, strict);
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block #todo")
    }
}

// --- Parselet ----------------------------------------------------------------

#[derive(Debug)]
pub struct Parselet {
    leftrec: bool,
    nullable: bool,
    mute: bool,
    locals: usize,
    body: Op
}

impl Parselet {
    pub fn new(body: Op, locals: usize) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            mute: false,
            locals,
            body
        }
    }

    /// Creates a new silent parselet, which does always return Capture::Empty
    pub fn new_muted(body: Op, locals: usize) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            mute: true,
            locals,
            body
        }
    }

    fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        let mut context = Context::new(runtime, self.locals);

        /*
            fixme:
            This loop exists for the planned "repeat"-statement,
            which repeats a given parselet programmatically and
            without using the Repeat parser. It currently is not
            really used, and may or may not come in future.
        */
        loop {
            let res = self.body.run(&mut context);

            match res {
                Ok(Accept::Skip) => {
                    return Ok(Accept::Push(Capture::Empty))
                },

                Ok(Accept::Return(value)) => {
                    //println!("{:?} returns {:?}", self.emit, value);

                    if let Some(value) = value {
                        if !self.mute {
                            return Ok(Accept::Push(Capture::Value(value)))
                        } else {
                            return Ok(Accept::Push(Capture::Empty))
                        }
                    }
                    else {
                        return Ok(Accept::Push(Capture::Empty))
                    }
                },

                Ok(Accept::Repeat(value)) => {
                    // this is unfinished...
                    if value.is_none() {
                        continue
                    }
                },

                Ok(Accept::Next) => {
                    return Ok(
                        if self.mute {
                            Accept::Push(Capture::Empty)
                        }
                        else {
                            Accept::Push(
                                Capture::Value(context.get_0())
                            )
                        }
                    )
                },

                res => {
                    return res
                }
            }
        }
    }

    pub fn resolve(&mut self, compiler: &Compiler, strict: bool)
    {
        self.body.resolve(compiler, strict);
    }

    pub fn finalize(parselets: &Vec<RefCell<Parselet>>) -> usize {
        let mut changes = true;
        let mut loops = 0;

        while changes {
            changes = false;

            for i in 0..parselets.len() {
                let mut parselet = parselets[i].borrow_mut();
                let mut leftrec = parselet.leftrec;
                let mut nullable = parselet.nullable;

                parselet.body.finalize(
                    parselets,
                    &mut leftrec,
                    &mut nullable
                );

                if !parselet.leftrec && leftrec {
                    parselet.leftrec = true;
                    changes = true;
                }

                if parselet.nullable && !nullable {
                    parselet.nullable = nullable;
                    changes = true;
                }
            }

            loops += 1;
        }

        println!("finalization finished after {} loops", loops);
        loops
    }
}



// --- Capture -----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Capture {
    Empty,                      // Empty capture
    Range(Range),               // Captured range from the input
    Value(RefValue)             // Captured value
}

impl Capture {
    fn as_value(&self, runtime: &Runtime) -> RefValue {
        match self {
            Capture::Empty => {
                Value::Void.into_ref()
            },

            Capture::Range(range) => {
                Value::String(
                    runtime.reader.extract(range)
                ).into_ref()
            },

            Capture::Value(value) => {
                value.clone()
            }
        }
    }
}

// --- Context -----------------------------------------------------------------

pub struct Context<'runtime, 'program, 'reader> {
    pub runtime: &'runtime mut Runtime<'program, 'reader>,  // fixme: Temporary pub?

    stack_start: usize,
    capture_start: usize,
    reader_start: usize
}

impl<'runtime, 'program, 'reader> Context<'runtime, 'program, 'reader> {

    pub fn new(
        runtime: &'runtime mut Runtime<'program, 'reader>,
        preserve: usize
    ) -> Self
    {
        let stack_start = runtime.stack.len();

        runtime.stack.resize(
            runtime.stack.len() + preserve + 1,
            (Capture::Empty, None)
        );

        //ret.runtime.stack.push((Capture::Empty, None));
        Self{
            stack_start,
            capture_start: runtime.stack.len() - 1,
            reader_start: runtime.reader.tell(),
            runtime: runtime
        }
    }

    pub fn push(&mut self, value: RefValue) {
        self.runtime.stack.push((Capture::Value(value), None))
    }

    pub fn pop(&mut self) -> RefValue {
        let value = self.runtime.stack.pop().unwrap().0;
        if let Capture::Value(value) = value {
            value
        } else {
            Value::Void.into_ref() // fixme: is this ok?
        }
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        // Capture $0 is specially handled.
        if pos == 0 {
            return Some(self.get_0());
        }

        // Anything else by position.
        let pos = self.capture_start + pos;

        if pos >= self.runtime.stack.len() {
            return None
        }

        Some(self.runtime.stack[pos].0.as_value(&self.runtime))
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        // fixme: Should be examined in reversed order
        for (i, capture) in
            self.runtime.stack[self.capture_start..].iter().enumerate()
        {
            if let Some(alias) = &capture.1 {
                if alias == name {
                    return self.get_capture(i);
                }
            }
        }

        None
    }

    /** Returns the current $0 value */
    pub fn get_0(&self) -> RefValue {
        if let Capture::Value(value) =
            &self.runtime.stack[self.capture_start].0
        {
            return value.clone()
        }

        Value::String(
            self.runtime.reader.extract(
                &(self.reader_start..self.runtime.reader.tell())
            )
        ).into_ref()
    }

    /** Save current $0 value */
    pub fn set_0(&mut self, value: RefValue) {
        self.runtime.stack[self.capture_start].0 = Capture::Value(value)
    }

    /** Helper function to collect captures from a capture_start and turn
    them either into a Complex-type value or take it as is. */
    fn collect(
        &mut self,
        capture_start: usize,
        copy: bool,
        single: bool) -> Option<RefValue>
    {
        let mut captures: Vec<(Capture, Option<String>)> = if copy {
            Vec::from_iter(
                self.runtime.stack[capture_start..].iter().cloned()
            )
        }
        else {
            self.runtime.stack.drain(capture_start..).collect()
        };

        if captures.len() == 0 {
            None
        }
        else if single && captures.len() == 1 && captures[0].1.is_none()
        {
            match captures.pop().unwrap().0 {
                Capture::Empty => None,
                Capture::Range(range) => {
                    Some(
                        Value::String(
                            self.runtime.reader.extract(&range)
                        ).into_ref()
                    )
                },
                Capture::Value(value) => {
                    Some(value)
                }
            }
        }
        else {
            let (values, ranges): (_ ,Vec<_>) = captures.into_iter().partition(
                |item| matches!(item.0, Capture::Value(_)));

            let mut complex = Complex::new();
            let captures = if !values.is_empty() { values } else { ranges };

            // Collect any significant captures and values
            for (capture, alias) in captures.into_iter() {
                let value = match capture {
                    Capture::Empty => continue,
                    Capture::Range(range) => {
                        Value::String(
                            self.runtime.reader.extract(&range)
                        ).into_ref()
                    },
                    Capture::Value(value) => {
                        value
                    }
                };

                // Named capture becomes complex key
                if let Some(name) = alias {
                    complex.push_key_value(name, value);
                }
                else {
                    complex.push_value(value);
                }
            }

            /* When there is only one value without a key in the map,
                return this single value only! */
            if complex.len() == 1 {
                if let Some((None, value)) = complex.get(0) {
                    return Some(value.clone())
                }
            }

            if complex.len() > 0 {
                // Return the complex when it contains something
                Some(Value::Complex(Box::new(complex)).into_ref())
            }
            else {
                None
            }
        }
    }
}

impl<'runtime, 'program, 'reader> Drop for Context<'runtime, 'program, 'reader> {
    fn drop(&mut self) {
        self.runtime.stack.truncate(self.capture_start);
    }
}


// --- Runtime -----------------------------------------------------------------

pub struct Runtime<'program, 'reader> {
    program: &'program Program,
    pub reader: &'reader mut Reader,  // temporary pub

    memo: HashMap<(usize, usize), (usize, Result<Accept, Reject>)>,

    stack: Vec<(Capture, Option<String>)>
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            memo: HashMap::new(),
            stack: Vec::new()
        }
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
    }
}


// --- Program -----------------------------------------------------------------

#[derive(Debug)]
pub struct Program {
    parselets: Vec<Parselet>,
    statics: Vec<RefValue>
}

impl Program {
    pub fn new(
        parselets: Vec<Parselet>,
        statics: Vec<RefValue>
    ) -> Self {
        Self{
            parselets,
            statics
        }
    }

    pub fn dump(&self) {
        for i in 0..self.parselets.len() {
            println!("P{}{} = {{", i, if self.parselets[i].nullable { "  # nullable" } else { "" });
            //dump(&self.parselets[i].body, 1);
            println!("{:#?}", self.parselets[i].body);
            println!("}}");
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        self.parselets.last().unwrap().run(runtime)
    }
}
