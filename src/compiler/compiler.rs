//! Tokay compiler interface

use super::*;
use crate::builtin::Builtin;
use crate::error::Error;
use crate::reader::*;
use crate::value::{RefValue, Token};
use crate::vm::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Parselets introduce new variable scopes.
Loops introduce a new loop scope.
*/
#[derive(Debug)]
pub(in crate::compiler) enum Scope {
    Parselet {
        // parselet-level scope (variables and constants can be defined here)
        usage_start: usize, // Begin of usages to resolve until when scope is closed
        constants: HashMap<String, ImlValue>, // Constants symbol table
        variables: HashMap<String, usize>, // Variable symbol table
        begin: Vec<ImlOp>,  // Begin operations
        end: Vec<ImlOp>,    // End operations
        is_consuming: bool, // Determines whether the scope is consuming input for early consumable detection
    },
    Block {
        // block level (constants can be defined here)
        usage_start: usize, // Begin of usages to resolve until when scope is closed
        constants: HashMap<String, ImlValue>, // Constants symbol table
    },
    Loop, // loop level (allows use of break & continue)
}

/** Tokay compiler instance

A tokay compiler initializes a Tokay parser for later re-use when called multiple times.

The compiler works in a mode so that statics, variables and constants once built
won't be removed and can be accessed on later calls.
*/
pub struct Compiler {
    parser: Option<parser::Parser>, // Internal Tokay parser
    pub debug: u8,                  // Compiler debug mode

    pub(in crate::compiler) scopes: Vec<Scope>, // Current compilation scopes
    pub(in crate::compiler) usages: Vec<ImlOp>, // Unresolved calls or loads
    pub(in crate::compiler) errors: Vec<Error>, // Collected errors during compilation
}

impl Compiler {
    /** Initialize a new compiler.

    The compiler struct serves as some kind of helper that should be used during traversal of a
    Tokay program's AST. It therefore offers functions to open particular blocks and handle symbols
    in different levels. Parselets are created by using the parselet_pop() function with provided
    parameters.

    By default, the prelude should be loaded, otherwise several standard parselets are not available.
    Ignoring the prelude is only useful on bootstrap currently.
    */
    pub fn new(with_prelude: bool) -> Self {
        let mut compiler = Self {
            parser: None,
            debug: 0,
            scopes: Vec::new(),
            usages: Vec::new(),
            errors: Vec::new(),
        };

        // Compile with the default prelude
        if with_prelude {
            compiler
                .compile_from_str(include_str!("../prelude.tok"))
                .unwrap(); // this should panic in case of an error!
        }

        // Set compiler debug level afterwards
        compiler.debug = if let Ok(level) = std::env::var("TOKAY_DEBUG") {
            level.parse::<u8>().unwrap_or_default()
        } else {
            0
        };

        compiler
    }

    /** Compile a Tokay program from a Reader source into the compiler. */
    pub fn compile(&mut self, reader: Reader) -> Result<Option<Program>, Vec<Error>> {
        // Create the Tokay parser when not already done
        if self.parser.is_none() {
            self.parser = Some(Parser::new());
        }

        let parser = self.parser.as_ref().unwrap();
        let ast = match parser.parse(reader) {
            Ok(ast) => ast,
            Err(error) => {
                return Err(vec![error]);
            }
        };

        if self.debug > 0 {
            ast::print(&ast);
        }

        let ret = ast::traverse(self, &ast);

        if !self.errors.is_empty() {
            return Err(self.errors.drain(..).collect());
        }

        if self.debug > 1 {
            assert!(self.scopes.len() == 1);
            println!("--- Global scope ---\n{:#?}", self.scopes.last().unwrap())
        }

        if let ImlOp::Call {
            target: ImlTarget::Static(main),
            ..
        } = ret
        {
            if self.debug > 1 {
                println!("--- Intermediate main ---\n{:#?}", main);
            }

            match Linker::new(main).finalize() {
                Ok(program) => {
                    if self.debug > 1 {
                        println!("--- Finalized program ---");
                        program.dump();
                    }

                    Ok(Some(program))
                }
                Err(errors) => Err(errors),
            }
        } else {
            Ok(None)
        }
    }

    /// Shortcut to compile a Tokay program from a &str into the compiler.
    pub fn compile_from_str(&mut self, src: &str) -> Result<Option<Program>, Vec<Error>> {
        self.compile(Reader::new(Box::new(std::io::Cursor::new(src.to_owned()))))
    }

    /// Tries to resolves open usages from the current scope
    pub(in crate::compiler) fn resolve(&mut self) {
        if let Scope::Parselet { usage_start, .. } | Scope::Block { usage_start, .. } =
            &self.scopes[0]
        {
            // Cut out usages created inside this scope for processing
            let usages: Vec<ImlOp> = self.usages.drain(usage_start..).collect();

            // Afterwards, resolve and insert them again in case there where not resolved
            for mut op in usages.into_iter() {
                if op.resolve(self) {
                    continue;
                }

                self.usages.push(op); // Insert again for later resolve
            }
        }
    }

    /// Push a parselet scope
    pub(in crate::compiler) fn parselet_push(&mut self) {
        self.scopes.insert(
            0,
            Scope::Parselet {
                usage_start: self.usages.len(),
                variables: HashMap::new(),
                constants: HashMap::new(),
                begin: Vec::new(),
                end: Vec::new(),
                is_consuming: false,
            },
        )
    }

    /// Push a block scope
    pub(in crate::compiler) fn block_push(&mut self) {
        self.scopes.insert(
            0,
            Scope::Block {
                usage_start: self.usages.len(),
                constants: HashMap::new(),
            },
        )
    }

