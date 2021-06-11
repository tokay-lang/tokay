//! Tokay compiler interface

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;

use super::*;
use crate::builtin;
use crate::error::Error;
use crate::reader::Reader;
use crate::token::Token;
use crate::value::{RefValue, Value};
use crate::vm::*;

/** Compiler symbolic scope.

In Tokay code, this relates to any block. Parselet blocks (parselets) introduce new variable scopes.
*/
#[derive(Debug)]
pub(crate) struct Scope {
    pub(super) variables: Option<HashMap<String, usize>>, // Variable symbol table; Determines whether a scope is a parselet-level scope or just block scope
    constants: HashMap<String, RefValue>,                 // Constants symbol table
    pub(super) begin: Vec<Op>, // Begin operations; Can only be set for parselet-scopes
    pub(super) end: Vec<Op>,   // End operations; Can only be set for parselet-scopes
    usage_start: usize,        // Begin of usages to resolve until when scope is closed
    pub(super) consuming: bool, // Determines whether the scope is consuming input for early consumable detection
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
        let mut compiler = Self {
            parser: None,
            debug: false,
            interactive: false,
            statics: RefCell::new(Vec::new()),
            scopes: Vec::new(),
            usages: Vec::new(),
            errors: Vec::new(),
        };

        compiler.push_scope(true); // Main scope
        compiler
    }

    /** Compile a Tokay program from source into a Program struct.

    In case None is returend, causing errors where already reported to stdout. */
    pub fn compile(&mut self, reader: Reader) -> Option<Program> {
        // Push a main scope on if not present
        if self.scopes.len() == 0 {
            self.push_scope(true); // Main scope
        }

        // Create the Tokay parser when not already done
        if self.parser.is_none() {
            self.parser = Some(Parser::new());
        }

        let ast = match self.parser.as_ref().unwrap().parse(reader) {
            Ok(ast) => ast,
            Err(error) => {
                println!("{}", error);
                return None;
            }
        };

        if self.debug {
            ast::print(&ast);
        }

        ast::traverse(self, &ast);

        let program = match self.to_program() {
            Ok(program) => program,
            Err(errors) => {
                for error in errors {
                    println!("{}", error);
                }

                return None;
            }
        };

        if self.debug {
            program.dump();
        }

        Some(program)
    }

    /// Shortcut to compile a Tokay program from a &str.
    pub fn compile_str(&mut self, src: &'static str) -> Option<Program> {
        self.compile(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    /** Converts the compiled information into a Program. */
    pub(crate) fn to_program(&mut self) -> Result<Program, Vec<Error>> {
        let mut errors: Vec<Error> = self.errors.drain(..).collect();

        // Close all scopes except main
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        // Either resolve or pop global scope
        if self.interactive {
            self.resolve_scope();
        } else {
            self.pop_scope();
        }

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

    /// Introduces a new scope, either for variables or constants only.
    pub(crate) fn push_scope(&mut self, has_variables: bool) {
        self.scopes.insert(
            0,
            Scope {
                variables: if has_variables {
                    Some(HashMap::new())
                } else {
                    None
                },
                constants: HashMap::new(),
                begin: Vec::new(),
                end: Vec::new(),
                usage_start: self.usages.len(),
                consuming: false,
            },
        );
    }

    /// Resolves usages from current scope
    pub(super) fn resolve_scope(&mut self) {
        // Cut out usages created inside this scope for processing
        let usages: Vec<Result<Vec<Op>, Usage>> =
            self.usages.drain(self.scopes[0].usage_start..).collect();

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

    /// Resolves and pops a scope.
    pub(super) fn pop_scope(&mut self) -> Scope {
        if self.scopes.len() == 0 {
            panic!("No more scopes to pop!");
        }

        self.resolve_scope();

        // Now scope can be removed
        let scope = self.scopes.remove(0);

        // Inherit consumable attribute to upper scope when this is not a variable-holding scope
        if scope.consuming && self.scopes.len() > 0 && self.scopes[0].variables.is_none() {
            self.scopes[0].consuming = true;
        }

        scope
    }

    /// Resolves and pops a scope and creates a new parselet from it
    pub(crate) fn create_parselet(
        &mut self,
        name: Option<String>,
        sig: Vec<(String, Option<usize>)>,
        body: Op,
        consuming: Option<bool>,
        silent: bool,
        main: bool,
    ) -> Parselet {
        if main {
            assert!(
                self.scopes[0].variables.is_some(),
                "Main scope must be a parselet-level scope."
            );

            Parselet::new(
                name,
                sig,
                self.scopes[0]
                    .variables
                    .as_ref()
                    .map_or(0, |vars| vars.len()),
                consuming.unwrap_or(self.scopes[0].consuming),
                silent,
                Op::from_vec(self.scopes[0].begin.drain(..).collect()),
                Op::from_vec(self.scopes[0].end.drain(..).collect()),
                body,
            )
        } else {
            loop {
                let scope = self.pop_scope();
                if scope.variables.is_some() {
                    break Parselet::new(
                        name,
                        sig,
                        scope.variables.map_or(0, |vars| vars.len()),
                        consuming.unwrap_or(scope.consuming),
                        silent,
                        Op::from_vec(scope.begin),
                        Op::from_vec(scope.end),
                        body,
                    );
                }
            }
        }
    }

    /** Retrieves the address of a local variable under a given name.

    Returns None when the variable does not exist. */
    pub(crate) fn get_local(&self, name: &str) -> Option<usize> {
        // Retrieve local variables from next scope owning variables, except global scope!
        for scope in &self.scopes[..self.scopes.len() - 1] {
            // Check for scope with variables
            if let Some(variables) = &scope.variables {
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
            if let Some(variables) = &mut scope.variables {
                if let Some(addr) = variables.get(name) {
                    return *addr;
                }

                let addr = variables.len();
                variables.insert(name.to_string(), addr);
                return addr;
            }
        }

        unreachable!("This should not be possible")
    }

    /** Retrieve address of a global variable. */
    pub(crate) fn get_global(&self, name: &str) -> Option<usize> {
        let variables = self.scopes.last().unwrap().variables.as_ref().unwrap();

        if let Some(addr) = variables.get(name) {
            Some(*addr)
        } else {
            None
        }
    }

    /** Set constant to name in current scope. */
    pub(crate) fn set_constant(&mut self, name: &str, value: RefValue) {
        self.scopes
            .first_mut()
            .unwrap()
            .constants
            .insert(name.to_string(), value);
    }

    /** Get constant value, either from current or preceding scope,
    a builtin or special. */
    pub(crate) fn get_constant(&self, name: &str) -> Option<RefValue> {
        for scope in &self.scopes {
            if let Some(value) = scope.constants.get(name) {
                return Some(value.clone());
            }
        }

        // When not found, check for a builtin
        if let Some(builtin) = builtin::get(name) {
            return Some(Value::Builtin(builtin).into_refvalue());
        }

        // Special tokens
        match name {
            "Void" => Some(Token::Void.into_value().into_refvalue()),
            "Any" => Some(Token::Any.into_value().into_refvalue()),
            "EOF" => Some(Token::EOF.into_value().into_refvalue()),
            _ => None,
        }
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
