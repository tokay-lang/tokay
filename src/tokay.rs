use std::collections::HashMap;
use std::cell::RefCell;

use crate::value::{Complex, Value, RefValue};
use crate::token::{self, Match, Capture};
use crate::reader::{Reader};
use crate::compiler::Compiler;


#[derive(Debug, Clone)]
pub enum Accept {
    Next,
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

pub trait Parser: std::fmt::Debug {
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

    /** Convert parser object into boxed dyn Parser */
    fn into_box(self) -> Box<dyn Parser>
        where Self: std::marker::Sized + 'static
    {
        Box::new(self)
    }
}

// --- Rust -------------------------------------------------------------------

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


// --- Atomic -----------------------------------------------------------------

/**
Atomic parsers and operations.

Specifies atomic level operations like matching a token or running VM code.
*/
#[derive(Debug)]
pub enum Atomic {
    Nop,

    // Tokens
    Empty,
    Token(Box<dyn token::Token>),

    // Semantics
    Accept(Option<RefValue>),
    Repeat(Option<RefValue>),
    Reject,

    // Eval
    Push(RefValue),  // just a test

    // Parselets
    Call(usize),
    Name(String)

    //And(Box<Atomic>),
    //Not(Box<Atomic>)
}

impl Parser for Atomic {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            Atomic::Accept(value) => {
                Ok(Accept::Return(value.clone()))
            },

            Atomic::Repeat(value) => {
                Ok(Accept::Repeat(value.clone()))
            },

            Atomic::Reject => {
                Err(Reject::Return)
            },

            Atomic::Push(value) => {
                Ok(Accept::Push(Capture::Value(value.clone(), 1)))
            },

            Atomic::Empty => {
                Ok(Accept::Push(Capture::Empty))
            },

            Atomic::Token(token) => {
                let reader_start = context.runtime.reader.tell();

                if let Some(capture) = token.read(&mut context.runtime.reader) {
                    Ok(Accept::Push(capture))
                } else {
                    context.runtime.reader.reset(reader_start);
                    Err(Reject::Next)
                }
            },

            Atomic::Call(parselet) => {
                context.runtime.program.parselets[*parselet].run(
                    context.runtime
                )
            },

