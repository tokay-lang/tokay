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

// --- State ---

pub enum State {
    Capture(Option<RefValue>),  // Soft accept, capture & continue
    Accept(Option<RefValue>),   // Hard accept, take value & continue
    Reject                      // Reject
    // todo: Escape?
}

impl State {
    fn finalize<T: Read> (self, runtime: &mut Runtime<T>) -> State {
        match self {
            State::Capture(Some(value)) => State::Accept(Some(value)),

            // Take determined Accept or any Reject as is...
            State::Accept(Some(_)) | State::Reject => self,

            // otherwise, automatically determine return value from captures
            State::Accept(None) | State::Capture(None) => {
                if runtime.capture_start == runtime.capture.len() {
                    // There is no capture!
                    State::Accept(None)
                }
                else {
                    let mut complex = Complex::new();

                    // Collect any significant captures and values
                    for capture in runtime.capture.drain(runtime.capture_start..) {
                        let value = match capture.0 {
                            // Skip unsignificant capture.
                            Capture::Capture(_) | Capture::Value(_) => {
                                continue
                            },
                            // Turn significant capture into string
                            Capture::SigCapture(range) => {
                                Value::String(
                                    runtime.reader.extract(&range)
                                ).into_ref()
                            },
                            // Take value as is
                            Capture::SigValue(value) => {
                                value.clone()
                            }
                        };

                        // Named capture becomes complex key
                        if let Some(name) = capture.1 {
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
                            return State::Accept(Some(value.clone()))
                        }
                    }

                    if complex.len() > 0 {
                        // Return the complex when it contains something
                        State::Accept(Some(Value::Complex(complex).into_ref()))
                    }
                    else {
                        State::Accept(None)
                    }
                }
            }
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        match self {
            State::Capture(Some(value)) => State::Capture(Some(value.clone())),
            State::Capture(None) => State::Capture(None),
            State::Accept(Some(value)) => State::Accept(Some(value.clone())),
            State::Accept(None) => State::Accept(None),
            State::Reject => State::Reject
        }
    }
}

// --- Token ---

pub enum CallBy {
    Index(usize),
    Name(String)
}

pub enum Token<T> {
    None,
    Any,
    Touch(String),
    Match(String),
    Char(Ccl),
    Chars(Ccl),
    Call(CallBy),
    Rust(fn(&mut Runtime<T>) -> State),
}

impl<T> std::fmt::Debug for Token<T> {
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
        }
    }
}

// --- Rule ---

#[derive(Debug)]
pub struct Rule<T> {
    pub sequence: Vec<(Token<T>, Option<String>)>,

    nullable: bool,
    leftrec: bool,
    first: Ccl
}

impl<T: Read> Rule<T> {
    pub fn new() -> Self {
        return Self{
            sequence: Vec::new(),
            nullable: false,
            leftrec: false,
            first: Ccl::new()
        };
    }

    pub fn push_token(&mut self, token: Token<T>, alias: Option<&str>) {
        if let Some(alias) = alias {
            self.sequence.push((token, Some(alias.to_string())));
        }
        else {
            self.sequence.push((token, None));
        }
    }

    fn run(&self, program: &Program<T>, runtime: &mut Runtime<T>) -> State {
        for (token, alias) in &self.sequence {
            let reader_start = runtime.reader.tell();

            match token {
                Token::None => {
                    runtime.capture.push((
                        Capture::Capture(runtime.reader.capture(reader_start)),
                        alias.clone()));
                },

                Token::Any => {
                    if runtime.reader.next().is_none() {
                        return State::Reject;
                    }

                    runtime.capture.push((
                        Capture::SigCapture(runtime.reader.capture(reader_start)),
                        alias.clone()));
                },

                Token::Match(string) | Token::Touch(string) => {
                    for ch in string.chars() {
                        if let Some(c) = runtime.reader.next() {
                            if c != ch {
                                return State::Reject;
                            }
                        }
                        else {
                            return State::Reject;
                        }
                    }

                    runtime.capture.push((
                        if matches!(token, Token::Match(_)) {
                            Capture::SigCapture(
                                runtime.reader.capture(reader_start)
                            )
                        } else {
                            Capture::Capture(
                                runtime.reader.capture(reader_start)
                            )
                        },
                        alias.clone()));
                },

                Token::Char(accept) => {
                    if let Some(c) = runtime.reader.next() {
                        if !accept.test(&(c..=c)) {
                            return State::Reject;
                        }
                    }
                    else {
                        return State::Reject;
                    }

                    runtime.capture.push((
                        Capture::Capture(runtime.reader.capture(reader_start)),
                        alias.clone()));
                },

                Token::Chars(accept) => {
                    while let Some(c) = runtime.reader.peek() {
                        if !accept.test(&(c..=c)) {
                            break;
                        }

                        runtime.reader.next();
                    }

                    if reader_start == runtime.reader.tell() {
                        return State::Reject;
                    }

                    runtime.capture.push((
                        Capture::SigCapture(
                            runtime.reader.capture(reader_start)
                        ),
                        alias.clone()));
                },

                Token::Call(p) => {
                    let p = match p {
                        CallBy::Index(p) => *p,
                        CallBy::Name(p) => program.named_parselets[p]
                    };

                    if let State::Accept(value) = program.run(runtime, p) {
                        runtime.capture.push((
                            Capture::SigValue(
                                value.or(
                                    Some(Value::None.into_ref())
                                ).unwrap()),
                                alias.clone()));
                    }
                    else {
                        return State::Reject;
                    }
                },

                Token::Rust(func) => {
                    match func(runtime) {
                        State::Capture(value) => {
                            if let Some(value) = value {
                                runtime.capture.push((
                                    Capture::SigValue(value),
                                    alias.clone()
                                ));
                            }
                            else {
                                runtime.capture.push((
                                    Capture::Value(Value::None.into_ref()),
                                    alias.clone()
                                ));
                            }
                        },

                        other => {
                            return other
                        }
                    }
                }
            }
        }

        State::Accept(None)
    }
}