    /// Push a loop scope
    pub(in crate::compiler) fn loop_push(&mut self) {
        self.scopes.insert(0, Scope::Loop);
    }

    /// Resolves and drops a parselet scope and creates a new parselet from it.
    pub(in crate::compiler) fn parselet_pop(
        &mut self,
        offset: Option<Offset>,
        name: Option<String>,
        severity: Option<u8>,
        gen: Option<Vec<(String, Option<ImlValue>)>>,
        sig: Option<Vec<(String, Option<ImlValue>)>>,
        body: ImlOp,
    ) -> ImlValue {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Parselet { .. }));

        self.resolve();

        // Clear any unresolved usages when reaching global scope
        if self.scopes.len() == 1 {
            self.usages.clear();
        }

        let mut scope = self.scopes.remove(0);

        if let Scope::Parselet {
            variables,
            begin,
            end,
            is_consuming,
            ..
        } = &mut scope
        {
            fn ensure_block(ops: Vec<ImlOp>) -> ImlOp {
                match ops.len() {
                    0 => ImlOp::Nop,
                    1 => ops.into_iter().next().unwrap(),
                    _ => ImlOp::Alt { alts: ops },
                }
            }

            let signature = sig.unwrap_or(Vec::new());
            let constants = gen.unwrap_or(Vec::new());

            assert!(
                signature.len() <= variables.len(),
                "signature may not be longer than locals..."
            );

            let begin = ensure_block(begin.drain(..).collect());
            let end = ensure_block(end.drain(..).collect());

            //println!("begin = {:?}", begin);
            //println!("body  = {:?}", body);
            //println!("end   = {:?}", end);

            let parselet = ImlParselet {
                offset,
                name,
                consuming: *is_consuming
                    || begin.is_consuming()
                    || end.is_consuming()
                    || body.is_consuming(),
                severity: severity.unwrap_or(5), // severity
                constants,                       // constants
                signature,                       // signature
                locals: variables.len(),
                // Ensure that begin and end are blocks.
                begin,
                end,
                body,
            };

            if self.scopes.len() == 0 {
                //*consuming = false;
                self.scopes.push(scope);
            }

            ImlValue::Parselet(Rc::new(RefCell::new(parselet)))
        } else {
            unreachable!();
        }
    }

    /// Drops a block scope.
    pub(in crate::compiler) fn block_pop(&mut self) {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Block { .. }));
        self.resolve();
        self.scopes.remove(0);
    }

    /// Drops a loop scope.
    pub(in crate::compiler) fn loop_pop(&mut self) {
        assert!(self.scopes.len() > 0 && matches!(self.scopes[0], Scope::Loop));
        self.scopes.remove(0);
    }

    /// Marks the nearest parselet scope as consuming
    pub(in crate::compiler) fn parselet_mark_consuming(&mut self) {
        for scope in &mut self.scopes {
            if let Scope::Parselet { is_consuming, .. } = scope {
                *is_consuming = true;
                return;
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /// Check if there's a loop
    pub(in crate::compiler) fn loop_check(&mut self) -> bool {
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
    pub(in crate::compiler) fn get_local(&self, name: &str) -> Option<usize> {
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
    pub(in crate::compiler) fn new_local(&mut self, name: &str) -> usize {
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
    pub(in crate::compiler) fn get_global(&self, name: &str) -> Option<usize> {
        if let Scope::Parselet { variables, .. } = self.scopes.last().unwrap() {
            if let Some(addr) = variables.get(name) {
                return Some(*addr);
            }

            return None;
        }

        unreachable!("Top-level scope is not a parselet scope");
    }

    /** Set constant to name in current scope. */
    pub(in crate::compiler) fn set_constant(&mut self, name: &str, mut value: ImlValue) {
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
            self.parselet_push();
            self.parselet_mark_consuming();
            value = self.parselet_pop(
                None,
                Some("__".to_string()),
                Some(0), // Zero severity
                None,
                None,
                // becomes `Value+`
                ImlOp::call(None, value, None).into_positive(),
            );

            // Remind "__" as new constant
            secondary = Some(("__", value.clone()));

            // ...and then in-place "_" is defined as `_ : __?`
            self.parselet_push();
            self.parselet_mark_consuming();
            value = self.parselet_pop(
                None,
                Some(name.to_string()),
                Some(0), // Zero severity
                None,
                None,
                // becomes `Value?`
                ImlOp::call(None, value, None).into_optional(),
            );

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
    pub(in crate::compiler) fn get_constant(&mut self, name: &str) -> Option<ImlValue> {
        // Check for constant in available scopes
        for scope in &self.scopes {
            if let Scope::Parselet { constants, .. } | Scope::Block { constants, .. } = scope {
                if let Some(value) = constants.get(name) {
                    return Some(value.clone());
                }
            }
        }

        // When not found, check for a builtin function
        if let Some(builtin) = Builtin::get(name) {
            return Some(RefValue::from(builtin).into()); // fixme: Makes a Value into a RefValue into a Value...
        }

        // Builtin constants are defined on demand as fallback
        if name == "_" || name == "__" {
            // Fallback for "_" defines parselet `_ : Whitespace?`
            self.set_constant(
                "_",
                RefValue::from(Token::builtin("Whitespaces").unwrap()).into(),
            );

            return Some(self.get_constant(name).unwrap());
        }

        // Check for built-in token
        if let Some(value) = Token::builtin(name) {
            return Some(RefValue::from(value).into());
        }

        None
    }
}
