use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::collections::HashMap;

use crate::map::Map;
use crate::value::{Complex, Value, RefValue};
use crate::token::{Token, Capture, Match};
use crate::reader::{Reader, Range};


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
    leftrec: bool,
    nullable: bool,
    items: Vec<(Item, Option<String>)>
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
    items: Vec<Item>,
    leftrec: bool
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
    leftrec: bool,
    nullable: bool,
    body: Item
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
    pub fn new() -> Self {
        Self{
            parselets: Vec::new(),
            statics: Vec::new()
        }
    }

    pub fn finalize(&mut self) {
        // Turn parselets vec into a RefCell vec
        let parselets: Vec<RefCell<Parselet>> =
            self.parselets.drain(..).map(|item| RefCell::new(item)).collect();

        fn walk(parselets: &Vec<RefCell<Parselet>>, 
                leftrec: &mut bool,
                nullable: &mut bool,
                item: &mut Item)
        {
            match item {
                Item::Name(name) => panic!("OH no, there is Name({}) still!", name),
                Item::Token(_) => {
                    *nullable = false;
                },
                Item::Call(idx) => {
                    if let Ok(mut parselet) = parselets[*idx].try_borrow_mut() {
                        let mut my_leftrec = parselet.leftrec;
                        let mut my_nullable = parselet.nullable;

                        walk(
                            parselets,
                            &mut my_leftrec,
                            &mut my_nullable,
                            &mut parselet.body
                        );

                        parselet.leftrec = my_leftrec;
                        parselet.nullable = my_nullable;

                        *nullable = parselet.nullable;
                    }
                    else {
                        *leftrec = true;
                    }
                },

                Item::Sequence(sequence) => {
                    for (item, _) in sequence.items.iter_mut() {
                        walk(
                            parselets,
                            &mut sequence.leftrec,
                            &mut sequence.nullable,
                            item
                        );

                        if !sequence.nullable {
                            break
                        }
                    }

                    *leftrec = sequence.leftrec;
                    *nullable = sequence.nullable;
                },

                Item::Block(block) => {
                    *nullable = false;

                    for item in block.items.iter_mut() {
                        let mut my_nullable = true;
                        let mut my_leftrec = true;

                        walk(
                            parselets,
                            &mut my_leftrec,
                            &mut my_nullable,
                            item
                        );

                        if !my_nullable {
                            *nullable = false;
                        }

                        if my_leftrec {
                            block.leftrec = true;
                        }
                    }

                    *leftrec = block.leftrec;
                }

                _ => {}
            }
        }

        let mut changes = true;
        let mut loops = 0;

        while changes {
            changes = false;

            for i in 0..parselets.len() {
                let mut parselet = parselets[i].borrow_mut();
                let mut leftrec = parselet.leftrec;
                let mut nullable = parselet.nullable;
    
                walk(
                    &parselets,
                    &mut leftrec,
                    &mut nullable,
                    &mut parselet.body
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

        println!("finalize stopped after {} loops", loops);

        self.parselets = parselets.into_iter().map(|item| item.into_inner()).collect();
        self.dump();
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


pub struct Compiler {
    scopes: Vec<Scope>,                     // Current compilation scopes
    statics: HashMap<String, usize>,        // Mapping statics from origins
    parselets: Vec<Parselet>
}

struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>,
    parselets: usize
}

impl Compiler {
    pub fn new() -> Self {
        Self{
            scopes: vec![
                Scope{
                    variables: Some(HashMap::new()),
                    constants: HashMap::new(),
                    parselets: 0
                }
            ],
            statics: HashMap::new(),
            parselets: Vec::new()
        }
    }

    pub fn into_program(mut self) -> Program {
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        self.resolve();

        let mut program = Program::new();
        program.parselets = self.parselets.drain(..).collect();

        //program.dump();

        program.finalize();
        program
    }

    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(0,
            Scope{
                variables: if variables { Some(HashMap::new()) } else { None },
                constants: HashMap::new(),
                parselets: self.parselets.len()
            }
        );
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Can't pop main scope");
        }

        self.resolve();
        self.scopes.remove(0);
    }

    pub fn get_var(&self, name: &str) -> Option<usize> {
        for scope in &self.scopes {
            if let Some(variables) = &scope.variables {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr)
                }
                else {
                    break;
                }
            }
        }

        None
    }

    pub fn set_var(&mut self, name: &str, addr: usize) {
        for scope in self.scopes.iter_mut() {
            if let Some(variables) = &mut scope.variables {
                variables.insert(name.to_string(), addr);
                break;
            }
        }
    }

    pub fn get_const(&self, name: &str) -> Option<usize> {
        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(*addr)
            }
        }

        None
    }

    pub fn set_const(&mut self, name: &str, addr: usize) {
        self.scopes.first_mut().unwrap().constants.insert(
            name.to_string(), addr
        );
    }

    /** Pushes a static value into the program.

    The function checks for origin, to avoid making several
    static values with equal content. In case an origin is provided and found,
    returns the address of existing value instead.

    Returns the unique address of the static inside the program.
    */
    /*
    pub fn push_static(&mut self, origin: Option<&str>, value: RefValue) -> usize
    {
        if let Some(origin) = origin {
            if let Some(addr) = self.statics.get(origin) {
                return *addr;
            }
        }

        let addr = self.program.statics.len();
        self.program.statics.push(value);

        if let Some(origin) = origin {
            self.statics.insert(origin.to_string(), addr);
        }
        
        addr
    }
    */

    pub fn push_parselet(&mut self, parselet: Parselet) -> usize
    {
        self.parselets.push(parselet);
        self.parselets.len() - 1
    }

    pub fn resolve(&mut self) {
        fn walk(scopes: &Vec<Scope>, item: &mut Item) {
            match item {
                Item::Name(name) => {
                    for scope in scopes {
                        if let Some(addr) = scope.constants.get(name) {
                            println!("resolved {:?} as {:?}", name, *addr);
                            *item = Item::Call(*addr);
                            return;
                        }
                    }

                    panic!("Cannot resolve {:?}", name);
                },
    
                Item::Sequence(ref mut sequence) => {
                    for (item, _) in sequence.items.iter_mut() {
                        walk(scopes, item);
                    }
                },
    
                Item::Block(ref mut block) => {
                    for item in block.items.iter_mut() {
                        walk(scopes, item);
                    }
                },
    
                _ => {}
            };
        }

        for i in self.scopes.first().unwrap().parselets..self.parselets.len() {
            walk(&self.scopes, &mut self.parselets[i].body);
        }
    }
}


