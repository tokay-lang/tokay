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

#[derive(Clone, Debug)]
pub enum CallBy {
    Index(usize),
    Name(String)
}


// --- Op ----------------------------------------------------------------------

//#[derive(Debug)]
pub enum Op {
    // Semantics
    Accept(Option<RefValue>),
    Reject,

    // Items
    Token(Box<dyn Token>),
    Call(Box<CallBy>),

    // Operators
    Sequence(Box<Sequence>),
    Block(Box<Block>),
    //Kleene(Box<Op>),
    //Positive(Box<Op>),
    //Optional(Box<Op>),
    //And(Box<Op>),
    //Not(Box<Op>),

    Rust(fn(&mut Scope) -> Result<Accept, Reject>),
}

impl Op {
    fn run(&self, scope: &mut Scope) -> Result<Accept, Reject> {
        match self {
            Op::Accept(value) => {
                Ok(Accept::Return(value.clone()))
            },
            Op::Reject => {
                Err(Reject::Return)
            },

            Op::Token(token) => {
                let reader_start = scope.runtime.reader.tell();

                if let Some(capture) = token.read(&mut scope.runtime.reader) {
                    Ok(Accept::Push(capture))
                } else {
                    scope.runtime.reader.reset(reader_start);
                    Err(Reject::Next)
                }
            },

            Op::Call(callee) => {
                let parselet = match callee.as_ref() {
                    CallBy::Name(name) => {
                        if let Some(p) = scope.runtime.program.parselets.get_by_key(name) {
                            p
                        } else {
                            return Err(Reject::Error(format!("The parselet {} does not exist!", name)));
                        }
                    },
                    CallBy::Index(idx) => scope.runtime.program.parselets.get(*idx).unwrap().1
                };

                parselet.run(scope.runtime)
            },

            Op::Sequence(sequence) => sequence.run(scope),
            Op::Block(block) => block.run(scope),
            Op::Rust(callback) => callback(scope)
        }
    }

    /*
    fn is_nullable(&mut self) -> bool {
        match self {
            Op::Token(token) => token.is_nullable(),
            Op::Sequence(sequence) => sequence.nullable,
            Op::Block(block) => block.nullablle,
            Op::Rust(_) => true
            _ => false
        }
    }

    fn is_leftrec(&mut self, origin: usize) -> bool {
        if let Op::Call(callee) = self {
            let mut parselet = match callee.as_ref() {
                CallBy::Name(name) => {
                    if let Some(p) = scope.runtime.program.parselets.get_by_key(name) {
                        p
                    } else {
                        return Err(Reject::Error(format!("The parselet {} does not exist!", name)));
                    }
                },
                CallBy::Index(idx) => scope.runtime.program.parselets.get(*idx).unwrap().1
            };

            if parselet.id() == origin {
                return true;
            } else {
                parselet.body.is_leftrec()
            }
        }

        false
    }
    */
}

// --- Sequence ----------------------------------------------------------------

pub struct Sequence {
    leftrec: bool,
    nullable: bool,
    items: Vec<(Op, Option<String>)>
}

impl Sequence {
    pub fn new(items: Vec<(Op, Option<String>)>) -> Self {
        Self{
            leftrec: false,
            nullable: false,
            items
        }
    }

    /*
    fn is_leftrec(&mut self, origin: usize) -> bool {
        for item in &self.items.iter_mut() {
            if item.is_leftrec(target) {
                self.leftrec |= true;
            }

            if !item.is_nullable() {
                return self.leftrec;
            }
        }

        self.nullable = true;
        self.leftrec
    }
    */

    fn run(&self, scope: &mut Scope) -> Result<Accept, Reject> {
        let capture_start = scope.runtime.capture.len();
        let reader_start = scope.runtime.reader.tell();
        
        for (item, alias) in &self.items {
            match item.run(scope) {
                Err(reject) => {
                    scope.runtime.capture.truncate(capture_start);
                    scope.runtime.reader.reset(reader_start);
                    return Err(reject);
                }

                Ok(accept) => {
                    match accept {
                        Accept::Next => scope.runtime.capture.push((Capture::Empty, alias.clone())),
                        Accept::Push(capture) => scope.runtime.capture.push((capture, alias.clone())),
                        Accept::Return(value) => {
                            scope.runtime.capture.truncate(capture_start);
                            return Ok(Accept::Return(value));
                        }
                    }
                }
            }
        }

        // todo: generate a value or dingens
        Ok(Accept::Push(Capture::Range(scope.runtime.reader.capture_from(reader_start), 1)))
    }
}

