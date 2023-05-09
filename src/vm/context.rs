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

Via the context, most operations regarding capture storing and loading is performed. */
pub struct Context<'program, 'parselet, 'runtime> {
    // References
    pub program: &'program Program,     // Program
    pub parselet: &'parselet Parselet,  // Current parselet that is executed
    pub runtime: &'runtime mut Runtime, // Overall runtime

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

impl<'program, 'parselet, 'runtime> Context<'program, 'parselet, 'runtime> {
    pub fn new(
        program: &'program Program,
        parselet: &'parselet Parselet,
        runtime: &'runtime mut Runtime,
        locals: usize,
        take: usize,
        hold: usize,
        depth: usize,
    ) -> Self {
        let stack_start = runtime.stack.len() - take;

        /*
        println!("--- {:?} ---", parselet.name);
        println!("stack = {:#?}", runtime.stack);
        println!("stack = {:?}", runtime.stack.len());
        println!("start = {:?}", stack_start);
        println!("resize = {:?}", stack_start + locals + 1);
        */

        // Initialize local variables and $0
        runtime
            .stack
            .resize(stack_start + locals + 1, Capture::Empty);

        // Create context frame0
        let frame = Frame {
            fuse: None,
            capture_start: stack_start + locals + 1,
            reader_start: runtime.reader.tell(),
        };

        // Create Context
        Self {
            program,
            parselet,
            runtime,
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
                format!("@{: <4}", self.runtime.reader.tell().offset)
            } else {
                "".to_string()
            },
            msg
        );
    }

    /// Shortcut for an Ok(Accept::Push) with the given value.
    /// To push a value immediatelly, use context.runtime.stack.push().
    #[inline]
    pub fn push(&self, value: RefValue) -> Result<Accept, Reject> {
        Ok(Accept::Push(Capture::Value(value, None, 10)))
    }

    /// Pop value off the stack.
    #[inline]
    pub fn pop(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let mut capture = self.runtime.stack.pop().unwrap();
        capture.extract(&mut self.runtime.reader)
    }

    /// Peek top value of stack.
    #[inline]
    pub fn peek(&mut self) -> RefValue {
        // todo: check for context limitations on the stack?
        let capture = self.runtime.stack.last_mut().unwrap();
        capture.extract(&mut self.runtime.reader)
    }

    // Push a value onto the stack
    #[inline]
    pub fn load(&mut self, index: usize) -> Result<Accept, Reject> {
        let capture = &mut self.runtime.stack[index];
        let value = capture.extract(&self.runtime.reader);
        self.push(value)
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

        if pos >= self.runtime.stack.len() {
            None
        }
        // This is $0?
        else if pos < capture_start {
            // Capture 0 either returns an already set value or ...
            if let Capture::Value(value, ..) = &self.runtime.stack[pos] {
                return Some(value.clone());
            }

            // ...returns the current range read so far.
            Some(RefValue::from(
                self.runtime
                    .reader
                    .get(&self.runtime.reader.capture_from(&reader_start)),
            ))
        // Any other index.
        } else {
            self.runtime.stack[pos].degrade();
            Some(self.runtime.stack[pos].extract(&self.runtime.reader))
        }
    }

    /** Return a capture by name as RefValue. */
    pub fn get_capture_by_name(&mut self, name: &str) -> Option<RefValue> {
        let capture_start = self.frame0().capture_start;
        let tos = self.runtime.stack.len();

        for i in (0..tos - capture_start).rev() {
            let capture = &mut self.runtime.stack[capture_start + i];

            if capture.alias(name) {
                capture.degrade();
                return Some(capture.extract(&self.runtime.reader));
            }
        }

        None
    }

    /** Set a capture to a RefValue by index. */
    pub fn set_capture(&mut self, pos: usize, value: RefValue) {
        let capture_start = self.frame0().capture_start;
        let pos = capture_start + pos - 1;

        if pos >= self.runtime.stack.len() {
            return;
        }

        let capture = &mut self.runtime.stack[pos];

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
        let tos = self.runtime.stack.len();

        for i in (0..tos - capture_start).rev() {
            let capture = &mut self.runtime.stack[capture_start + i];

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
        if capture_start > self.runtime.stack.len() {
            return Capture::Empty;
        }

        assert!(capture_start >= self.frame0().capture_start);

        // Eiter copy or drain captures from stack
        let captures: Vec<Capture> = if copy {
            // fixme: copy feature isn't used...
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
                    let value = RefValue::from(self.runtime.reader.get(&range));

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
        let tos = self.runtime.stack.len();
        assert!(n <= tos - self.frame0().capture_start);

        let captures: Vec<Capture> = self
            .runtime
            .stack
            .drain(tos - n..)
            .filter(|capture| !matches!(capture, Capture::Empty))
            .collect();

        captures
            .into_iter()
            .map(|mut capture| capture.extract(&self.runtime.reader))
            .collect()
    }

    /// Run the current context with the associated parselet
    pub fn run(&mut self, main: bool) -> Result<Accept, Reject> {
        // Initialize parselet execution loop
        let mut results = List::new();
        let mut first = self.parselet.begin.len() > 0;
        let mut state = if self.parselet.begin.len() == 0 {
            None
        } else {
            Some(true)
        };

        // Debugging
        if self.runtime.debug < 3 {
            if let Ok(inspect) = std::env::var("TOKAY_INSPECT") {
                for name in inspect.split(" ") {
                    if name == self.parselet.name {
                        self.runtime.debug = 6;
                        break;
                    }
                }
            }
        }

        let result = loop {
            let capture_start = self.frame.capture_start;

            let ops = match state {
                // begin
                Some(true) => &self.parselet.begin,

                // end
                Some(false) => &self.parselet.end,

                // default
                None => &self.parselet.body,
            };

            let mut result = Op::execute(ops, self);

            // Either take $0 if it was set to a value, or
            // use any last remaining value as result,
            // if available
            if let Ok(Accept::Next) = result {
                let dollar0 = &mut self.runtime.stack[capture_start - 1];

                if let Capture::Value(value, ..) = dollar0 {
                    result = Ok(Accept::Push(value.clone().into()));
                } else if self.runtime.stack.len() > capture_start {
                    result = Ok(Accept::Push(self.runtime.stack.pop().unwrap()));
                }
            }

            // Debug
            if self.runtime.debug > 2 {
                self.log(&format!("returns {:?}", state));
            }

            /*
                In case this is the main parselet, try matching main as much
                as possible. This is only the case when input is consumed.
            */
            if main {
                //println!("main result(1) = {:#?}", result);
                result = match result {
                    Ok(Accept::Next) => Ok(Accept::Repeat(None)),
                    Ok(Accept::Return(value)) => Ok(Accept::Repeat(value)),
                    Ok(Accept::Push(capture)) => Ok(Accept::Repeat(match capture {
                        Capture::Range(range, ..) => {
                            Some(RefValue::from(self.runtime.reader.get(&range)))
                        }
                        Capture::Value(value, ..) => Some(value),
                        _ => None,
                    })),
                    result => result,
                };
                //println!("main result(2) = {:#?}", result);
            }

            // if main {
            //     println!("state = {:?} result = {:?}", state, result);
            // }

            // Evaluate result of parselet loop.
            match result {
                Ok(accept) => {
                    match accept {
                        Accept::Hold => break Some(Ok(Accept::Next)),

                        Accept::Return(value) => {
                            if let Some(value) = value {
                                break Some(Ok(Accept::Push(Capture::Value(
                                    value,
                                    None,
                                    self.parselet.severity,
                                ))));
                            } else {
                                break Some(Ok(Accept::Push(Capture::Empty)));
                            }
                        }

                        Accept::Repeat(value) => {
                            if let Some(value) = value {
                                results.push(value);
                            }
                        }

                        Accept::Push(mut capture)
                            if capture.get_severity() > self.parselet.severity =>
                        {
                            capture.set_severity(self.parselet.severity);
                            break Some(Ok(Accept::Push(capture)));
                        }

                        accept => {
                            if results.len() > 0 {
                                break None;
                            }

                            break Some(Ok(accept));
                        }
                    }

                    if main {
                        // In case no input was consumed in main loop, skip character
                        if state.is_none()
                            && self
                                .runtime
                                .reader
                                .capture_from(&self.frame.reader_start)
                                .len()
                                == 0
                        {
                            self.runtime.reader.next();
                        }

                        // Clear input buffer
                        self.runtime.reader.commit();

                        // Clear memo table
                        self.runtime.memo.clear();
                    }
                }

                Err(reject) => {
                    match reject {
                        Reject::Skip => {
                            if main && state.is_none() {
                                continue;
                            }

                            break Some(Ok(Accept::Next));
                        }
                        Reject::Error(mut err) => {
                            // Patch source position on error, when no position already set
                            if let Some(source_offset) = self.source_offset {
                                err.patch_offset(source_offset);
                            }

                            break Some(Err(Reject::Error(err)));
                        }
                        Reject::Main if !main => break Some(Err(Reject::Main)),
                        _ => {}
                    }

                    // Skip character and reset reader start
                    if main && state.is_none() {
                        self.runtime.reader.next();
                        self.frame.reader_start = self.runtime.reader.tell();
                    } else if results.len() > 0 && state.is_none() {
                        state = Some(false);
                        continue;
                    } else if state.is_none() {
                        break Some(Err(reject));
                    }
                }
            }

            if let Some(false) = state {
                break None;
            } else if !first && self.runtime.reader.eof() {
                state = Some(false);
            } else {
                state = None;
            }

            // Reset capture stack for loop repeat
            self.runtime.stack.truncate(self.frame.capture_start - 1); // Truncate stack including $0
            self.runtime.stack.push(Capture::Empty); // re-initialize $0
            self.frame.reader_start = self.runtime.reader.tell(); // Reset reader

            first = false;
        };

        match result {
            Some(result) if !matches!(result, Ok(Accept::Next)) => result,
            _ => {
                if results.len() > 1 {
                    Ok(Accept::Push(Capture::Value(
                        RefValue::from(results),
                        None,
                        self.parselet.severity,
                    )))
                } else if results.len() == 1 {
                    Ok(Accept::Push(Capture::Value(
                        results.pop().unwrap(),
                        None,
                        self.parselet.severity,
                    )))
                } else {
                    Ok(Accept::Push(Capture::Empty))
                }
            }
        }
    }

    /// Run the current context with the associated parselet
    pub fn run2(&mut self, main: bool) -> Result<Accept, Reject> {
        // collected results (from repeated parselet)
        let mut retlist = List::new();

        // Debugging
        if self.runtime.debug < 3 {
            if let Ok(inspect) = std::env::var("TOKAY_INSPECT") {
                for name in inspect.split(" ") {
                    if name == self.parselet.name {
                        self.runtime.debug = 6;
                        break;
                    }
                }
            }
        }

        // Save start of capture for $0
        let capture_start = self.frame.capture_start;

        if !self.parselet.begin.is_empty() {
            // Begin
            match Op::execute(&self.parselet.begin, self) {
                Ok(Accept::Next) => {}
                Ok(Accept::Repeat(value)) => {
                    if let Some(value) = value {
                        retlist.push(value);
                    }

                    self.frame.reader_start = self.runtime.reader.tell(); // Set reader start to current position
                }
                Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
                other => return other,
            }

            // Reset capture stack for loop repeat
            self.runtime.stack.truncate(self.frame.capture_start - 1); // Truncate stack including $0
            self.runtime.stack.push(Capture::Empty); // re-initialize $0
        }

        // Body
        let mut first = true;

        loop {
            match Op::execute(&self.parselet.body, self) {
                Ok(Accept::Next) if main => {}
                Err(Reject::Next) if main => {
                    // When in main, skip one char!
                    if self.runtime.reader.next().is_none() {
                        break;
                    }
                }
                Ok(Accept::Next) => break,
                Ok(Accept::Repeat(value)) => {
                    if let Some(value) = value {
                        retlist.push(value);
                    }
                }
                Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
                other => return other,
            }

            // Reset capture stack for loop repeat
            self.runtime.stack.truncate(self.frame.capture_start - 1); // Truncate stack including $0
            self.runtime.stack.push(Capture::Empty); // re-initialize $0
            self.frame.reader_start = self.runtime.reader.tell(); // Set reader start to current position

            first = false;
        }

        // End
        match Op::execute(&self.parselet.end, self) {
            Ok(Accept::Next) | Err(Reject::Next) => {}
            Ok(Accept::Repeat(value)) => {
                if let Some(value) = value {
                    retlist.push(value);
                }
            }
            Ok(accept) => return Ok(accept.into_push(self.parselet.severity)),
            other => return other,
        }

        // Take result from retlist
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
            // Otherwise, either take $0 if it was set to a value,
            // or use any last remaining value as result.
            let dollar0 = &mut self.runtime.stack[capture_start - 1];

            if let Capture::Value(value, ..) = dollar0 {
                Ok(Accept::Push(Capture::Value(
                    value.clone(),
                    None,
                    self.parselet.severity,
                )))
            } else if self.runtime.stack.len() > capture_start {
                Ok(Accept::Push(self.runtime.stack.pop().unwrap())
                    .into_push(self.parselet.severity))
            } else {
                Ok(Accept::Push(Capture::Empty))
            }
        }
    }
}

impl<'program, 'parselet, 'runtime> Drop for Context<'program, 'parselet, 'runtime> {
    fn drop(&mut self) {
        self.runtime.stack.truncate(self.stack_start + self.hold);
    }
}