#[macro_export]
macro_rules! tokay_item {
    // Rust
    ($compiler:expr, |$var:ident| $code:block) => {
        Item::Rust(|$var| $code)
    };

    // Assign
    ( $compiler:expr, ( $name:ident = $item:tt ) ) => {
        {
            let item = tokay_item!($compiler, $item);
            let id = $compiler.push_parselet(
                Parselet::new(item)
            );

            $compiler.set_const(
                stringify!($name),
                id
            );

            println!("assign {} = {}", stringify!($name), stringify!($item));
            Item::Nop
        }
    };

    // Sequence
    ( $compiler:expr, [ $( $item:tt ),* ] ) => {
        {
            //println!("sequence");
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            Item::Sequence(
                Box::new(
                    Sequence::new(
                        items.into_iter().filter(
                            // Remove any Nops
                            |item| !matches!(item, Item::Nop)).map(
                                // Turn into (item, None) tuples
                                |item| (item, None)).collect()
                    )
                )
            )
        }
    };

    // Parselet
    ( $compiler:expr, { $( $item:tt ),* } ) => {
        {
            println!("open scope {{");
            $compiler.push_scope(true);
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let block = Block::new(
                items.into_iter().filter(
                    // Remove any Nops
                    |item| !matches!(item, Item::Nop)).collect()
            );

            let mut item = Item::Block(
                Box::new(
                    block
                )
            );

            $compiler.pop_scope();
            println!("}} close scope");
            item
        }
    };

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            if let Some(addr) = $compiler.get_const(stringify!($ident)) {
                Item::Call(addr)
            } else {
                Item::Name(stringify!($ident).to_string())
            }
        }
    };

    // Match
    ( $compiler:expr, $literal:literal ) => {
        /*println!("match = {}", $literal);*/
        Item::Token(
            Match::new($literal)
            //Match::new_touch($literal)
        )
    };

    // Fallback
    ( $compiler:expr, $expr:tt ) => {
        {
            //println!("expr = {}", stringify!($expr));
            $expr
        }
    };
}


#[macro_export]
macro_rules! tokay {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();

            {
                let main = tokay_item!(compiler, $( $items ),*);
                compiler.push_parselet(Parselet::new(main));
            }

            compiler.into_program()
        }
    }
}
