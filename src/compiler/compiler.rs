//! Tokay compiler interface

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;

use super::*;
use crate::builtin;
use crate::error::Error;
use crate::reader::Reader;
use crate::token;
use crate::value::{RefValue, Value};
use crate::vm::*;

/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Parselets introduce new variable scopes.
Loops introduce a new loop scope.
*/
#[derive(Debug)]
pub(crate) enum Scope {
    Parselet {
        // parselet-level scope (variables and constants can be defined here)
        usage_start: usize, // Begin of usages to resolve until when scope is closed
        constants: HashMap<String, RefValue>, // Constants symbol table
        variables: HashMap<String, usize>, // Variable symbol table
        begin: Vec<Op>,     // Begin operations
        end: Vec<Op>,       // End operations
        consuming: bool, // Determines whether the scope is consuming input for early consumable detection
    },
    Block {
        // block level (constants can be defined here)
        usage_start: usize, // Begin of usages to resolve until when scope is closed
        constants: HashMap<String, RefValue>, // Constants symbol table
    },
    Loop, // loop level (allows use of break & continue)
}

/** Tokay compiler instance

A tokay compiler initializes a Tokay parser for later re-use when called multiple times.

The compiler can be set into an interactive mode so that statics, variables and constants once built
won't be removed and can be accessed later on. This is useful in REPL mode.
*/
pub struct Compiler {
    parser: Option<parser::Parser>, //Tokay parser
    pub debug: bool,
    pub interactive: bool,
    pub(super) statics: RefCell<Vec<RefValue>>, // Static values and parselets collected during compile
    pub(super) scopes: Vec<Scope>,              // Current compilation scopes
    pub(super) usages: Vec<Result<Vec<Op>, Usage>>, // Usages of symbols in parselets
    pub(super) errors: Vec<Error>,              // Collected errors during compilation
}

impl Compiler {
    pub fn new() -> Self {
        // Initialize new compiler.
        Self {
            parser: None,
            debug: false,
            interactive: false,
            statics: RefCell::new(Vec::new()),
            scopes: Vec::new(),
            usages: Vec::new(),
            errors: Vec::new(),
        }
    }

    /** Compile a Tokay program from source into a Program struct.

    In case None is returend, causing errors where already reported to stdout. */
    pub fn compile(&mut self, reader: Reader) -> Result<Program, Vec<Error>> {
        // Create the Tokay parser when not already done
        if self.parser.is_none() {
            self.parser = Some(Parser::new());
        }

        let ast = match self.parser.as_ref().unwrap().parse(reader) {
            Ok(ast) => ast,
            Err(error) => {
                eprintln!("{}", error);
                return Err(vec![error]);
            }
        };

        if self.debug {
            ast::print(&ast);
        }

        ast::traverse(self, &ast);

        let program = match self.to_program() {
            Ok(program) => program,
            Err(errors) => {
                for error in &errors {
                    eprintln!("{}", error);
                }

                return Err(errors);
            }
        };

        if self.debug {
            program.dump();
        }

        Ok(program)
    }