            Atomic::Nop | Atomic::Name(_) => panic!("{:?} cannot be executed", self),
        }
    }

    fn finalize(
        &mut self,
        parselets: &Vec<RefCell<Parselet>>,
        leftrec: &mut bool,
        nullable: &mut bool)
    {
        match self {
            Atomic::Name(name) => panic!("OH no, there is Name({}) still!", name),

            Atomic::Token(_) => {
                *nullable = false;
            },

            Atomic::Call(idx) => {
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
        if let Atomic::Name(name) = self {
            if let Some(value) = compiler.get_constant(name) {
                match &*value.borrow() {
                    Value::Parselet(p) => {
                        println!("resolved {:?} as {:?}", name, *p);
                        *self = Atomic::Call(*p);
                        return;
                    },
                    Value::String(s) => {
                        *self = Atomic::Token(Match::new_touch(&s.clone()));
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
        }
    }
}


// --- Repeat ------------------------------------------------------------------

/** Repeating parser.

This is a simple programmtic repetition. For several reasons, repetitions can
also be expressed on a specialized token-level or by the grammar using left-
or right-recursive structures. It's to on the compiler for chosing the best
option. (see kle!-, pos!-, opt!-macros from compiler)
*/

#[derive(Debug)]
pub struct Repeat {
    parser: Box<dyn Parser>,
    min: usize,
    max: usize,
    capture: bool
}

impl Repeat {
    pub fn new(parser: Box<dyn Parser>, min: usize, max: usize, capture: bool)
        -> Self
    {
        assert!(max == 0 || max >= min);

        Self{
            parser,
            min,
            max,
            capture
        }
    }
}

impl Parser for Repeat {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Remember capturing positions
        let capture_start = context.runtime.capture.len();
        let reader_start = context.runtime.reader.tell();

        let mut count: usize = 0;

        loop {
            match self.parser.run(context) {
                Err(Reject::Next) => break,

                Err(reject) => {
                    context.runtime.capture.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject)
                },

                Ok(Accept::Next) => {},

                Ok(Accept::Push(capture)) => {
                    if self.capture {
                        context.runtime.capture.push((capture, None))
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
            context.runtime.capture.truncate(capture_start);
            context.runtime.reader.reset(reader_start);
            Err(Reject::Next)
        }
        else {
            if let Some(value) = context.collect_captures(capture_start, false)
            {
                Ok(Accept::Push(Capture::Value(value, 1)))
            }
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
    items: Vec<(Box<dyn Parser>, Option<String>)>
}

impl Sequence {
    pub fn new(items: Vec<(Box<dyn Parser>, Option<String>)>) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            items
        }
    }
}

impl Parser for Sequence {

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Empty sequence?
        if self.items.len() == 0 {
            return Ok(Accept::Next);
        }

        // Remember capturing positions
        let capture_start = context.runtime.capture.len();
        let reader_start = context.runtime.reader.tell();

        // Iterate over sequence
        for (item, alias) in &self.items {
            match item.run(context) {
                Err(reject) => {
                    context.runtime.capture.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(Accept::Next) => {
                    context.runtime.capture.push((Capture::Empty, alias.clone()))
                },

                Ok(Accept::Push(capture)) => {
                    context.runtime.capture.push((capture, alias.clone()))
                },

                Ok(Accept::Return(value)) if value.is_none() => {
                    return Ok(
                        Accept::Return(
                            context.collect_captures(capture_start, true)
                        )
                    )
                },

                Ok(Accept::Repeat(value)) if value.is_none() => {
                    return Ok(
                        Accept::Repeat(
                            context.collect_captures(capture_start, true)
                        )
                    )
                },

                other => return other
            }
        }

        /*
            When no explicit Return is performed, first try to collect any
            significant captures.
        */
        if let Some(value) = context.collect_captures(capture_start, true) {
            Ok(Accept::Push(Capture::Value(value, 1)))
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
                        context.runtime.reader.capture_from(reader_start), 1
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

// --- Block -------------------------------------------------------------------

/** Block parser.

A block parser defines either an alternation of sequences or a grouped sequence
of VM instructions. The compiler has to guarantee for correct usage of the
block parser.

Block parsers support static program constructs being left-recursive, and extend
the generated parse tree automatically until no more input can be consumed.
*/

#[derive(Debug)]
pub struct Block {
    leftrec: bool,
    items: Vec<(Box<dyn Parser>, bool)>
}

impl Block {
    pub fn new(items: Vec<Box<dyn Parser>>) -> Self {
        Self{
            items: items.into_iter().map(|item| (item, false)).collect(),
            leftrec: false
        }
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
                    Err(_) => {
                        if loops == 0 {
                            return res;
                        } else {
                            break;
                        }
                    },

                    Ok(Accept::Return(_)) | Ok(Accept::Repeat(_)) => {
                        return res;
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
                context.runtime.capture.truncate(context.capture_start);
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


// --- Parselet ----------------------------------------------------------------

#[derive(Debug)]
pub struct Parselet {
    leftrec: bool,
    nullable: bool,
    body: Box<dyn Parser>
}

impl Parselet {
    pub fn new(body: Box<dyn Parser>) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            body
        }
    }

    fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        let mut context = Context::new(runtime);

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
                Ok(Accept::Return(value)) => {
                    if let Some(value) = value {
                        return Ok(Accept::Push(Capture::Value(value, 1)))
                    }
                    else {
                        return Ok(Accept::Push(Capture::Empty))
                    }
                },

                Ok(Accept::Repeat(value)) => {
                    if value.is_none() {
                        continue
                    }
                },

                Ok(Accept::Next) => {
                    return Ok(
                        Accept::Push(Capture::Value(context.get_value(), 0))
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


// --- Context -----------------------------------------------------------------

pub struct Context<'runtime, 'program, 'reader> {
    runtime: &'runtime mut Runtime<'program, 'reader>,

    stack_start: usize,
    capture_start: usize,
    reader_start: usize
}

impl<'runtime, 'program, 'reader> Context<'runtime, 'program, 'reader> {
    pub fn new(runtime: &'runtime mut Runtime<'program, 'reader>) -> Self {
        let ret = Self{
            stack_start: runtime.stack.len(),
            capture_start: runtime.capture.len(),
            reader_start: runtime.reader.tell(),
            runtime: runtime
        };

        ret.runtime.capture.push((Capture::Empty, None));
        ret
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        // Capture $0 is specially handled.
        if pos == 0 {
            return Some(self.get_value());
        }

        // Anything else by position.
        let pos = self.capture_start + pos;

        if pos >= self.runtime.capture.len() {
            return None
        }

        let replace = match &self.runtime.capture[pos].0 {
            Capture::Empty => {
                Capture::Value(
                    Value::Void.into_ref(), 0
                )
            },

            Capture::Range(range, severity) => {
                Capture::Value(
                    Value::String(self.runtime.reader.extract(range)).into_ref(), *severity
                )
            },

            Capture::Value(value, _) => {
                return Some(value.clone())
            }
        };

        self.runtime.capture[pos].0 = replace;
        self.get_capture(pos - self.capture_start)
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        // fixme: Maybe this should be examined in reversed order?
        for (i, capture) in self.runtime.capture[self.capture_start..].iter().enumerate() {
            if let Some(alias) = &capture.1 {
                if alias == name {
                    return self.get_capture(i);
                }
            }
        }

        None
    }

    /** Returns the current $0 value */
    pub fn get_value(&self) -> RefValue {
        if let Capture::Value(value, _) = &self.runtime.capture[self.capture_start].0 {
            return value.clone()
        }

        Value::String(
            self.runtime.reader.extract(
                &(self.reader_start..self.runtime.reader.tell())
            )
        ).into_ref()
    }

    /** Save current $0 value */
    pub fn set_value(&mut self, value: RefValue) {
        self.runtime.capture[self.capture_start].0 = Capture::Value(value, 2)
    }

    /** Helper function to collect context captures from a capture_start and turn
    them either into a Complex-type value or take it as is. */
    fn collect_captures(
        &mut self,
        capture_start: usize,
        allow_single: bool) -> Option<RefValue>
    {
        fn get_significant_value(
            context: &mut Context, capture: &Capture)
                -> Option<RefValue>
        {
            match capture {
                // Turn significant capture into string
                Capture::Range(range, severity)
                    if *severity > 0 => {
                    Some(
                        Value::String(
                            context.runtime.reader.extract(&range)
                        ).into_ref()
                    )
                },

                // Take value as is
                Capture::Value(value, severity)
                    if *severity > 0 => {
                    Some(value.clone())
                },

                _ => {
                    None
                }
            }
        }

        let captures: Vec<(Capture, Option<String>)>
            = self.runtime.capture.drain(capture_start..).collect();

        if captures.len() == 0 {
            None
        }
        else if allow_single && captures.len() == 1 && captures[0].1.is_none()
        {
            if let Some(value) = get_significant_value(self, &captures[0].0)
            {
                Some(value)
            }
            else {
                None
            }
        }
        else {
            let mut complex = Complex::new();

            // Collect any significant captures and values
            for (capture, alias) in captures
            {
                if let Some(value) = get_significant_value(self, &capture)
                {
                    // Named capture becomes complex key
                    if let Some(name) = alias {
                        complex.push_key_value(Value::String(name), value);
                    }
                    else {
                        complex.push_value(value);
                    }
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
        self.runtime.capture.truncate(self.capture_start);
        self.runtime.stack.truncate(self.stack_start);
    }
}


// --- Runtime -----------------------------------------------------------------

pub struct Runtime<'program, 'reader> {
    program: &'program Program,
    reader: &'reader mut Reader,

    memo: HashMap<(usize, usize), (usize, Result<Accept, Reject>)>,

    stack: Vec<RefValue>,
    capture: Vec<(Capture, Option<String>)>
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            memo: HashMap::new(),
            stack: Vec::new(),
            capture: Vec::new()
        }
    }

    pub fn dump(&self) {
        println!("memo has {} entries", self.memo.len());
        println!("stack has {} entries", self.stack.len());
        println!("capture has {} entries", self.capture.len());
    }
}


// --- Program -----------------------------------------------------------------

#[derive(Debug)]
pub struct Program {
    parselets: Vec<Parselet>,
    statics: Vec<RefValue>
}

impl Program {
    pub fn new(parselets: Vec<Parselet>) -> Self {
        Self{
            parselets,
            statics: Vec::new()
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
