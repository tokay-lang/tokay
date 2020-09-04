use std::io::prelude::*;
use std::collections::HashMap;

use crate::ccl::Ccl;
use crate::reader::{Reader, Range};
use crate::value::{Value, RefValue, Complex};

/*
- a Program
    - has Parselets...
        - which have Rules...
            - which is a sequence of Tokens...
*/

// --- Token ---

pub type Sequence = Vec<(Token, Option<String>)>;

#[derive(Clone, Debug)]
pub enum CallBy {
    Index(usize),
    Name(String)
}

#[derive(Clone)]
pub enum Token {
    None,
    Any,
    Touch(String), // todo: merge with Match
    Match(String),
    Char(Ccl),
    Chars(Ccl),  // todo: remove soon
    Call(CallBy),
    Rust(fn(&mut Runtime) -> Return),
    Sequence(Vec<Token>),
    Positive(Box<Token>),
    Optional(Box<Token>),
    Kleene(Box<Token>),
}

impl Token {
    fn run(&self, program: &Program, runtime: &mut Runtime) -> Return {
        let reader_start = runtime.reader.tell();

        match self {
            Token::None => {
                Return::Capture(
                    Capture::Range(runtime.reader.capture(reader_start), 1)
                )
            },

            Token::Any => {
                if runtime.reader.next().is_none() {
                    return Return::Reject;
                }

                Return::Capture(
                    Capture::Range(runtime.reader.capture(reader_start), 1),
                )
            },

            Token::Match(string) | Token::Touch(string) => {
                for ch in string.chars() {
                    if let Some(c) = runtime.reader.next() {
                        if c != ch {
                            return Return::Reject;
                        }
                    }
                    else {
                        return Return::Reject;
                    }
                }

                Return::Capture(
                    Capture::Range(
                        runtime.reader.capture(reader_start),
                        matches!(self, Token::Match(_)) as u8
                    )
                )
            },

            Token::Char(accept) => {
                if let Some(c) = runtime.reader.next() {
                    if !accept.test(&(c..=c)) {
                        return Return::Reject;
                    }
                }
                else {
                    return Return::Reject;
                }

                Return::Capture(
                    Capture::Range(
                        runtime.reader.capture(reader_start), 0
                    )
                )
            },

            Token::Chars(accept) => {
                while let Some(c) = runtime.reader.peek() {
                    if !accept.test(&(c..=c)) {
                        break;
                    }

                    runtime.reader.next();
                }

                if reader_start == runtime.reader.tell() {
                    return Return::Reject;
                }

                Return::Capture(
                    Capture::Range(
                        runtime.reader.capture(reader_start),
                        1
                    )
                )
            },

            Token::Call(p) => {
                let parselet = program.get_parselet(p).unwrap();

                if let Return::Accept(value) = parselet.run(program, runtime) {
                    Return::Capture(
                        match value {
                            Some(value) => Capture::Value(value, 1),
                            None => Capture::Range(
                                runtime.reader.capture(reader_start), 0
                            )
                        }
                    )
                }
                else {
                    Return::Reject
                }
            },

            Token::Rust(func) => {
                func(runtime)
            },

            // Todo: Positive, Optional & Kleene

            _ => {
                Return::Reject
            }
        }
    }
}

impl Token {

    /** Creates a positive closure on the provided token,
        by introducing a new, repeating parselet.
    */
    pub fn into_positive(self, program: &mut Program) -> Token {
        let parselet = program.new_embedded_parselet();

        parselet.new_rule(
            vec![
                (parselet.to_call(), None),
                (self.clone(), None)
            ]
        );

        parselet.new_rule(
            vec![
                (self, None)
            ]
        );

        parselet.to_call()
    }

    /** Creates an optional closure on the provided token,
        by introducing a new, differencing parselet.
    */
    pub fn into_optional(self, program: &mut Program) -> Token {
        let parselet = program.new_embedded_parselet();

        parselet.new_rule(
            vec![
                (self, None)
            ]
        );

        parselet.new_rule(
            vec![
                (Token::None, None),
            ]
        );

        parselet.to_call()
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::None => write!(f, "None"),
            Token::Any => write!(f, "."),
            Token::Touch(s) => write!(f, "'{}'", s),
            Token::Match(s) => write!(f, "\"{}\"", s),
            Token::Char(c) => write!(f, "|{:?}|", c),
            Token::Chars(c) => write!(f, "|{:?}|+", c),
            Token::Call(p) => {
                match p {
                    CallBy::Index(idx) => write!(f, "@{}", idx),
                    CallBy::Name(name) => write!(f, "{}", name)
                }
            },
            Token::Rust(_) => write!(f, "{{rust-function}}"),
            Token::Sequence(seq) => {
                for item in seq {
                    write!(f, "{:?} ", item)?;
                }

                Ok(())
            },
            Token::Positive(t) => write!(f, "{:?}+", t),
            Token::Optional(t) => write!(f, "{:?}?", t),
            Token::Kleene(t) => write!(f, "{:?}*", t),
        }
    }
}