    /// Shortcut to compile a Tokay program from a &str.
    pub fn compile_str(&mut self, src: &'static str) -> Result<Program, Vec<Error>> {
        self.compile(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    /** Converts the compiled information into a Program. */
    pub(crate) fn to_program(&mut self) -> Result<Program, Vec<Error>> {
        let mut errors: Vec<Error> = self.errors.drain(..).collect();

        // Close all scopes except main
        assert!(self.scopes.len() == 0 || (self.scopes.len() == 1 && self.interactive));

        let statics: Vec<RefValue> = if self.interactive {
            self.statics.borrow().clone()
        } else {
            self.statics.borrow_mut().drain(..).collect()
        };

        let usages = self
            .usages
            .drain(..)
            .map(|usage| {
                match usage {
                    Ok(usage) => usage,
                    Err(usage) => {
                        let error = match usage {
                            Usage::Load { name, offset } | Usage::CallOrCopy { name, offset } => {
                                Error::new(offset, format!("Use of unresolved symbol '{}'", name))
                            }

                            Usage::Call {
                                name,
                                args: _,
                                nargs: _,
                                offset,
                            } => {
                                Error::new(offset, format!("Call to unresolved symbol '{}'", name))
                            }

                            Usage::Error(error) => error,
                        };

                        errors.push(error);
                        vec![Op::Nop] // Dummy instruction
                    }
                }
            })
            .collect();

        Parselet::finalize(usages, &statics);

        // Stop when any unresolved usages occured;
        // We do this here so that eventual undefined symbols are replaced by Op::Nop,
        // and later don't throw other errors especially when in interactive mode.
        if errors.len() > 0 {
            return Err(errors);
        }

        // Make program from statics
        Ok(Program::new(statics))
    }

    /// Resolves usages from current scope
    pub(super) fn resolve(&mut self) {
        if let Scope::Parselet { usage_start, .. } | Scope::Block { usage_start, .. } =
            &self.scopes[0]
        {
            // Cut out usages created inside this scope for processing
            let usages: Vec<Result<Vec<Op>, Usage>> = self.usages.drain(usage_start..).collect();

            // Afterwards, resolve and insert them again
            for usage in usages.into_iter() {
                match usage {
                    Err(mut usage) => {
                        if let Some(res) = usage.try_resolve(self) {
                            self.usages.push(Ok(res))
                        } else {
                            self.usages.push(Err(usage))
                        }
                    }
                    Ok(res) => self.usages.push(Ok(res)),
                }
            }
        }
    }

    /// Push a parselet scope
    pub(crate) fn push_parselet(&mut self) {
        self.scopes.insert(
            0,
            Scope::Parselet {
                usage_start: self.usages.len(),
                variables: HashMap::new(),
                constants: HashMap::new(),
                begin: Vec::new(),
                end: Vec::new(),
                consuming: false,
            },
        )
    }

    /// Push a block scope
    pub(crate) fn push_block(&mut self) {
        self.scopes.insert(
            0,
            Scope::Block {
                usage_start: self.usages.len(),
                constants: HashMap::new(),
            },
        )
    }

    /// Push a loop scope
    pub(crate) fn push_loop(&mut self) {
        self.scopes.insert(0, Scope::Loop);
    }

    /// Resolves and drops a parselet scope and creates a new parselet from it.
    pub(crate) fn pop_parselet(
        &mut self,
        name: Option<String>,
        sig: Vec<(String, Option<usize>)>,
        body: Op,
    ) -> Parselet {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Parselet { .. }));

        self.resolve();
        let mut scope = self.scopes.remove(0);

        if let Scope::Parselet {
            variables,
            begin,
            end,
            consuming,
            ..
        } = &mut scope
        {
            fn ensure_block(ops: Vec<Op>) -> Op {
                match ops.len() {
                    0 => Op::Nop,
                    1 => ops.into_iter().next().unwrap(),
                    _ => Block::new(ops).into_op(),
                }
            }

            let mut parselet = Parselet::new(
                name,
                sig,
                variables.len(),
                // Ensure that begin and end are blocks.
                ensure_block(begin.drain(..).collect()),
                ensure_block(end.drain(..).collect()),
                body,
            );

            parselet.consuming = *consuming;

            if self.scopes.len() == 0 && self.interactive {
                self.scopes.push(scope);
            }

            parselet
        } else {
            unreachable!();
        }
    }