// --- Block -------------------------------------------------------------------

pub struct Block {
    leftrec: bool,
    nullable: bool,
    items: Vec<Op>
}

impl Block {
    pub fn new(items: Vec<Op>) -> Self {
        Self{
            leftrec: false,
            nullable: false,
            items
        }
    }

    /*
    fn is_leftrec(&mut self, origin: usize) -> bool {
        for item in &self.items.iter_mut() {
            if item.is_leftrec(origin) {
                self.leftrec |= true;
            }

            if item.is_nullable() {
                self.nullable |= true;
            }
        }

        self.leftrec
    }
    */

    pub fn run(&self, scope: &mut Scope) -> Result<Accept, Reject> {
        for item in &self.items {
            match item.run(scope) {
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

pub struct Parselet {
    id: usize,
    leftrec: bool,
    body: Block
}

impl Parselet {
    fn new(id: usize, body: Block) -> Self {
        Self{
            id,
            leftrec: false,
            body
        }
    }

    /*
    fn is_leftrec(&mut self) {
        if !self.leftrec {
            self.leftrec = self.body.is_leftrec(self.id());
        }

        self.leftrec
    }
    */

    fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        self.body.run(&mut Scope::new(runtime))
    }
}


// --- Scope -------------------------------------------------------------------

pub struct Scope<'runtime, 'program, 'reader> {
    runtime: &'runtime mut Runtime<'program, 'reader>,

    stack_start: usize,
    capture_start: usize,
    reader_start: usize
}

impl<'runtime, 'program, 'reader> Scope<'runtime, 'program, 'reader> {
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

impl<'runtime, 'program, 'reader> Drop for Scope<'runtime, 'program, 'reader> {
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

pub struct Program {
    // Input & memoization
    //memo: HashMap<(usize, usize), (usize, State)>,
    pub parselets: Map<String, Parselet>
}

impl Program {
    pub fn new() -> Self {
        Self{
            parselets: Map::new()
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Accept, Reject> {
        self.parselets[0].run(runtime)
    }

    //pub fn finalize(&mut self)

    pub fn new_parselet(&mut self, name: Option<String>, body: Block) -> usize {
        let id = self.parselets.len();
        let parselet = Parselet::new(id, body);

        if let Some(name) = name {
            self.parselets.push_key_value(name, parselet);
        }
        else {
            self.parselets.push_value(parselet);
        }
        
        id
    }
}


#[macro_export]
macro_rules! token {
    (|$var:ident| $code:block) => {
        Op::Rust(|$var| $code)
    };
    ($ident:ident) => {
        Op::Call(Box::new(CallBy::Name(stringify!($ident).to_string())))
    };
    ($literal:literal) => {
        Op::Token(Match::new_touch($literal))
    };
    ($expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! modifier {
    /*
    ($program:expr, pos( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($program, $($token)+);
            token.into_positive($program)
        }
    };
    ($program:expr, opt( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($program, $($token)+);
            token.into_optional($program)
        }
    };
    ($program:expr, kle( $( $token:tt )+ ) ) => {
        {
            let token = modifier!($program, $($token)+);
            let token = token.into_positive($program);
            token.into_optional($program)
        }
    };
    */
    ( $( ( $( $token:tt )+ ) )* ) => {
        sequence!([ $( ( $($token)+ ) ),* ] )
    };
    ($( $token:tt )+) => {
        token!($($token)+)
    };
}

#[macro_export]
macro_rules! sequence {
    ([ $( ( $( $token:tt )+ ) ),* ] ) => {
        {
            Op::Sequence(
                Box::new(
                    Sequence::new(vec![
                        $(
                            ( modifier!($( $token )+), None )
                        ),*
                    ])
                )
            )
        }
    };
}

#[macro_export]
macro_rules! tokay {
    ($program:expr, $( $name:ident { $( => $( ( $( $token:tt )+ ) )* )+ } )+ )
        => {
        {
            $(
                $program.new_parselet(
                    Some(stringify!($name).to_string()),
                    Block::new(vec![
                        $( sequence!([ $( ( $($token)+ ) ),* ] ) ),+
                    ])
                );
            )+
        }
    }
}
