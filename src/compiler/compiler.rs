//! Tokay compiler

use super::*;
use crate::builtin::Builtin;
use crate::error::Error;
use crate::reader::*;
use crate::value;
use crate::value::{RefValue, Token};
use crate::vm::*;
use indexmap::{IndexMap, IndexSet};

pub(super) type ImlValueLookup = IndexMap<String, ImlValue>;

/** Compiler symbolic scopes.

In Tokay code, this relates to any block.
Parselets introduce new variable scopes.
Loops introduce a new loop scope.
*/
#[derive(Debug, PartialEq)]
pub(super) enum ScopeLevel {
    Parselet, // parselet-level scope (variables and constants can be defined here)
    Block,    // block level (constants can be defined here)
    Loop,     // loop level (allows the use of break & continue)
}

#[derive(Debug)]
pub(super) struct Scope {
    //compiler: &'compiler mut Compiler,  // reference to compiler
    level: ScopeLevel, // Scope level
    is_global: bool,   // Globa flag
    //next: Option<Box<Scope>>,  // Previous scope

    // Parselet- and block-level only
    usage_start: usize,
    //usages: Vec<ImlValue>, // Unresolved usages within a block scope
    constants: ImlValueLookup, // Symbol table of named constants

    // Parselet-level only
    variables: IndexMap<String, usize>, // Symbol table of variables and their addresses
    temporaries: Vec<usize>,            // Table of (unused) temporary addresses for later-reuse
    pub(super) instance: Option<ImlParseletInstance>, // Currently constructed parselet instance
}

impl Scope {
    pub fn new(level: ScopeLevel, compiler: &Compiler) -> Self {
        Self {
            level,
            is_global: compiler.scopes.is_empty(),
            //usages: Vec::new(),
            usage_start: compiler.usages.len(),
            constants: ImlValueLookup::new(),
            variables: IndexMap::new(),
            temporaries: Vec::new(),
            instance: None,
        }
    }
}

/*
impl 'compiler Scope<'compiler> {
    pub fn new<'compiler>(level: ScopeLevel, compiler: &'compiler mut Compiler), prev: Option<Scope>) -> Self {
        Self {
            compiler,
            level,
            next: prev,
            usages: Vec::new(),
            contants: ImlValueLookup::new(),
            variables: IndexMap::new(),
            temporaries: Vec::new(),
            instance:
        }

        compiler.scopes.insert(
            0,
            Self {
                usage_start: compiler.usages.len(),
                constants: IndexMap::new(),

                instance: if level == ScopeLevel::Parselet { Some(ImlParseletInstance::new(ImlParseletModel::new(signature), generics, offset, name, 5),
                variables: IndexMap::new(),

                temporaries: Vec::new(),
                is_global: self.scopes.is_empty(),
                level,
            },
    }

    pub fn new_parselet<'compiler>(compiler: &'compiler mut Compiler, generics: Option<ImlValueLookup>) -> &'compiler mut Self {


        );

        &mut compiler.scopes[0]
    }

    pub fn new_block<'compiler>(compiler: &'compiler mut Compiler) -> &'compiler mut Self {
        compiler.scopes.insert(
            0,
            Scope::Block {
                usage_start: compiler.usages.len(),
                constants: IndexMap::new(),
            },
        );

        &mut compiler.scopes[0]
    }

    pub fn new_loop<'compiler>(compiler: &'compiler mut Compiler) -> &'compiler mut Self {
        compiler.scopes.insert(
            0,
            Scope::Loop,
        );

        &mut compiler.scopes[0]
    }
}
*/

