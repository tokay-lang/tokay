//! Tokay compiler

use super::*;
use crate::error::Error;
use crate::reader::*;
use crate::value;
use crate::value::RefValue;
use crate::vm::*;
use indexmap::{indexset, IndexSet};
use std::cell::RefCell;

/** Tokay compiler instance

A tokay compiler initializes a Tokay parser for later re-use when called multiple times.

The compiler works in a mode so that statics, variables and constants once built
won't be removed and can be accessed on later calls.
*/
pub struct Compiler {
    parser: Option<parser::Parser>, // Internal Tokay parser
    pub debug: u8,                  // Compiler debug mode
    pub(super) restrict: bool,      // Restrict assignment of reserved identifiers
    pub(super) statics: RefCell<IndexSet<RefValue>>, // Static values collected during compilation
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
        let statics = indexset![
            value!(void),
            value!(null),
            value!(true),
            value!(false),
            value!(0),
            value!(1),
        ];

        let mut compiler = Self {
            parser: None,
            debug: 0,
            restrict: true,
            statics: RefCell::new(statics),
        };

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
        let main_parselet = ImlParselet::new(ImlParseletInstance::new(
            None,
            None,
            None,
            Some("__main__".to_string()),
            5,
            false,
        ));

        let global_scope = Scope::new(self, ScopeLevel::Parselet(main_parselet.clone()), None);

        ast::traverse(&global_scope, &ast);

        for usage in global_scope.usages.borrow_mut().drain(..) {
            if let ImlValue::Unresolved(usage) = usage {
                let usage = usage.borrow();
                if let ImlValue::Name { offset, name } = &*usage {
                    global_scope.error(offset.clone(), format!("Use of undefined name '{}'", name));
                }
            }
        }

        if !global_scope.errors.borrow().is_empty() {
            return Err(global_scope.errors.borrow_mut().drain(..).collect());
        }

        /*
        if self.debug > 1 {
            println!("--- Global scope ---\n{:#?}", scope)
        }
        */

        if self.debug > 1 {
            println!("--- Intermediate main ---\n{:#?}", main_parselet);
        }

        let mut program = ImlProgram::new(ImlValue::from(main_parselet));
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
    pub(super) fn register_static(&self, value: RefValue) -> ImlValue {
        let mut statics = self.statics.borrow_mut();

        if let Some(value) = statics.get(&value) {
            ImlValue::Value(value.clone())
        } else {
            statics.insert(value.clone());
            ImlValue::Value(value)
        }
    }

    /*
    /// Push a parselet scope
    pub(super) fn parselet_push(
        &mut self,
        name: Option<String>,
        offset: Option<Offset>,
        generics: Option<ImlValueLookup>,
        signature: Option<IndexMap<String, ImlValue>>,
    ) {
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

        {
            let mut main = instance.model.borrow_mut();

            main.body = body;

            if self.scopes.is_empty() {
                // Rebuild __main__ scope
                self.parselet_push(Some("__main__".to_string()), None, None, None);

                self.scopes[0].constants = scope.constants;
                self.scopes[0].usages = scope.usages;

                let new_instance = self.scopes[0].instance.as_ref().unwrap();
                let mut new_main = new_instance.model.borrow_mut();

                new_main.locals = main.locals;
                new_main.variables = main.variables.clone();
                new_main.temporaries = main.temporaries.clone();
            } else {
                self.scopes[0].usages.extend(scope.usages);
            }
        }

        ImlValue::from(instance)
    }
    */
}