// --- Parselet ---

#[derive(Debug)]
pub struct Parselet<T> {
    pub rules: Vec<Rule<T>>,

    //embedded: bool,
    nullable: bool,
    leftrec: bool,

    pub lexem: bool,
    pub first: Ccl
}

impl<T: Read> Parselet<T> {
    pub fn new() -> Self {
        Self{
            rules: Vec::new(),
            //embedded: false,
            nullable: false,
            leftrec: false,
            lexem: false,
            first: Ccl::new()
        }
    }

    pub fn push_rule(&mut self, rule: Rule<T>) {
        self.rules.push(rule);
    }

    fn run(&self, program: &Program<T>, runtime: &mut Runtime<T>, leftrec: bool)
        -> State {
        
        for rule in &self.rules {
            // Skip left-recursive rules?
            if rule.leftrec != leftrec {
                continue
            }

            if let State::Accept(value) = rule.run(program, runtime) {
                //println!("{:?} accepts with {:?}, values={:?}", rule.sequence, value, runtime.capture);
                return State::Accept(value)
            }

            runtime.reader.reset(runtime.reader_start);
            runtime.capture.truncate(runtime.capture_start);
        }

        State::Reject
    }
}

#[macro_export]
macro_rules! token {
    (|$var:ident| $code:block) => {
        Token::Rust(|$var| $code)
    };
    ($ident:ident) => {
        Token::Call(CallBy::Name(stringify!($ident).to_string()))
    };
    ($literal:literal) => {
        Token::Touch($literal.to_string())
    };
    ($expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! tokay {
    ($program:expr, $( $name:ident { $( => $( ( $( $token:tt )+ ) )* )+ } )+ )
        => {
                {
                    $(
                        let mut parselet = $crate::tokay::Parselet::new();
                        $(
                            let mut rule = $crate::tokay::Rule::new();
                            $(
                                rule.push_token(token!($($token)*), None);
                            )*
                            parselet.push_rule(rule);
                        )+

                        $program.push_parselet(
                            Some(stringify!($name)), parselet
                        );
                    )+
                }
    }
}

// --- Program ---

pub struct Program<T> {
    named_parselets: HashMap<String, usize>,
    parselets: Vec<Parselet<T>>
}

impl<T: Read> Program<T> {
    pub fn new() -> Self {
        Self{
            named_parselets: HashMap::new(),
            parselets: Vec::new()
        }
    }

    pub fn push_parselet(&mut self, name: Option<&str>, parselet: Parselet<T>)
        -> usize
    {
        let id = self.parselets.len();

        if let Some(name) = name {
            self.named_parselets.insert(
                name.to_string(),
                id
            );
        }
        
        self.parselets.push(parselet);
        id
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
                                    let p = match p {
                                        CallBy::Index(p) => *p,
                                        CallBy::Name(p) => self.named_parselets[p]
                                    };

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

    /** Run the program with the given runtime and parselet. */
    pub fn run(&self, runtime: &mut Runtime<T>, index: usize) -> State {
        let mut state = State::Reject;
        let reader_start = runtime.reader.tell();
        let mut reader_end: Option<usize> = None;

        // Check for an existing memo-entry, and return it in case of a match
        if let Some((mem_end, mem_state)) =
                runtime.memo.get(&(reader_start, index))
        {
            reader_end = Some(*mem_end);
            state = mem_state.clone();
        }

        if let Some(reader_end) = reader_end {
            runtime.reader.reset(reader_end);
            return state;
        }

        // Perform parselet call
        let parselet = &self.parselets[index];

        runtime.level += 1;
        let prev_value = runtime.value.clone();
        let prev_reader_start = runtime.reader_start;
        let prev_capture_start = runtime.capture_start;
        let prev_flag_capture = runtime.flag_capture;

        runtime.reader_start = reader_start;
        runtime.capture_start = runtime.capture.len();

        if parselet.lexem {
            runtime.flag_capture = false;
        }

        if parselet.leftrec {
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
                (reader_start, index),
                (reader_end, State::Reject)
            );
            
            let mut loops = 0;

            loop {
                let rec_state = parselet.run(self, runtime, loops > 0);

                // Stop either on reject or when no more input was consumed
                if matches!(rec_state, State::Reject)
                    || runtime.reader.tell() <= reader_end {
                    break;
                }

                state = rec_state.finalize(runtime);

                reader_end = runtime.reader.tell();
                runtime.memo.insert(
                    (reader_start, index),
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
            state = parselet.run(self, runtime, false).finalize(runtime);
        }

        // Clear captures and memoize current position and state.
        runtime.capture.truncate(runtime.capture_start);
        runtime.memo.insert(
            (reader_start, index),
            (runtime.reader.tell(), state.clone())
        );

        // Restore runtime
        runtime.value = prev_value;
        runtime.reader_start = prev_reader_start;
        runtime.capture_start = prev_capture_start;
        runtime.flag_capture = prev_flag_capture;
        runtime.level -= 1;

        // Return state
        state
    }

    /** Skips until execution of the given parselet is reasonable. */
    pub fn skip(&self, runtime: &mut Runtime<T>, parselet: usize) {
        let parselet = &self.parselets[parselet];

        while let Some(c) = runtime.reader.peek() {
            if parselet.first.test(&(c..=c)) {
                break
            }

            runtime.reader.next();
        }
    }
}


// --- Runtime ---

/**
    Represents captures with different severities, mostly used for
    automatic syntax tree construction from rules.
*/
#[derive(Debug)]
enum Capture {
    Capture(Range),     // non-significant range, just used by semantics
    SigCapture(Range),  // significant range, turned into a parse tree leaf
    Value(RefValue),    // non-significant value
    SigValue(RefValue)  // significant value, turned into a parse tree leaf
                        // (or node, when using Value::Complex)
}

/**
    The Runtime structure represents

    - the current reader state
    - parse tree memoization,
    - and captures.

    It is shared by several functions in this module during a parse.
*/
pub struct Runtime<T> {
    // Input & memoization
    reader: Reader<T>,
    memo: HashMap<(usize, usize), (usize, State)>,

    // Captures and overall value $0
    capture: Vec<(Capture, Option<String>)>,
    value: Option<RefValue>,

    // Positions per parselet
    reader_start: usize,
    capture_start: usize,
    flag_capture: bool,
    level: usize
}

impl <T: Read> Runtime<T> {
    pub fn new(reader: Reader<T>) -> Self {
        Self{
            reader,
            memo: HashMap::new(),
            capture: Vec::new(),
            value: None,
            reader_start: 0,
            capture_start: 0,
            flag_capture: true,
            level: 0
        }
    }

    // Returns true when eof is reached
    pub fn is_eof(&self) -> bool {
        self.reader.eof
    }

    // Unfinished...
    pub fn clean(&mut self) {
        let offset = self.reader.tell();
        self.memo.retain(|&(start, _), _| start >= offset);
        self.reader.commit();
    }

    // Runtime state 
    pub fn stats(&self) {
        println!("---");
        println!("{} memo entries", self.memo.len());
        println!("current {:?}", &self.capture[self.capture_start..]);
        println!("capture {:?} (full)", self.capture);
        println!("---");
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        if pos == 0 {
            return Some(self.get_value());
        }

        let pos = self.capture_start + pos - 1;

        if pos >= self.capture.len() {
            return None
        }

        let replace = match &self.capture[pos].0 {
            Capture::Capture(range) => {
                Capture::Value(
                    Value::String(self.reader.extract(range)).into_ref()
                )
            },
            
            Capture::SigCapture(range) => {
                Capture::SigValue(
                    Value::String(self.reader.extract(range)).into_ref()
                )
            },

            Capture::Value(value) | Capture::SigValue(value) => {
                return Some(value.clone())
            }
        };

        self.capture[pos].0 = replace;
        self.get_capture(pos - self.capture_start + 1)
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        for (i, capture) in self.capture[self.capture_start..].iter().enumerate() {
            if let Some(alias) = &capture.1 {
                if alias == name {
                    return self.get_capture(i + 1);
                }
            }
        }

        None
    }

    /** Returns the current reference value */
    pub fn get_value(&self) -> RefValue {
        if let Some(value) = &self.value {
            return value.clone()
        }

        Value::String(
            self.reader.extract(
                &(self.reader_start..self.reader.tell())
            )).into_ref()
    }

    /** Save current reference value */
    pub fn set_value(&mut self, value: RefValue) {
        self.value = Some(value)
    }

}
