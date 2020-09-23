use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use crate::map::Map;
use crate::value::{Value, RefValue};
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
    // Semantics
    Accept(Option<RefValue>),
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
            Item::Accept(value) => {
                Ok(Accept::Return(value.clone()))
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
                context.runtime.program.parselets[*parselet].run(context.runtime)
            },

            Item::Sequence(sequence) => sequence.run(context),
            Item::Block(block) => block.run(context),
            Item::Name(_) => panic!("{:?} cannot be executed", self)
            //Item::Rust(callback) => callback(context)
        }
    }
}

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
            nullable: false,
            items
        }
    }

    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let capture_start = context.runtime.capture.len();
        let reader_start = context.runtime.reader.tell();
        
        for (item, alias) in &self.items {
            match item.run(context) {
                Err(reject) => {
                    context.runtime.capture.truncate(capture_start);
                    context.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(accept) => {
                    match accept {
                        Accept::Next => context.runtime.capture.push((Capture::Empty, alias.clone())),
                        Accept::Push(capture) => context.runtime.capture.push((capture, alias.clone())),
                        Accept::Return(value) => {
                            context.runtime.capture.truncate(capture_start);
                            return Ok(Accept::Return(value));
                        }
                    }
                }
            }
        }

        // todo: generate a value or dingens
        Ok(Accept::Push(Capture::Range(context.runtime.reader.capture_from(reader_start), 1)))
    }
}

// --- Block -------------------------------------------------------------------

#[derive(Debug)]
pub struct Block {
    leftrec: bool,
    nullable: bool,
    items: Vec<Item>
}

impl Block {
    pub fn new(items: Vec<Item>) -> Self {
        Self{
            leftrec: false,
            nullable: false,
            items
        }
    }

    pub fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        for item in &self.items {
            match item.run(context) {
                Err(reject) => {
                    if let Reject::Next = reject {
                        continue
                    }

                    return Err(reject);
                },
                Ok(accept) => {
                    if let Accept::Next = accept {
                        continue
                    }

                    return Ok(accept);
                }
            }
        }

        Ok(Accept::Next)
    }
}


// --- Parselet ----------------------------------------------------------------

#[derive(Debug)]
pub struct Parselet {
    leftrec: bool,
    body: Item
}

impl Parselet {
    fn new(body: Item) -> Self {
        Self{
            leftrec: false,
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

    stack: Vec<RefValue>,
    capture: Vec<(Capture, Option<String>)>
}

impl<'program, 'reader> Runtime<'program, 'reader> {
    pub fn new(program: &'program Program, reader: &'reader mut Reader) -> Self {
        Self {
            program,
            reader,
            stack: Vec::new(),
            capture: Vec::new()
        }
    }
}


// --- Program -----------------------------------------------------------------

#[derive(Debug)]
pub struct Program {
    // Input & memoization
    //memo: HashMap<(usize, usize), (usize, State)>,
    pub parselets: Vec<Parselet>
}

impl Program {
    pub fn new() -> Self {
        Self{
            parselets: Vec::new()
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        self.parselets[0].run(runtime)
    }

    pub fn new_parselet(&mut self, body: Item) -> usize {
        self.parselets.push(Parselet::new(body));
        self.parselets.len() - 1
    }
}



pub struct Scope<'scope> {
    program: Rc<RefCell<Program>>,
    parent: Option<&'scope Scope<'scope>>,
    locals: u32,
    symbols: HashMap<String, usize>
}

impl<'scope> Scope<'scope> {
    pub fn new(program: Rc<RefCell<Program>>) -> Self {
        Self {
            program,
            parent: None,
            locals: 0,
            symbols: HashMap::new()
        }
    }

    pub fn new_below(scope: &'scope Scope) -> Self {
        Self {
            program: scope.program.clone(),
            parent: Some(scope),
            locals: 0,
            symbols: HashMap::new()
        }
    }

    pub fn get_name(&self, name: &str) -> Option<usize> {
        println!("get {:?}", name);
        self.symbols.get(name).cloned()
    }

    pub fn set_name(&mut self, name: String, addr: usize) {
        println!("set {:?} to {:?}", name, addr);
        self.symbols.insert(name, addr);
    }
}

#[macro_export]
macro_rules! item {
    ($scope:expr, |$var:ident| $code:block) => {
        Item::Rust(|$var| $code)
    };
    ($scope:expr, $ident:ident) => {
        if let Some(addr) = $scope.get_name(stringify!($ident)) {
            Item::Call(addr)
        } else {
            Item::Name(stringify!($ident).to_string())
        }
    };
    ($scope:expr, $literal:literal) => {
        Item::Token(Match::new_touch($literal))
    };
    ($scope:expr, $expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! modifier {
    /*
    ($scope:expr, pos( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($scope, $($token)+);
            token.into_positive($scope)
        }
    };
    ($scope:expr, opt( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($scope, $($token)+);
            token.into_optional($scope)
        }
    };
    ($scope:expr, kle( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($scope, $($token)+);
            let token = token.into_positive($scope);
            token.into_optional($scope)
        }
    };
    */
    ($scope:expr, $( ( $( $token:tt )+ ) )*) => {
        sequence!($scope, [ $( ( $($token)+ ) ),* ] )
    };
    ($scope:expr, $( $token:tt )+) => {
        item!($scope, $($token)+)
    };
}

#[macro_export]
macro_rules! sequence {
    ($scope:expr, [ $( ( $( $token:tt )+ ) ),* ]) => {
        {
            Item::Sequence(
                Box::new(
                    Sequence::new(vec![
                        $(
                            ( modifier!($scope, $( $token )+), None )
                        ),*
                    ])
                )
            )
        }
    };
}

#[macro_export]
macro_rules! tokay {
    ( $( $name:ident { $( => $( ( $( $token:tt )+ ) )* )+ } )+ ) => {
        {
            let program = Rc::new(RefCell::new(Program::new()));

            {
                let mut scope = Scope::new(program.clone());

                $(
                    scope.set_name(stringify!($name).to_string(), program.borrow().parselets.len());

                    let block = Item::Block(
                        Box::new(
                            Block::new(vec![
                                $( sequence!(scope, [ $( ( $($token)+ ) ),* ] ) ),+
                            ])
                        )
                    );

                    program.borrow_mut().new_parselet(block);
                )+
            }

            Rc::try_unwrap(program).unwrap().into_inner()
        }
    }
}
