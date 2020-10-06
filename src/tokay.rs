use std::cell::{RefCell};
use std::collections::HashMap;

use crate::value::{Complex, Value, RefValue};
use crate::token::{Token, Capture};
use crate::reader::{Reader};


#[derive(Debug, Clone)]
pub enum Accept {
    Next,
    Push(Capture),
    Return(Option<RefValue>)
}

#[derive(Debug, Clone)]
pub enum Reject {
    Next,
    Return,
    Main,
    Error(String)
}


// --- Item --------------------------------------------------------------------

#[derive(Debug)]
pub enum Item {
    Nop,

    // Semantics
    Accept,
    Reject,

    // Atomics
    Empty,
    Token(Box<dyn Token>),
    Call(usize),
    //Goto(usize),
    Name(String),

    // Operators
    Sequence(Box<Sequence>),
    Block(Box<Block>),
    //Kleene(Box<Item>),
    //Positive(Box<Item>),
    //Optional(Box<Item>),
    //And(Box<Item>),
    //Not(Box<Item>),

    //Rust(fn(&mut Context) -> Result<Accept, Reject>),
}

impl Item {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            Item::Accept => {
                Ok(Accept::Return(None))
            },

            Item::Reject => {
                Err(Reject::Return)
            },

            Item::Empty => {
                Ok(Accept::Push(Capture::Empty))
            },

            Item::Token(token) => {
                let reader_start = context.runtime.reader.tell();

                if let Some(capture) = token.read(&mut context.runtime.reader) {
                    Ok(Accept::Push(capture))
                } else {
                    context.runtime.reader.reset(reader_start);
                    Err(Reject::Next)
                }
            },

            Item::Call(parselet) => {
                context.runtime.program.parselets[*parselet].run(
                    context.runtime
                )
            },

            Item::Sequence(sequence) => sequence.run(context),
            Item::Block(block) => block.run(context),
            Item::Nop | Item::Name(_) => panic!("{:?} cannot be executed", self),
            //Item::Rust(callback) => callback(context)
        }
    }
}

/*
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Item::Empty => write!(f, "Empty"),
            Item::Token(t) => write!(f, "{:?}", t),
            Item::Call(idx) => write!(f, "P{}", idx),
            Item::Name(s) => write!(f, "{:?}", s),
            Item::Rust(_) => write!(f, "{{rust-function}}"),
            _ => self.0.fmt(f)?,
        }
        Ok(())
    }
}
*/

// --- Sequence ----------------------------------------------------------------

#[derive(Debug)]
pub struct Sequence {
    pub leftrec: bool,
    pub nullable: bool,
    pub items: Vec<(Item, Option<String>)>
}

impl Sequence {
    pub fn new(items: Vec<(Item, Option<String>)>) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            items
        }
    }

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

                Ok(accept) => {
                    match accept {
                        Accept::Next => {
                            context.runtime.capture.push((Capture::Empty, alias.clone()))
                        },
                        Accept::Push(capture) => {
                            context.runtime.capture.push((capture, alias.clone()))
                        },
                        Accept::Return(value) => {
                            context.runtime.capture.truncate(capture_start);
                            return Ok(Accept::Return(value));
                        }
                    }
                }
            }
        }

        if self.items.len() == 1 && self.items[0].1.is_none() {
            Ok(Accept::Push(context.runtime.capture.pop().unwrap().0))
        }
        else {
            let mut complex = Complex::new();

            // Collect any significant captures and values
            for (value, alias) in context.runtime.capture.drain(capture_start..) {
                let value = match value {
                    // Turn significant capture into string
                    Capture::Range(range, severity) if severity > 0 => {
                        Value::String(
                            context.runtime.reader.extract(&range)
                        ).into_ref()
                    },
                    
                    // Take value as is
                    Capture::Value(value, severity) if severity > 0 => {
                        value.clone()
                    },

                    _ => {
                        continue
                    }
                };

                // Named capture becomes complex key
                if let Some(name) = alias {
                    complex.push_key_value(Value::String(name), value);
                }
                else {
                    complex.push_value(value);
                }
            }

            /* When there is only one value without a key in the map,
                return this single value only! */
            if complex.len() == 1 {
                if let Some((None, value)) = complex.get(0) {
                    return Ok(Accept::Push(Capture::Value(value.clone(), 1)))
                }
            }

            if complex.len() > 0 {
                // Return the complex when it contains something
                Ok(Accept::Push(
                    Capture::Value(
                        Value::Complex(Box::new(complex)).into_ref(), 1)
                    )
                )
            }
            else {
                Ok(Accept::Push(Capture::Empty))
            }
        }
    }
}

// --- Block -------------------------------------------------------------------

#[derive(Debug)]
pub struct Block {
    pub leftrec: bool,
    pub items: Vec<Item>
}

impl Block {
    pub fn new(items: Vec<Item>) -> Self {
        Self{
            items,
            leftrec: false
        }
    }

    pub fn run(&self, context: &mut Context) -> Result<Accept, Reject> {

        // Internal run function
        fn run(block: &Block, context: &mut Context, leftrec: bool)
                -> Result<Accept, Reject>
        {
            let mut res = Ok(Accept::Next);

            for item in &block.items {
                // Skip over sequences without matching leftrec configuration
                if let Item::Sequence(seq) = item {
                    if seq.leftrec != leftrec {
                        continue;
                    }
                }

                res = item.run(context);

                // Stop on anything which is not Accept::Next or Reject::Next
                if !matches!(&res, Ok(Accept::Next) | Err(Reject::Next)) {
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
                let tmp_result = run(self, context, loops > 0);

                // Stop either on reject or when no more input was consumed
                if matches!(tmp_result, Err(Reject::Next))
                    || context.runtime.reader.tell() <= reader_end {
                    break;
                }

                result = tmp_result;

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
}


// --- Parselet ----------------------------------------------------------------

#[derive(Debug)]
pub struct Parselet {
    pub leftrec: bool,
    pub nullable: bool,
    pub body: Item
}

impl Parselet {
    pub fn new(body: Item) -> Self {
        Self{
            leftrec: false,
            nullable: true,
            body
        }
    }

    fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        self.body.run(&mut Context::new(runtime))
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
            )).into_ref()
    }

    /** Save current $0 value */
    pub fn set_value(&mut self, value: RefValue) {
        self.runtime.capture[self.capture_start].0 = Capture::Value(value, 2)
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
        fn dump(item: &Item, level: usize) {
            match item {
                Item::Block(block) => {
                    for item in &block.items {
                        print!("{}", " ".repeat(level));
                        dump(item, level + 1);
                        print!("\n");
                    }
                },
                Item::Sequence(sequence) => {
                    for (item, alias) in &sequence.items {
                        dump(item, level + 1);
                        if let Some(alias) = alias {
                            print!(":{} ", alias);
                        }
                        else {
                            print!(" ");
                        }
                    }

                    if sequence.leftrec || sequence.nullable {
                            print!("  # {}{} ",
                            if sequence.leftrec {"left-recursive " } else {""},
                            if sequence.nullable {"nullable"} else {""}
                        );
                    }
                },
                other => {
                    print!("{:?}", other);
                }
            }
        }

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