// --- Rule ---

#[derive(Debug)]
pub struct Rule {
    sequence: Sequence,
    nullable: bool,
    leftrec: bool,
    first: Ccl
}

impl Rule {
    fn new(sequence: Option<Sequence>) -> Self {
        return Self{
            sequence: sequence.unwrap_or_else(|| Vec::new()),
            nullable: false,
            leftrec: false,
            first: Ccl::new()
        };
    }

    fn run(&self, program: &Program, runtime: &mut Runtime) -> Return {
        let reader_start = runtime.reader.tell();
        let capture_start = runtime.capture.len();

        // Try to parse along sequence
        for (item, alias) in &self.sequence {
            match item.run(program, runtime) {
                Return::Reject => {
                    runtime.capture.truncate(capture_start);
                    runtime.reader.reset(reader_start);
                    return Return::Reject;
                },

                Return::Capture(capture) => {
                    runtime.capture.push(
                        (capture, alias.clone())
                    );
                },

                Return::Accept(value) => {
                    return Return::Accept(value);
                }
            }
        }

        // Empty sequence?
        if self.sequence.len() == 0 {
            Return::Accept(None)
        }
        // When sequence length is only one without an alias, return this value!
        /*
        else if runtime.capture.last().unwrap().1.is_none() {
            Return::Accept(Some(runtime.capture.pop().unwrap().0))
        }
        */
        // todo: What happens with $0, when explicitly set?
        else {
            let mut complex = Complex::new();

            // Collect any significant captures and values
            for (value, alias) in runtime.capture.drain(capture_start..) {
                let value = match value {
                    // Turn significant capture into string
                    Capture::Range(range, severity) if severity > 0 => {
                        Value::String(
                            runtime.reader.extract(&range)
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
                    return Return::Accept(Some(value.clone()))
                }
            }

            if complex.len() > 0 {
                // Return the complex when it contains something
                Return::Accept(Some(Value::Complex(complex).into_ref()))
            }
            else {
                Return::Accept(None)
            }
        }
    }
}

// --- Parselet ---

#[derive(Debug)]
pub struct Parselet {
    index: usize,
    rules: Vec<Rule>,

    embedded: bool,
    nullable: bool,
    leftrec: bool,

    pub lexem: bool,
    pub first: Ccl
}

impl Parselet {
    fn new(index: usize) -> Self {
        Self{
            index,
            rules: Vec::new(),
            embedded: false,
            nullable: false,
            leftrec: false,
            lexem: false,
            first: Ccl::new()
        }
    }

    /// Returns the index of the parselet, which is unique in a separate program.
    pub fn get_index(&self) -> usize {
        self.index
    }

    /** Makes a new rule inside the parselet, with an optional sequence.

    When a new rule was added, Program.finalize() should be recalled. */
    pub fn new_rule(&mut self, sequence: Sequence) {
        self.rules.push(Rule::new(Some(sequence)));
    }

    /// Return a Token::Call instance to this function.
    pub fn to_call(&self) -> Token {
        Token::Call(CallBy::Index(self.index))
    }

    // Sequentially executes all rules of the parselet until one succeeds.
    fn exec(&self, program: &Program, runtime: &mut Runtime,
        leftrec: bool) -> Return
    {
        for rule in &self.rules {
            // Skip left-recursive rules?
            if rule.leftrec != leftrec {
                continue
            }

            if let Return::Accept(value) = rule.run(program, runtime) {
                return Return::Accept(value)
            }

            runtime.reader.reset(runtime.reader_start);
            runtime.capture.truncate(runtime.capture_start);
        }

        Return::Reject
    }

    /// Run parselet with a given runtime.
    fn run(&self, program: &Program, runtime: &mut Runtime) -> Return {
        let mut state = Return::Reject;
        let reader_start = runtime.reader.tell();
        let mut reader_end: Option<usize> = None;

        // Check for an existing memo-entry, and return it in case of a match
        if let Some((mem_end, mem_state)) =
            runtime.memo.get(&(reader_start, self.index))
        {
            reader_end = Some(*mem_end);
            state = mem_state.clone();
        }

        if let Some(reader_end) = reader_end {
            runtime.reader.reset(reader_end);
            return state;
        }

        // Perform parselet call
        runtime.level += 1;
        let prev_top_reader_start = runtime.top_reader_start;
        let prev_top_capture_start = runtime.top_capture_start;
        let prev_reader_start = runtime.reader_start;
        let prev_capture_start = runtime.capture_start;

        let prev_flag_capture = runtime.flag_capture;

        runtime.reader_start = reader_start;

        if !self.embedded {
            runtime.top_capture_start = runtime.capture.len();
            runtime.top_reader_start = reader_start;
            runtime.capture.push((Capture::None, None));
        }

        runtime.capture_start = runtime.capture.len();

        if self.lexem {
            runtime.flag_capture = false;
        }

        if self.leftrec {
            // Left-recursive parselet is called in a loop until no more input
            // is consumed.

            let mut reader_end = reader_start;
           
            // Insert a fake memo entry to avoid endless recursion
            
            /* info: removing this fake entry does not affect program run!

            This is because of the leftrec parameter to Parselet::run(), 
            which only accepts non-left-recursive calls on the first run.
            As an additional fuse, this fake memo entry should anyway be kept.
            */
            runtime.memo.insert(
                (reader_start, self.index),
                (reader_end, Return::Reject)
            );
            
            let mut loops = 0;

            loop {
                let rec_state = self.exec(program, runtime, loops > 0);

                // Stop either on reject or when no more input was consumed
                if matches!(rec_state, Return::Reject)
                    || runtime.reader.tell() <= reader_end {
                    break;
                }

                state = rec_state;

                reader_end = runtime.reader.tell();
                runtime.memo.insert(
                    (reader_start, self.index),
                    (reader_end, state.clone())
                );

                runtime.reader.reset(runtime.reader_start);
                runtime.capture.truncate(runtime.capture_start);
                loops += 1;
            }

            runtime.reader.reset(reader_end);
        }
        else {
            // Non-left-recursive parselet can be called directly.
            state = self.exec(program, runtime, false);
        }

        // Top-level call with empty result saves capture.
        if runtime.level == 1 && matches!(state, Return::Accept(None)) {
            state = Return::Accept(runtime.get_capture(0));
        }

        // Clear captures and memoize current position and state.
        runtime.capture.truncate(runtime.capture_start);
        runtime.memo.insert(
            (reader_start, self.index),
            (runtime.reader.tell(), state.clone())
        );

        // Restore runtime positions
        runtime.top_reader_start = prev_top_reader_start;
        runtime.top_capture_start = prev_top_capture_start;
        runtime.reader_start = prev_reader_start;
        runtime.capture_start = prev_capture_start;

        runtime.flag_capture = prev_flag_capture;
        runtime.level -= 1;

        // Return state
        state
    }
}

#[macro_export]
macro_rules! token {
    ($program:expr, |$var:ident| $code:block) => {
        Token::Rust(|$var| $code)
    };
    ($program:expr, $ident:ident) => {
        Token::Call(CallBy::Name(stringify!($ident).to_string()))
    };
    ($program:expr, $literal:literal) => {
        Token::Touch($literal.to_string())
    };
    ($program:expr, $expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! modifier {
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
    ($program:expr, $( ( $( $token:tt )+ ) )* ) => {
        {
            let rule = sequence!($program, [ $( ( $($token)+ ) ),* ] );
            let parselet = $program.new_embedded_parselet();
            parselet.new_rule(rule);
            parselet.to_call()
        }
    };
    ($program:expr, $( $token:tt )+) => {
        token!($program, $($token)+)
    };
}

#[macro_export]
macro_rules! sequence {
    ($program:expr, [ $( ( $( $token:tt )+ ) ),* ] ) => {
        {
            vec![
                $(
                    (modifier!($program, $( $token )+), None)
                ),*
            ]
        }
    };
}

#[macro_export]
macro_rules! tokay {
    ($program:expr, $( $name:ident { $( => $( ( $( $token:tt )+ ) )* )+ } )+ )
        => {
        {
            $(
                let parselet = $program.new_parselet(
                    Some(stringify!($name))
                ).get_index();

                $(
                    let rule = sequence!($program, [ $( ( $($token)+ ) ),* ] );

                    $program.get_parselet_mut(
                        &CallBy::Index(parselet)
                    ).unwrap().new_rule(rule);
                )+
            )+
        }
    }
}

// --- Program ---

pub struct Program {
    named_parselets: HashMap<String, usize>,
    parselets: Vec<Parselet>
}

impl Program {
    pub fn new() -> Self {
        Self{
            named_parselets: HashMap::new(),
            parselets: Vec::new()
        }
    }

    /// Creates a new parselet with an optional name.
    pub fn new_parselet(&mut self, name: Option<&str>) -> &mut Parselet
    {
        if let Some(name) = name {
            self.named_parselets.insert(
                name.to_string(),
                self.parselets.len()
            );
        }

        self.parselets.push(Parselet::new(self.parselets.len()));
        self.parselets.last_mut().unwrap()
    }

    /// Create a new embedded parselet.
    pub fn new_embedded_parselet(&mut self) -> &mut Parselet {
        let parselet = self.new_parselet(None);
        parselet.embedded = true;
        parselet
    }

    fn get_parselet_idx(&self, id: &CallBy) -> Option<usize> {
        match id {
            CallBy::Name(name) => {
                if let Some(idx) = self.named_parselets.get(name) {
                    Some(*idx)
                } else {
                    None
                }
            },

            CallBy::Index(idx) => {
                if *idx < self.parselets.len() {
                    Some(*idx)
                } else {
                    None
                }
            }
        }
    }

    /// Returns a parselet either by index or name.
    pub fn get_parselet(&self, id: &CallBy) -> Option<&Parselet> {
        if let Some(idx) = self.get_parselet_idx(id) {
            Some(&self.parselets[idx])
        } else {
            None
        }
    }

    /// Returns a mutable parselet either by index or name.
    pub fn get_parselet_mut(&mut self, id: &CallBy) -> Option<&mut Parselet> {
        if let Some(idx) = self.get_parselet_idx(id) {
            Some(&mut self.parselets[idx])
        } else {
            None
        }
    }

    /** Finalizes the program, by finding nullable and left-recursive parselets
        and building FIRST()-sets of characters that may appear in the input. */
    pub fn finalize(&mut self) {
        let mut changes = true;

        while changes {
            changes = false;

            for p in 0..self.parselets.len() {
                let mut call: Vec<usize> = vec![p];
                let mut done: Vec<usize> = Vec::new();

                while call.len() > 0 {
                    let p = call.pop().unwrap();
                    done.push(p);

                    for r in 0..self.parselets[p].rules.len() {
                        let mut nullable = true;
                        let mut leftrec = false;
                        let mut first = Ccl::new();
    
                        for (token, _) in &self.parselets[p].rules[r].sequence
                        {
                            match token {
                                Token::Any => {
                                    // Any results in a FIRST() over all!
                                    first.clear();
                                    first.negate();
                                    nullable = false;
                                },

                                Token::Char(chars) | Token::Chars(chars) => {
                                    first.union(&chars);
                                    nullable = false;
                                },
    
                                Token::Match(string) | Token::Touch(string) => {
                                    if let Some(ch) = string.chars().next() {
                                        first.add(ch..=ch);
                                        nullable = false;
                                    }
                                },

                                Token::Call(p) => {
                                    let p = self.get_parselet_idx(p).unwrap();

                                    // Calling something from the done stack,
                                    // then this parselet is left-recursive.
                                    if let Some(_) = done.iter().position(
                                            |x| x == &p ) {
                                        leftrec = true;
                                    }
                                    // Otherwise, when not on the call-stack,
                                    // push it there.
                                    else if call.iter().position
                                            (|x| x == &p).is_none() {
                                        call.push(p);
                                    }

                                    first.union(&self.parselets[p].first);
                                    nullable = self.parselets[p].nullable;
                                },
    
                                _ => {}
                            }
    
                            if !nullable {
                                break;
                            }
                        }

                        let parselet = &mut self.parselets[p];
                        let rule = &mut parselet.rules[r];

                        // Is nullable?
                        if !rule.nullable && nullable {
                            rule.nullable = true;
                            parselet.nullable = true;
                            changes = true;
                        }

                        // Left-recursiveness
                        if !rule.leftrec && leftrec {
                            rule.leftrec = true;
                            parselet.leftrec = true;
                            changes = true;
                        }

                        // FIRST-set
                        if rule.first.union(&first) > 0 {
                            parselet.first.union(&first);
                            changes = true;
                        }
                    }
                }
            }
        }
    }

    /// Run the program with the given runtime and parselet.
    pub fn run(&self, runtime: &mut Runtime) -> Return {
        self.parselets[0].run(&self, runtime)
    }
}


// --- Runtime ---


#[derive(Debug, Clone)]
pub enum Return {
    Capture(Capture),           // Soft accept, capture & continue
    Accept(Option<RefValue>),   // Hard accept, take value & continue
    Reject                      // Reject
    // todo: Escape?
}

/*
impl Clone for Return {
    fn clone(&self) -> Self {
        match self {
            State::Capture(c) => State::Capture(Some(value.clone())),
            State::Capture(None) => State::Capture(None),
            State::Accept(Some(value)) => State::Accept(Some(value.clone())),
            State::Accept(None) => State::Accept(None),
            State::Reject => State::Reject
        }
    }
}
*/

/**
    Represents captures with different severities, mostly used for
    automatic syntax tree construction from rules.
*/
#[derive(Debug, Clone)]
pub enum Capture {
    None,                   // Empty capture
    Range(Range, u8),       // Range with severity
    Value(RefValue, u8),    // Value with severity
}

/**
    The Runtime structure represents

    - the current reader state
    - parse tree memoization,
    - and captures.

    It is shared by several functions in this module during a parse.
*/
pub struct Runtime {
    // Input & memoization
    reader: Reader,
    memo: HashMap<(usize, usize), (usize, Return)>,

    // Captures and overall value $0
    capture: Vec<(Capture, Option<String>)>,

    // Positions per top-level parselet
    top_reader_start: usize,    // Reader offset at current top-level parselet
    top_capture_start: usize,   // Capture offset at current top-level parselet

    // Positions per parselet
    reader_start: usize,        // Reader offset at current parselet
    capture_start: usize,       // Capture offset at current parselet

    flag_capture: bool,
    level: usize
}

impl Runtime {
    pub fn new(reader: Reader) -> Self {
        Self{
            reader,
            memo: HashMap::new(),
            capture: Vec::new(),
            top_reader_start: 0,
            top_capture_start: 0,
            reader_start: 0,
            capture_start: 0,
            flag_capture: true,
            level: 0
        }
    }

    // Skip one character, clear any memoization in front of new offset
    pub fn skip(&mut self) {
        self.reader.next();

        let offset = self.reader.tell();
        self.memo.retain(|&(start, _), _| start >= offset);
        self.reader.commit();
    }

    // Returns true when eof is reached
    pub fn is_eof(&self) -> bool {
        self.reader.eof
    }

    // Runtime state 
    pub fn stats(&self) {
        println!("---");
        println!("{} memo entries", self.memo.len());
        println!("current {:?}", &self.capture[self.top_capture_start..]);
        println!("capture {:?} (full)", self.capture);
        println!("---");
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        // Capture $0 is specially handled.
        if pos == 0 {
            return Some(self.get_value());
        }

        // Anything else by position.
        let pos = self.top_capture_start + pos;

        if pos >= self.capture.len() {
            return None
        }

        let replace = match &self.capture[pos].0 {
            Capture::None => {
                Capture::Value(
                    Value::Void.into_ref(), 0
                )
            },

            Capture::Range(range, sig) => {
                Capture::Value(
                    Value::String(self.reader.extract(range)).into_ref(), *sig
                )
            },
            
            Capture::Value(value, _) => {
                return Some(value.clone())
            }
        };

        self.capture[pos].0 = replace;
        self.get_capture(pos - self.top_capture_start)
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        // fixme: Maybe this should be examined in reversed order?
        for (i, capture) in self.capture[self.top_capture_start..].iter().enumerate() {
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
        if let Capture::Value(value, _) = &self.capture[self.top_capture_start].0 {
            return value.clone()
        }

        Value::String(
            self.reader.extract(
                &(self.top_reader_start..self.reader.tell())
            )).into_ref()
    }

    /** Save current $0 value */
    pub fn set_value(&mut self, value: RefValue) {
        self.capture[self.top_capture_start].0 = Capture::Value(value, 2)
    }

}