/** Tokay compiler instance

A tokay compiler initializes a Tokay parser for later re-use when called multiple times.

The compiler works in a mode so that statics, variables and constants once built
won't be removed and can be accessed on later calls.
*/
pub struct Compiler {
    parser: Option<parser::Parser>,         // Internal Tokay parser
    pub debug: u8,                          // Compiler debug mode
    pub(super) restrict: bool,              // Restrict assignment of reserved identifiers
    pub(super) statics: IndexSet<RefValue>, // Static values collected during compilation
    pub(super) scopes: Vec<Scope>,          // Current compilation scopes
    pub(super) usages: Vec<ImlValue>,       // Unresolved values
    pub(super) errors: Vec<Error>,          // Collected errors during compilation
}

impl Compiler {
    /** Initialize a new compiler.

    The compiler serves functions to compile Tokay source code into programs executable by
    the Tokay VM. It uses an intermediate language representation to implement derives of
    generics, statics, etc.

    The compiler struct serves as some kind of helper that should be used during traversal of a
    Tokay program's AST. It therefore offers functions to open particular blocks and handle symbols
    in different levels. Parselets are created by using the parselet_pop() function with provided
    parameters.
    */
    pub fn new() -> Self {
        let mut compiler = Self {
            parser: None,
            debug: 0,
            restrict: true,
            statics: IndexSet::new(),
            scopes: Vec::new(),
            usages: Vec::new(),
            errors: Vec::new(),
        };

        // Preload oftenly used static constants
        for value in [
            value!(void),
            value!(null),
            value!(true),
            value!(false),
            value!(0),
            value!(1),
        ] {
            compiler.statics.insert(value);
        }

        // Compile with the default prelude
        compiler.load_prelude();

        // Set compiler debug level afterwards
        compiler.debug = if let Ok(level) = std::env::var("TOKAY_DEBUG") {
            level.parse::<u8>().unwrap_or_default()
        } else {
            0
        };

        compiler
    }

