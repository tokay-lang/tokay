//! Contexts and stack frames for parselet calls.
use super::*;
use crate::reader::Offset;
use crate::value::{Dict, List, Object, Parselet, RefValue};
use std::iter::FromIterator;

/** Representation of a stack-frame based on current context. */
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub fuse: Option<usize>,  // optional fuse
    pub capture_start: usize, // capture start
    pub reader_start: Offset, // reader start
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "capture: {}, reader: {}, fuse: {:?}",
            self.capture_start, self.reader_start.offset, self.fuse
        )
    }
}

/** Representatin of a loop-frame */
#[derive(Debug, Clone, Copy)]
pub struct Loop {
    pub frames: usize, // Number of frames at loop start
    pub start: usize,  // Start address of loop iteration
    pub end: usize,    // End address of loop
}

/** Contexts represent stack frames for parselet calls.

Within the context, most operations regarding capture storing and loading is performed. */
pub struct Context<'program, 'reader, 'thread, 'parselet> {
    // References
    pub thread: &'thread mut Thread<'program, 'reader>, // Current VM thread
    pub parselet: &'parselet Parselet,                  // Current parselet

    pub depth: usize, // Recursion depth

    // Positions
    pub stack_start: usize, // Stack start (including locals and parameters)
    hold: usize,            // Defines number of stack items to hold on context drop

    // Virtual machine
    pub frames: Vec<Frame>, // Frame stack
    pub frame: Frame,       // Current frame

    pub loops: Vec<Loop>, // Loop stack

    // Variables
    pub source_offset: Option<Offset>, // Tokay source offset needed for error reporting
}

impl<'program, 'reader, 'thread, 'parselet> Context<'program, 'reader, 'thread, 'parselet> {
    pub fn new(
        thread: &'thread mut Thread<'program, 'reader>,
        parselet: &'parselet Parselet,
        locals: usize,
        take: usize,
        hold: usize,
        depth: usize,
    ) -> Self {
        let stack_start = thread.stack.len() - take;

        /*
        println!("--- {:?} ---", parselet.name);
        println!("stack = {:#?}", thread.stack);
        println!("stack = {:?}", thread.stack.len());
        println!("start = {:?}", stack_start);
        println!("resize = {:?}", stack_start + locals + 1);
        */

        // Initialize local variables and $0
        thread
            .stack
            .resize(stack_start + locals + 1, Capture::Empty);

        // Create context frame0
        let frame = Frame {
            fuse: None,
            capture_start: stack_start + locals + 1,
            reader_start: thread.reader.tell(),
        };

        // Create Context
        Self {
            thread,
            parselet,
            depth,
            stack_start,
            hold,
            frames: Vec::new(),
            frame,
            loops: Vec::new(),
            source_offset: None,
        }
    }

    /// Print debug output with context depth indention
    #[inline]
    pub fn log(&self, msg: &str) {
        println!(
            "{}{}{:5} {}",
            ".".repeat(self.depth),
            //self.parselet.name.as_deref().unwrap_or("(unnamed)"), // fixme: TEMPORAL!
            self.parselet.name,
            //if self.parselet.consuming.is_some() {
            if self.parselet.consuming.is_some() {
                format!("@{: <4}", self.thread.reader.tell().offset)
            } else {
                "".to_string()
            },
            msg
        );
    }

    /// Shortcut for an Ok(Accept::Push) with the given value.
    /// To push a value immediatelly, use context.thread.stack.push().
    #[inline]
    pub fn push(&self, value: RefValue) -> Result<Accept, Reject> {
        Ok(Accept::Push(Capture::Value(value, None, 10)))
    }