    /// Drops a block scope.
    pub(crate) fn pop_block(&mut self) {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Block { .. }));
        self.resolve();
        self.scopes.remove(0);
    }

    /// Drops a loop scope.
    pub(crate) fn pop_loop(&mut self) {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Loop));
        self.scopes.remove(0);
    }

    /// Marks the nearest parselet scope as consuming
    pub(crate) fn mark_consuming(&mut self) {
        for scope in &mut self.scopes {
            if let Scope::Parselet { consuming, .. } = scope {
                *consuming = true;
                return;
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /// Check if there's a loop
    pub(crate) fn check_loop(&mut self) -> bool {
        for i in 0..self.scopes.len() {
            match &self.scopes[i] {
                Scope::Parselet { .. } => return false,
                Scope::Loop => return true,
                _ => {}
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Retrieves the address of a local variable under a given name.

    Returns None when the variable does not exist. */
    pub(crate) fn get_local(&self, name: &str) -> Option<usize> {
        // Retrieve local variables from next parselet scope owning variables, except global scope!
        for scope in &self.scopes[..self.scopes.len() - 1] {
            // Check for scope with variables
            if let Scope::Parselet { variables, .. } = &scope {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr);
                }

                break;
            }
        }

        None
    }

    /** Insert new local variable under given name in current scope. */
    pub(crate) fn new_local(&mut self, name: &str) -> usize {
        for scope in &mut self.scopes {
            // Check for scope with variables
            if let Scope::Parselet { variables, .. } = scope {
                if let Some(addr) = variables.get(name) {
                    return *addr;
                }

                let addr = variables.len();
                variables.insert(name.to_string(), addr);
                return addr;
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Retrieve address of a global variable. */
    pub(crate) fn get_global(&self, name: &str) -> Option<usize> {
        if let Scope::Parselet { variables, .. } = self.scopes.last().unwrap() {
            if let Some(addr) = variables.get(name) {
                return Some(*addr);
            }

            return None;
        }

        unreachable!("Top-level scope is not a parselet scope");
    }

    /** Set constant to name in current scope. */
    pub(crate) fn set_constant(&mut self, name: &str, mut value: RefValue) {
        /*
            Special meaning for whitespace constants names "_" and "__".

            When set, the corresponding consumable Value becomes the following:

            - `__ : Value+`
            - `_ : __?`

            This is always the case whenever "_" or "__" is set.
            Fallback defaults to `Value : Whitespace`, handled in get_constant().
        */
        let mut secondary = None;

        if name == "_" || name == "__" {
            // First of all, "__" is defined as `__ : Value+`...
            let mut parselet = Parselet::new(
                Some("__".to_string()),
                Vec::new(),
                0,
                Op::Nop,
                Op::Nop,
                // becomes silent `Value+`
                Repeat::new(Op::CallStatic(self.define_static(value)), 1, 0, true).into_op(),
            );

            parselet.consuming = true;
            parselet.silent = true;

            value = parselet.into_value().into_refvalue();

            // Insert "__" as new constant
            secondary = Some(("__", value.clone()));

            // ...and then in-place "_" is defined as `_ : __?`
            let mut parselet = Parselet::new(
                Some(name.to_string()),
                Vec::new(),
                0,
                Op::Nop,
                Op::Nop,
                // becomes silent `Value?`
                Repeat::new(Op::CallStatic(self.define_static(value)), 0, 1, true).into_op(),
            );

            parselet.consuming = true;
            parselet.silent = true;

            value = parselet.into_value().into_refvalue();

            // Insert "_" afterwards
        }

        // Insert constant into next constant-holding scope
        for scope in &mut self.scopes {
            if let Scope::Parselet { constants, .. } | Scope::Block { constants, .. } = scope {
                if let Some((name, value)) = secondary {
                    constants.insert(name.to_string(), value);
                }

                constants.insert(name.to_string(), value);
                return;
            }
        }

        unreachable!("There _must_ be at least one parselet or block scope!");
    }

    /** Get constant value, either from current or preceding scope,
    a builtin or special. */
    pub(crate) fn get_constant(&mut self, name: &str) -> Option<RefValue> {
        // Check for constant in available scopes
        for scope in &self.scopes {
            if let Scope::Parselet { constants, .. } | Scope::Block { constants, .. } = scope {
                if let Some(value) = constants.get(name) {
                    return Some(value.clone());
                }
            }
        }

        // When not found, check for a builtin function
        if let Some(builtin) = builtin::get(name) {
            return Some(Value::Builtin(builtin).into_refvalue());
        }

        // Builtin constants are defined on demand as fallback
        if name == "_" || name == "__" {
            // Fallback for "_" defines parselet `_ : Whitespace?`
            self.set_constant(
                "_",
                Value::Builtin(builtin::get("Whitespaces").unwrap()).into_refvalue(),
            );

            return Some(self.get_constant(name).unwrap());
        }

        // Check for built-in token
        if let Some(value) = token::get(name) {
            return Some(value.into_value().into_refvalue());
        }

        None
    }

    /** Defines a new static value inside the program.
    Statics are only inserted once when they already exist. */
    pub(crate) fn define_static(&self, value: RefValue) -> usize {
        let mut statics = self.statics.borrow_mut();

        // Check if there exists already a static equivalent to new_value
        // fixme: A HashTab might be more faster here...
        {
            let value = value.borrow();

            for (i, known) in statics.iter().enumerate() {
                if *known.borrow() == *value {
                    return i; // Reuse existing value address
                }
            }
        }

        // Save value as new
        statics.push(value);
        statics.len() - 1
    }
}