    /** Compile a Tokay program from an existing AST into the compiler. */
    pub(super) fn compile_from_ast(
        &mut self,
        ast: &RefValue,
    ) -> Result<Option<Program>, Vec<Error>> {
        let ret = ast::traverse(self, &ast);

        assert!(self.scopes.len() == 1);

        for usage in self.usages.drain(..) {
            if let ImlValue::Unresolved(usage) = usage {
                let usage = usage.borrow();
                if let ImlValue::Name { offset, name } = &*usage {
                    self.errors.push(Error::new(
                        offset.clone(),
                        format!("Use of undefined name '{}'", name),
                    ));
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.drain(..).collect());
        }

        if self.debug > 1 {
            println!("--- Global scope ---\n{:#?}", self.scopes.last().unwrap())
        }

        if let ImlOp::Call { target: main, .. } = ret {
            if self.debug > 1 {
                println!("--- Intermediate main ---\n{:#?}", main);
            }

            let mut program = ImlProgram::new(main);
            program.debug = self.debug > 1;

            match program.compile() {
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
            println!("--- Abstract Syntax Tree ---");
            ast::print(&ast);
            //println!("###\n{:#?}\n###", ast);
        }

        self.compile_from_ast(&ast)
    }

    /// Shortcut to compile a Tokay program from a &str into the compiler.
    pub fn compile_from_str(&mut self, src: &str) -> Result<Option<Program>, Vec<Error>> {
        self.compile(Reader::new(
            None,
            Box::new(std::io::Cursor::new(src.to_owned())),
        ))
    }

    /** Register a static value within a compiler instance.

    This avoids that the compiler produces multiple results pointing to effectively the same values
    (althought they are different objects, but  the same value)
    */
    pub(super) fn register_static(&mut self, value: RefValue) -> ImlValue {
        if let Some(value) = self.statics.get(&value) {
            ImlValue::Value(value.clone())
        } else {
            self.statics.insert(value.clone());
            ImlValue::Value(value)
        }
    }

    /// Tries to resolves open usages from the current scope
    pub(super) fn resolve(&mut self) {
        // Cut out usages created inside this scope for processing
        let usages: Vec<ImlValue> = self.usages.drain(self.scopes[0].usage_start..).collect();

        // Afterwards, resolve and insert them again in case there where not resolved
        for mut value in usages.into_iter() {
            if value.resolve(self) {
                continue;
            }

            self.usages.push(value); // Re-insert into usages for later resolve
        }
    }

    /// Push a parselet scope
    pub(super) fn parselet_push(
        &mut self,
        name: Option<String>,
        offset: Option<Offset>,
        generics: Option<ImlValueLookup>,
        signature: Option<IndexMap<String, ImlValue>>,
    ) {
        // Generate variables from signature, addresses are enumerated!
        let variables = signature
            .as_ref()
            .unwrap_or(&IndexMap::new())
            .keys()
            .enumerate()
            .map(|(index, key)| (key.to_string(), index))
            .collect();

        let instance = ImlParseletInstance::new(
            Some(ImlParseletModel::new(signature)),
            generics,
            offset,
            name,
            5,
            false,
        );

        /*
        if self.debug > 1 {
            println!("PUSH {:#?}", instance);
        }
        */
        let mut scope = Scope::new(ScopeLevel::Parselet, self);
        scope.instance = Some(instance);
        scope.variables = variables;

        self.scopes.insert(0, scope)
    }

    /// Push a block scope
    pub(super) fn block_push(&mut self) {
        self.scopes.insert(0, Scope::new(ScopeLevel::Block, self))
    }

    /// Push a loop scope
    pub(super) fn loop_push(&mut self) {
        self.scopes.insert(0, Scope::new(ScopeLevel::Loop, self))
    }

    /// Resolves and drops a parselet scope and creates a new parselet from it.
    pub(super) fn parselet_pop(&mut self, body: ImlOp) -> ImlValue {
        assert!(!self.scopes.is_empty() && self.scopes[0].level == ScopeLevel::Parselet);
        self.resolve();

        let scope = self.scopes.remove(0);
        let instance = scope.instance.unwrap();

        instance.model.borrow_mut().body = body;

        if self.scopes.is_empty() {
            // Rebuild __main__ scope
            self.parselet_push(Some("__main__".to_string()), None, None, None);

            let new_instance = self.scopes[0].instance.as_ref().unwrap();
            new_instance.model.borrow_mut().locals = instance.model.borrow().locals;
            self.scopes[0].constants = scope.constants;
            self.scopes[0].variables = scope.variables;
            self.scopes[0].temporaries = scope.temporaries;
        }

        ImlValue::from(instance)
    }

    /// Drops a block scope.
    pub(super) fn block_pop(&mut self) {
        assert!(!self.scopes.is_empty() && self.scopes[0].level == ScopeLevel::Block);
        self.resolve();
        self.scopes.remove(0);
    }

    /// Drops a loop scope.
    pub(super) fn loop_pop(&mut self) {
        assert!(!self.scopes.is_empty() && self.scopes[0].level == ScopeLevel::Loop);
        self.scopes.remove(0);
    }

    /// Marks the nearest parselet scope as consuming
    pub(super) fn parselet_mark_consuming(&mut self) {
        for scope in &mut self.scopes {
            if scope.level == ScopeLevel::Parselet {
                scope
                    .instance
                    .as_ref()
                    .unwrap()
                    .model
                    .borrow_mut()
                    .is_consuming = true;
                return;
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Returns if the current parselet scope is global */
    pub(super) fn is_global(&self) -> bool {
        for scope in &self.scopes {
            // Check for scope with variables
            if scope.level == ScopeLevel::Parselet {
                return scope.is_global;
            }
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /// Check if there's a loop
    pub(super) fn loop_check(&mut self) -> bool {
        for scope in &self.scopes {
            match scope.level {
                ScopeLevel::Loop => return true,
                ScopeLevel::Parselet => break,
                _ => {}
            }
        }

        false
    }

    /** Insert or get local variable with given name in current parselet scope. */
    pub(super) fn local(&mut self, name: &str) {
        for scope in &mut self.scopes {
            if scope.level != ScopeLevel::Parselet {
                continue;
            }

            if scope.variables.get(name).is_some() {
                return;
            }

            scope.variables.insert(name.to_string(), {
                // todo: Make function in ImlParseletModel
                let mut model = scope.instance.as_ref().unwrap().model.borrow_mut();
                let addr = model.locals;
                model.locals += 1;
                addr
            });

            return;
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Claim unused or new temporary variable in current parselet scope. */
    pub(super) fn temp(&mut self) -> usize {
        for scope in &mut self.scopes {
            if scope.level != ScopeLevel::Parselet {
                continue;
            }

            if let Some(temp) = scope.temporaries.pop() {
                return temp;
            }

            // todo: Make function in ImlParseletModel
            let mut model = scope.instance.as_ref().unwrap().model.borrow_mut();
            let addr = model.locals;
            model.locals += 1;
            return addr;
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Return temporary variable after use for later re-use */
    pub(super) fn untemp(&mut self, temp: usize) {
        for scope in &mut self.scopes {
            if scope.level != ScopeLevel::Parselet {
                continue;
            }

            scope.temporaries.push(temp);
            return;
        }

        unreachable!("There _must_ be at least one parselet scope!");
    }

    /** Set constant to name in current scope. */
    pub(super) fn constant(&mut self, name: &str, mut value: ImlValue) {
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
            // `__` becomes `Value+`
            value = value.into_generic("Pos", Some(0), None).try_resolve(self);
            secondary = Some(("__", value.clone()));

            // ...and then in-place "_" is defined as `_ : __?`
            value = value.into_generic("Opt", Some(0), None).try_resolve(self);
        }

        // Insert constant into next constant-holding scope
        for scope in &mut self.scopes {
            if scope.level == ScopeLevel::Parselet || scope.level == ScopeLevel::Block {
                if let Some((name, value)) = secondary {
                    scope.constants.insert(name.to_string(), value);
                }

                scope.constants.insert(name.to_string(), value);
                return;
            }
        }

        unreachable!("There _must_ be at least one parselet or block scope!");
    }

    /** Get named value, either from current or preceding scope, a builtin or special. */
    pub(super) fn get(&mut self, offset: Option<Offset>, name: &str) -> Option<ImlValue> {
        let mut top_parselet = true;

        for scope in &self.scopes {
            if scope.level == ScopeLevel::Loop {
                continue;
            }

            // Check constants first
            if let Some(value) = scope.constants.get(name) {
                return Some(value.clone());
            }

            // Check generic
            if let Some(instance) = &scope.instance {
                if instance.generics.get(name).is_some() {
                    return Some(ImlValue::Generic {
                        offset,
                        name: name.to_string(),
                    });
                }

                // Check for variable only in first or global scope
                if scope.is_global || top_parselet {
                    if let Some(addr) = scope.variables.get(name) {
                        return Some(ImlValue::Variable {
                            offset,
                            name: name.to_string(),
                            is_global: scope.is_global,
                            addr: *addr,
                        });
                    }
                }

                top_parselet = false;
            }
        }

        self.get_builtin(name)
    }

    /** Get defined builtin. */
    pub(super) fn get_builtin(&mut self, name: &str) -> Option<ImlValue> {
        // Check for a builtin function
        if let Some(builtin) = Builtin::get(name) {
            return Some(RefValue::from(builtin).into()); // fixme: Makes a Value into a RefValue into a Value...
        }

        // Builtin constants are defined on demand as fallback
        if name == "_" || name == "__" {
            // Fallback for "_" defines parselet `_ : Whitespace?`
            self.constant(
                "_",
                RefValue::from(Token::builtin("Whitespaces").unwrap()).into(),
            );

            return Some(self.get(None, name).unwrap());
        }

        // Check for built-in token
        if let Some(value) = Token::builtin(name) {
            return Some(RefValue::from(value).into());
        }

        None
    }
}