    /// Pop value off the stack.
    #[inline]
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let mut capture = self.thread.stack.pop().unwrap();
        capture.extract(&mut self.thread.reader)
    }

    /// Peek top value of stack.
    #[inline]
    pub fn peek(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.thread.stack.last_mut().unwrap();
        capture.extract(&mut self.thread.reader)
    }

    // Push a value onto the stack
    #[inline]
    pub fn load(&mut self, index: usize) -> Result<Accept, Reject> {
        let capture = &mut self.thread.stack[index];
        let value = capture.extract(&self.thread.reader);
        self.push(value)
    }

    // Reset context stack state
    #[inline]
    fn reset(&mut self, offset: Option<Offset>) {
        self.thread.stack.truncate(self.frame.capture_start - 1); // Truncate stack including $0
        self.thread.stack.push(Capture::Empty); // re-initialize $0

        if let Some(offset) = offset {
            self.frame.reader_start = offset; // Set reader start to provided position
        }
    }

    /// Return top-level frame
    pub fn frame0(&self) -> &Frame {
        if self.frames.is_empty() {
            &self.frame
        } else {
            &self.frames[0]
        }
    }

    /// Return mutable top-level frame
    pub fn frame0_mut(&mut self) -> &mut Frame {
        if self.frames.is_empty() {
            &mut self.frame
        } else {
            &mut self.frames[0]
        }
    }

    /** Return a capture by index as RefValue. */
    pub fn get_capture(&mut self, pos: usize) -> Option<RefValue> {
        let frame0 = self.frame0();
        let capture_start = frame0.capture_start;
        let reader_start = frame0.reader_start;

        let pos = capture_start + pos - 1;

        if pos >= self.thread.stack.len() {
            None
        }
        // This is $0?
        else if pos < capture_start {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, ..) = &self.thread.stack[pos] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(RefValue::from(
                self.thread
                    .reader
                    .get(&self.thread.reader.capture_from(&reader_start)),
            ))
        // Any other index.
        } else {
            self.thread.stack[pos].degrade();
            Some(self.thread.stack[pos].extract(&self.thread.reader))
        }
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        let capture_start = self.frame0().capture_start;
        let tos = self.thread.stack.len();

        for i in (0..tos - capture_start).rev() {
            let capture = &mut self.thread.stack[capture_start + i];

            if capture.alias(name) {
                capture.degrade();
                return Some(capture.extract(&self.thread.reader));
            }
        }

        None
    }

    /** Set a capture to a RefValue by index. */
    pub fn set_capture(&mut self, pos: usize, value: RefValue) {
        let capture_start = self.frame0().capture_start;
        let pos = capture_start + pos - 1;

        if pos >= self.thread.stack.len() {
            return;
        }

        let capture = &mut self.thread.stack[pos];

        // $0 gets a higher severity than normal captures.
        let severity = if pos < capture_start { 10 } else { 5 };

        match capture {
            Capture::Empty => *capture = Capture::Value(value, None, severity),
            Capture::Range(_, alias, _) => {
                *capture = Capture::Value(value, alias.clone(), severity)
            }
            Capture::Value(capture_value, ..) => {
                *capture_value = value;
            }
        }
    }

    /** Set a capture to a RefValue by name. */
    pub fn set_capture_by_name(&mut self, name: &str, value: RefValue) {
        let capture_start = self.frame0().capture_start;
        let tos = self.thread.stack.len();

        for i in (0..tos - capture_start).rev() {
            let capture = &mut self.thread.stack[capture_start + i];

            if capture.alias(name) {
                match capture {
                    Capture::Empty => *capture = Capture::Value(value, None, 5),
                    Capture::Range(_, alias, _) => {
                        *capture = Capture::Value(value, alias.clone(), 5)
                    }
                    Capture::Value(capture_value, ..) => {
                        *capture_value = value;
                    }
                }
                break;
            }
        }
    }

    /** Collect captures from a capture_start and turn them either into a dict or list object capture.

    Any items with a severity of at least 1 are being collected, but higher severities always win.

    - Results of a collection (either list or dict) inherit the highest collected severity
    - Token severity:
      - 0: Whitespace
      - 1: Touch
      - 5: Match, character-class, parselet
      - 10: Defined value

    This function is internally used for automatic AST construction and value inheriting.
    */
    pub fn collect(
        &mut self,
        capture_start: usize, // Stack offset to start from
        copy: bool,           // Copy values instead of draining them from the stack
        debug: bool,          // Print debug information
    ) -> Capture {
        // Early abort when capture_start is behind stack len
        if capture_start > self.thread.stack.len() {
            return Capture::Empty;
        }

        assert!(capture_start >= self.frame0().capture_start);

        // Eiter copy or drain captures from stack
        let captures: Vec<Capture> = if copy {
            // fixme: copy feature isn't used...
            Vec::from_iter(
                self.thread.stack[capture_start..]
                    .iter()
                    .filter(|item| !(matches!(item, Capture::Empty)))
                    .cloned(),
            )
        } else {
            self.thread
                .stack
                .drain(capture_start..)
                .filter(|item| !(matches!(item, Capture::Empty)))
                .collect()
        };

        if debug {
            self.log(&format!("collect captures = {}", captures.len()));

            for i in 0..captures.len() {
                self.log(&format!(" {}: {:?}", i, captures[i]));
            }
        }

        // Early abort when no valuable captures had been taken
        if captures.len() == 0 {
            return Capture::Empty;
        }

        // Capture inheritance is only possible when there is only one capture available
        let inherit = captures.len() == 1;

        let mut list = List::new(); // List collector
        let mut dict = Dict::new(); // Dict collector
        let mut max = 0; // Maximum severity
        let mut idx = 0; // Keep the order for dicts

        // Collect any significant captures and values
        // fixme: This part contains ugly and redundant code; must be reworked later.
        for capture in captures.into_iter() {
            match capture {
                Capture::Range(range, alias, severity) if severity >= max => {
                    // On higher severity, drop all results collected so far
                    if severity > max {
                        idx = 0;
                        max = severity;
                        list.clear();
                        dict.clear();
                    }

                    // fixme: This line is the only difference between the Capture::Range and Capture::Value branch.
                    //        This is totally ugly and should be reworked.
                    let value = RefValue::from(self.thread.reader.get(&range));

                    if let Some(alias) = alias {
                        // Move list items into dict when this is the first entry
                        if dict.is_empty() {
                            for (i, item) in list.drain(..).enumerate() {
                                dict.insert(RefValue::from(i), item);
                            }
                        }

                        dict.insert(RefValue::from(alias), value);
                    } else if inherit {
                        return Capture::Value(value, alias, severity);
                    } else if !value.is_void() {
                        // Eiher collect into list, or insert into the dict
                        if dict.is_empty() {
                            list.push(value);
                        } else {
                            dict.insert(RefValue::from(idx), value);
                        }
                    }

                    idx += 1;
                }

                Capture::Value(value, alias, severity) if severity >= max => {
                    // On higher severity, drop all results collected so far
                    if severity > max {
                        idx = 0;
                        max = severity;
                        list.clear();
                        dict.clear();
                    }

                    if let Some(alias) = alias {
                        // Move list items into dict when this is the first entry
                        if dict.is_empty() {
                            for (i, item) in list.drain(..).enumerate() {
                                dict.insert(RefValue::from(i), item);
                            }
                        }

                        dict.insert(RefValue::from(alias), value);
                    } else if inherit {
                        return Capture::Value(value, alias, severity);
                    } else if !value.is_void() {
                        // Eiher collect into list, or insert into the dict
                        if dict.is_empty() {
                            list.push(value);
                        } else {
                            dict.insert(RefValue::from(idx), value);
                        }
                    }

                    idx += 1;
                }

                _ => {}
            };
        }

        if debug {
            self.log(&format!("list = {:?}", list));
            self.log(&format!("dict = {:?}", dict));
        }

        if dict.is_empty() {
            if list.len() > 1 {
                Capture::Value(RefValue::from(list), None, max)
            } else if list.len() == 1 {
                Capture::Value(list.pop().unwrap(), None, max)
            } else {
                Capture::Empty
            }
        } else {
            Capture::Value(RefValue::from(dict), None, max)
        }
    }

    /// Drains n items off the stack into a vector of values
    pub fn drain(&mut self, n: usize) -> Vec<RefValue> {
        let tos = self.thread.stack.len();
        assert!(n <= tos - self.frame0().capture_start);

        let captures: Vec<Capture> = self
            .thread
            .stack
            .drain(tos - n..)
            .filter(|capture| !matches!(capture, Capture::Empty))
            .collect();

        captures
            .into_iter()
            .map(|mut capture| capture.extract(&self.thread.reader))
            .collect()
    }

    // Execute VM opcodes in a context.
    // This function is a wrapper for Op::run() which post-processes the result.
    fn execute(&mut self, ops: &[Op]) -> Result<Accept, Reject> {
        let mut state = Op::run(ops, self);

        match state {
            // In case state is Accept::Next, try to return a capture
            Ok(Accept::Next) => {
                // Either take $0 when set
                if let Capture::Value(value, ..) =
                    &mut self.thread.stack[self.frame.capture_start - 1]
                {
                    state = Ok(Accept::Push(Capture::Value(
                        value.clone(),
                        None,
                        self.parselet.severity,
                    )));
                // Otherwise, push last value
                } else if self.thread.stack.len() > self.frame.capture_start {
                    state = Ok(Accept::Push(self.thread.stack.pop().unwrap())
                        .into_push(self.parselet.severity));
                }
            }

            // Patch context source position on error, if no other position already set
            Err(Reject::Error(ref mut err)) => {
                if let Some(source_offset) = self.source_offset {
                    err.patch_offset(source_offset);
                }
            }

            _ => {}
        }

        if self.thread.debug > 3 {
            self.log(&format!("final state = {:?}", state));
        }

        state
    }

    /// Run the current context with the associated parselet
    pub fn run(&mut self, main: bool) -> Result<Accept, Reject> {
        if main {
            return self.run_as_main();
        }

        // Debugging
        if self.thread.debug < 3 {
            if let Ok(inspect) = std::env::var("TOKAY_INSPECT") {
                for name in inspect.split(" ") {
                    if name == self.parselet.name {
                        self.thread.debug = 6;
                        break;
                    }
                }
            }
        }

        // collected results (from repeated parselet)
        let mut retlist = List::new();

        // Begin
        let mut ret = match self.execute(&self.parselet.begin) {
            Ok(Accept::Next) | Err(Reject::Skip) => Capture::Empty,
            Ok(Accept::Push(capture)) => {
                self.reset(Some(self.thread.reader.tell()));
                capture
            }
            Ok(Accept::Repeat(value)) => {
                if let Some(value) = value {
                    retlist.push(value);
                }

                self.reset(Some(self.thread.reader.tell()));
                Capture::Empty
            }
            Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
            other => return other,
        };

        // Body
        let mut first = true;
        ret = loop {
            match self.execute(&self.parselet.body) {
                Err(Reject::Skip) => {}
                Ok(Accept::Next) => break ret,
                Ok(Accept::Push(capture)) => break capture,
                Ok(Accept::Repeat(value)) => {
                    if let Some(value) = value {
                        retlist.push(value);
                    }
                }
                Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
                Err(Reject::Next) if !first => break Capture::Empty,
                other => return other,
            }

            // Reset capture stack for loop repeat
            self.reset(Some(self.thread.reader.tell()));
            first = false;
        };

        // End
        ret = match self.execute(&self.parselet.end) {
            Ok(Accept::Next) | Err(Reject::Skip) => ret,
            Ok(Accept::Push(capture)) => capture,
            Ok(Accept::Repeat(value)) => {
                if let Some(value) = value {
                    retlist.push(value);
                }

                ret
            }
            Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
            other => return other,
        };

        // retlist has higher priority than ret
        if !retlist.is_empty() {
            ret = Capture::Value(
                if retlist.len() > 1 {
                    RefValue::from(retlist)
                } else {
                    retlist.pop().unwrap()
                },
                None,
                self.parselet.severity,
            );
        }

        Ok(Accept::Push(ret).into_push(self.parselet.severity))
    }

    /** Run the current context as a main parselet.

    __main__-parselets are executed differently, as they handle unrecognized input as whitespace or gap,
    by skipping over it. __main__ parselets do also operate on multiple input Readers by sequence inside
    of the Context's thread.
    */
    fn run_as_main(&mut self) -> Result<Accept, Reject> {
        // collected results
        let mut retlist = List::new();

        // Begin
        match self.execute(&self.parselet.begin) {
            Ok(Accept::Next) | Err(Reject::Skip) | Ok(Accept::Push(Capture::Empty)) => {}
            Ok(Accept::Push(mut capture)) => {
                retlist.push(capture.extract(&self.thread.reader));
            }
            Ok(Accept::Repeat(value)) => {
                if let Some(value) = value {
                    retlist.push(value);
                }
            }
            Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
            other => return other,
        };

        loop {
            self.reset(Some(self.thread.reader.tell()));

            // Body
            loop {
                match self.execute(&self.parselet.body) {
                    Err(Reject::Next)
                    | Err(Reject::Skip)
                    | Ok(Accept::Next)
                    | Ok(Accept::Push(Capture::Empty)) => {}
                    Ok(Accept::Push(mut capture)) => {
                        retlist.push(capture.extract(&self.thread.reader));
                    }
                    Ok(Accept::Repeat(value)) => {
                        if let Some(value) = value {
                            retlist.push(value);
                        }
                    }
                    Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
                    other => return other,
                }

                if self.frame.reader_start == self.thread.reader.tell() {
                    // Skip one character if nothing was consumed
                    self.thread.reader.next();

                    // Drop all memoizations
                    self.thread.memo.clear();
                }

                // Reset capture stack for loop repeat
                self.reset(Some(self.thread.reader.tell()));

                // Break on EOF
                if self.thread.reader.eof() {
                    break;
                }
            }

            if self.thread.readers.is_empty() {
                break;
            }

            // Change reader within thread, and continue
            self.thread.reader = self.thread.readers.remove(0);

            // Drop all memoizations
            self.thread.memo.clear();
        }

        // End
        match self.execute(&self.parselet.end) {
            Ok(Accept::Next) | Err(Reject::Skip) | Ok(Accept::Push(Capture::Empty)) => {}
            Ok(Accept::Push(mut capture)) => {
                retlist.push(capture.extract(&self.thread.reader));
            }
            Ok(Accept::Repeat(value)) => {
                if let Some(value) = value {
                    retlist.push(value);
                }
            }
            Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
            other => return other,
        };

        // retlist has higher priority than ret
        if !retlist.is_empty() {
            Ok(Accept::Push(Capture::Value(
                if retlist.len() > 1 {
                    RefValue::from(retlist)
                } else {
                    retlist.pop().unwrap()
                },
                None,
                self.parselet.severity,
            )))
        } else {
            Ok(Accept::Push(Capture::Empty))
        }
    }
}

impl<'program, 'reader, 'thread, 'parselet> Drop
    for Context<'program, 'reader, 'thread, 'parselet>
{
    fn drop(&mut self) {
        self.thread.stack.truncate(self.stack_start + self.hold);
    }
}
