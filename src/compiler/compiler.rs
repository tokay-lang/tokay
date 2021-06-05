use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::BufReader;

use super::*;
use crate::builtin;
use crate::ccl::Ccl;
use crate::error::Error;
use crate::reader::{Offset, Reader};
use crate::token::Token;
use crate::utils;
use crate::value::{BorrowByIdx, BorrowByKey, Dict, RefValue, Value};
use crate::vm::*;

/** Traversal result.

This enum is used to allow either for a value or ops created during the AST traversal in the compiler.
*/
#[derive(Debug)]
enum TraversalResult {
    Empty,
    Value(RefValue),
    Identifier(String, Option<Offset>),
    Ops(Vec<Op>),
}

impl TraversalResult {
    /** Turns a traversal result into a vector of operations;

    In case the result is a Value, it can either be called when calling with 0 arguments is possible,
    which is specified by the call flag.
    */
    fn into_ops(self, compiler: &mut Compiler, call: bool) -> Vec<Op> {
        match self {
            TraversalResult::Empty => Vec::new(),
            TraversalResult::Value(value) => {
                let inner = value.borrow();

                vec![if call && inner.is_callable(false) {
                    if let Value::Token(_) = &*inner {
                        compiler.scopes[0].consuming = true;
                    }

                    Op::CallStatic(compiler.define_static(value.clone()))
                } else {
                    // void, true and false can be directly pushed
                    match &*inner {
                        Value::Integer(0) => Op::Push0,
                        Value::Integer(1) => Op::Push1,
                        Value::Void => Op::PushVoid,
                        Value::Null => Op::PushNull,
                        Value::True => Op::PushTrue,
                        Value::False => Op::PushFalse,
                        _ => Op::LoadStatic(compiler.define_static(value.clone())),
                    }
                }]
            }
            TraversalResult::Identifier(name, offset) => {
                // In case there is a use of a known constant,
                // directly return its value as TraversalResult.
                if let Some(value) = compiler.get_constant(&name) {
                    TraversalResult::Value(value).into_ops(compiler, call)
                } else {
                    let usage = if call {
                        Usage::CallOrCopy { name, offset }
                    } else {
                        Usage::Load { name, offset }
                    };

                    usage.resolve_or_dispose(compiler)
                }
            }
            TraversalResult::Ops(ops) => ops,
        }
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(RefValue) when static_expression_evaluation-feature
    is enabled, the TraversalResult contains a value and this value is NOT a callable! */
    fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let TraversalResult::Value(value) = self {
                if !value.borrow().is_callable(false) {
                    return Ok(value.clone());
                }
            }
        }

        Err(())
    }
}

/** Compiler symbolic scope.

In Tokay code, this relates to any block. Parselet blocks (parselets) introduce new variable scopes.
*/
#[derive(Debug)]
pub(crate) struct Scope {
    variables: Option<HashMap<String, usize>>, // Variable symbol table; Determines whether a scope is a parselet-level scope or just block scope
    constants: HashMap<String, RefValue>,      // Constants symbol table
    begin: Vec<Op>,     // Begin operations; Can only be set for parselet-scopes
    end: Vec<Op>,       // End operations; Can only be set for parselet-scopes
    usage_start: usize, // Begin of usages to resolve until when scope is closed
    consuming: bool, // Determines whether the scope is consuming input for early consumable detection
}

/** Tokay compiler instance, with related objects. */
pub struct Compiler {
    parser: Option<parser::Parser>, //Tokay parser
    pub debug: bool,
    pub interactive: bool,
    pub(super) statics: RefCell<Vec<RefValue>>, // Static values and parselets collected during compile
    scopes: Vec<Scope>,                         // Current compilation scopes
    pub(super) usages: Vec<Result<Vec<Op>, Usage>>, // Usages of symbols in parselets
    pub(super) errors: Vec<Error>,              // Collected errors during compilation
}

impl Compiler {
    pub fn new() -> Self {
        // Compiler initialization
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

    /** Compile a Tokay program from source into a Program struct. */
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
            parser::Parser::print(&ast);
        }

        self.traverse(&ast);

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

    /// Compile a Tokay program from a &str.
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

    // Resolve current scope
    fn resolve_scope(&mut self) {
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

    // Pops a scope and returns it.
    fn pop_scope(&mut self) -> Scope {
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

    // Pops scope and creates a parselet from it
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

    /// Retrieve address of a local variable under a given name.
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

    /* Tokay AST node traversal */

    pub(crate) fn identifier_is_valid(ident: &str) -> Result<(), Error> {
        match ident {
            "accept" | "begin" | "end" | "expect" | "false" | "for" | "if" | "in" | "not"
            | "null" | "peek" | "reject" | "return" | "true" | "void" | "while" => Err(Error::new(
                None,
                format!("Expected identifier, found reserved word '{}'", ident),
            )),
            _ => Ok(()),
        }
    }

    pub(crate) fn identifier_is_consumable(ident: &str) -> bool {
        let ch = ident.chars().next().unwrap();
        ch.is_uppercase() || ch == '_'
    }

    // Traverse either a node or a list from the AST
    fn traverse(&mut self, ast: &Value) -> TraversalResult {
        if let Some(list) = ast.get_list() {
            let mut ops = Vec::new();

            for item in list.iter() {
                match self.traverse(&item.borrow()) {
                    TraversalResult::Empty => {}
                    TraversalResult::Ops(oplist) => ops.extend(oplist),
                    _ => unreachable!("{:#?} cannot be handled", list),
                }
            }

            if ops.len() > 0 {
                TraversalResult::Ops(ops)
            } else {
                TraversalResult::Empty
            }
        } else if let Some(dict) = ast.get_dict() {
            self.traverse_node(dict)
        } else {
            panic!("Cannot traverse {:?}", ast);
        }
    }

    // Extract offset positions into an Offset structure
    fn traverse_node_offset(&self, node: &Dict) -> Option<Offset> {
        let offset = node
            .get("offset")
            .and_then(|offset| Some(offset.borrow().to_addr()));
        let row = node
            .get("row")
            .and_then(|row| Some(row.borrow().to_addr() as u32));
        let col = node
            .get("col")
            .and_then(|col| Some(col.borrow().to_addr() as u32));

        if let (Some(offset), Some(row), Some(col)) = (offset, row, col) {
            Some(Offset { offset, row, col })
        } else {
            None
        }
    }

    // Traverse a value node into a Value instance
    fn traverse_node_value(&mut self, node: &Dict) -> Value {
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        // Generate a value from the given code
        match emit {
            // Literals
            "value_string" => {
                let value = node.borrow_by_key("value").to_string();
                Value::String(value)
            }
            "value_integer" => {
                let value = node.borrow_by_key("value").to_string();
                Value::Integer(match value.parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => 0,
                })
            }
            "value_float" => {
                let value = node.borrow_by_key("value").to_string();
                Value::Float(match value.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => 0.0,
                })
            }
            "value_true" => Value::True,
            "value_false" => Value::False,
            "value_null" => Value::Null,
            "value_void" => Value::Void,

            // Tokens
            "value_token_match" | "value_token_touch" => {
                let value = utils::unescape(node.borrow_by_key("value").to_string());

                if emit == "value_token_match" {
                    Token::Match(value).into_value()
                } else {
                    Token::Touch(value).into_value()
                }
            }
            "value_token_any" => Token::Any.into_value(),
            "value_token_ccl" => {
                let node = node.borrow_by_key("children").to_dict();

                let emit = node.borrow_by_key("emit");
                let emit = emit.get_string().unwrap();

                let children = node.borrow_by_key("children").to_list();

                let mut ccl = Ccl::new();

                for range in children {
                    let range = range.borrow().to_dict();

                    let emit = range.borrow_by_key("emit");
                    let emit = emit.get_string().unwrap();

                    let value = range.borrow_by_key("value");
                    let value = value.get_string().unwrap();

                    match &emit[..] {
                        "char" => {
                            let ch = value.chars().next().unwrap();
                            ccl.add(ch..=ch);
                        }
                        "range" => {
                            let from = value.chars().nth(0).unwrap();
                            let to = value.chars().nth(1).unwrap();

                            ccl.add(from..=to);
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }

                if emit == "ccl_neg" {
                    ccl.negate();
                } else {
                    assert!(emit == "ccl");
                }

                Token::Char(ccl).into_value()
            }

            // Parselets
            "value_parselet" => {
                self.push_scope(true);

                let children = node.borrow_by_key("children");

                let (args, body) = if let Some(children) = children.get_list() {
                    assert!(children.len() == 2);
                    (Some(children[0].borrow()), children[1].borrow())
                } else {
                    (None, children)
                };

                // Create signature
                let mut sig: Vec<(String, Option<usize>)> = Vec::new();
                let mut sig_names = HashSet::new();

                if let Some(args) = args {
                    for node in args.to_list() {
                        let node = node.borrow();
                        let node = node.get_dict().unwrap();

                        let children = node.borrow_by_key("children").to_list();

                        let ident = children.borrow_by_idx(0);
                        let ident = ident.get_dict().unwrap().borrow_by_key("value").to_string();

                        // Check for correct identifier semantics
                        if !ident.chars().nth(0).unwrap().is_lowercase() {
                            self.errors.push(
                                Error::new(
                                    self.traverse_node_offset(node),
                                    format!("Variable identifier '{}' invalid; Use identifier starting in lower-case, e.g. '{}{}'",
                                    ident, &ident[0..1].to_lowercase(), &ident[1..])
                                )
                            );
                        }

                        // check if identifier was not provided twice
                        if sig_names.contains(&ident) {
                            self.errors.push(Error::new(
                                self.traverse_node_offset(node),
                                format!("Identifier '{}' already given in signature before", ident),
                            ));

                            continue;
                        } else {
                            sig_names.insert(ident.clone());
                        }

                        self.new_local(&ident);

                        assert!(children.len() <= 2);
                        let default = if children.len() == 2 {
                            let default = children.borrow_by_idx(1);
                            let value =
                                self.traverse_node_static(None, default.get_dict().unwrap());
                            Some(self.define_static(value))
                        } else {
                            None
                        };

                        sig.push((ident.clone(), default));
                        //println!("{} {} {:?}", emit.to_string(), ident, default);
                    }
                }

                //println!("sig = {:?}", sig);

                // Body
                let body = self.traverse_node(&body.get_dict().unwrap());
                let body = Op::from_vec(body.into_ops(self, true));

                self.create_parselet(None, sig, body, None, false, false)
                    .into_value()
            }
            _ => unimplemented!("unhandled value node {}", emit),
        }
    }

    // Traverse a static value
    fn traverse_node_static(&mut self, name: Option<String>, node: &Dict) -> RefValue {
        self.push_scope(true);

        match self.traverse_node(node) {
            TraversalResult::Empty => {
                self.pop_scope();
                Value::Void.into_refvalue()
            }
            TraversalResult::Value(value) => {
                self.pop_scope();

                if name.is_some() {
                    if let Value::Parselet(parselet) = &*value.borrow() {
                        let mut parselet = parselet.borrow_mut();
                        parselet.name = name;
                    }
                }

                value
            }
            TraversalResult::Ops(ops) => self
                .create_parselet(name, Vec::new(), Op::from_vec(ops), None, false, false)
                .into_value()
                .into_refvalue(),
            _ => unreachable!(),
        }
    }

    // Traverse lvalue
    fn traverse_node_lvalue(&mut self, node: &Dict, store: bool, hold: bool) -> TraversalResult {
        let children = node.borrow_by_key("children").to_list();

        let mut ops = Vec::new();

        for (i, item) in children.iter().enumerate() {
            let item = item.borrow();
            let item = item.get_dict().unwrap();

            let emit = item.borrow_by_key("emit");
            let emit = emit.get_string().unwrap();

            match emit {
                capture if capture.starts_with("capture") => {
                    let children = item.borrow_by_key("children");

                    match capture {
                        "capture_expr" => {
                            ops.extend(self.traverse(&children).into_ops(self, false));
                            if store && hold {
                                ops.push(Op::StoreCaptureHold)
                            } else if store {
                                ops.push(Op::StoreCapture)
                            } else {
                                ops.push(Op::LoadCapture)
                            }
                        }

                        "capture_index" => {
                            let children = children.get_dict().unwrap();
                            let index = self.traverse_node_value(children);

                            if store && hold {
                                ops.push(Op::StoreFastCaptureHold(index.to_addr()));
                            } else if store {
                                ops.push(Op::StoreFastCapture(index.to_addr()));
                            } else {
                                ops.push(Op::LoadFastCapture(index.to_addr()));
                            }
                        }

                        "capture_alias" => {
                            unimplemented!("//todo");
                        }

                        _ => {
                            unreachable!();
                        }
                    }
                }

                "identifier" => {
                    let name = item.borrow_by_key("value");
                    let name = name.get_string().unwrap();

                    // Check for not assigning to a constant (at any level)
                    if self.get_constant(name).is_some() {
                        self.errors.push(Error::new(
                            self.traverse_node_offset(node),
                            format!("Cannot assign to constant '{}'", name),
                        ));

                        break;
                    }

                    if i < children.len() - 1 {
                        ops.extend(
                            Usage::Load {
                                name: name.to_string(),
                                offset: self.traverse_node_offset(item),
                            }
                            .resolve_or_dispose(self),
                        )
                    } else {
                        // Check if identifier is valid
                        if let Err(mut error) = Self::identifier_is_valid(name) {
                            if let Some(offset) = self.traverse_node_offset(node) {
                                error.patch_offset(offset);
                            }

                            self.errors.push(error);
                            break;
                        }

                        // Check if identifier is not defining a consumable
                        if Self::identifier_is_consumable(name) {
                            self.errors.push(Error::new(
                                self.traverse_node_offset(node),
                                format!(
                                    "Cannot assign variable named '{}'; Try lower-case identifier, e.g. '{}'",
                                    name, name.to_lowercase()
                                ),
                            ));

                            break;
                        }

                        ops.push(
                            /* Generates code for a symbol store, which means:

                                1. look-up local variable, and store into
                                2. look-up global variable, and store into
                                3. create local variable, and store into
                            */
                            if let Some(addr) = self.get_local(name) {
                                if store && hold {
                                    Op::StoreFastHold(addr)
                                } else if store {
                                    Op::StoreFast(addr)
                                } else {
                                    Op::LoadFast(addr)
                                }
                            } else if let Some(addr) = self.get_global(name) {
                                if store && hold {
                                    Op::StoreGlobalHold(addr)
                                } else if store {
                                    Op::StoreGlobal(addr)
                                } else {
                                    Op::LoadGlobal(addr)
                                }
                            } else {
                                let addr = self.new_local(name);
                                if store && hold {
                                    Op::StoreFastHold(addr)
                                } else if store {
                                    Op::StoreFast(addr)
                                } else {
                                    Op::LoadFast(addr)
                                }
                            },
                        )
                    }
                }

                other => {
                    unimplemented!("{:?} not implemented for lvalue", other);
                }
            }
        }

        TraversalResult::Ops(ops)
    }

    // Main traversal function, running recursively through the AST
    fn traverse_node(&mut self, node: &Dict) -> TraversalResult {
        // Normal node processing...
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        //println!("emit = {:?}", emit);

        match emit {
            "alias" => {
                let children = node.borrow_by_key("children");
                let children = children.get_list().unwrap();
                assert_eq!(children.len(), 2);

                let (left, right) = children.borrow_first_2();

                let left = self.traverse_node(&left.get_dict().unwrap());
                let right = self.traverse_node(&right.get_dict().unwrap());

                let mut ops = left.into_ops(self, true);
                ops.extend(right.into_ops(self, true));
                ops.push(Op::MakeAlias);

                TraversalResult::Ops(ops)
            }

            // assign ---------------------------------------------------------
            assign if assign.starts_with("assign") => {
                let children = node.borrow_by_key("children");
                let children = children.get_list();

                let (lvalue, value) = children.unwrap().borrow_first_2();
                let lvalue = lvalue.get_dict().unwrap();
                let value = value.get_dict().unwrap();

                let parts: Vec<&str> = assign.split("_").collect();

                let mut ops = Vec::new();

                if parts.len() > 1 && parts[1] != "hold" {
                    ops.extend(
                        self.traverse_node_lvalue(lvalue, false, false)
                            .into_ops(self, false),
                    );
                    ops.extend(self.traverse_node(value).into_ops(self, false));

                    match parts[1] {
                        "add" => ops.push(Op::IAdd),
                        "sub" => ops.push(Op::ISub),
                        "mul" => ops.push(Op::IMul),
                        "div" => ops.push(Op::IDiv),
                        _ => unreachable!(),
                    }

                    if *parts.last().unwrap() != "hold" {
                        ops.push(Op::Drop);
                    }
                } else {
                    ops.extend(self.traverse_node(value).into_ops(self, false));
                    ops.extend(
                        self.traverse_node_lvalue(lvalue, true, *parts.last().unwrap() == "hold")
                            .into_ops(self, false),
                    );
                }

                TraversalResult::Ops(ops)
            }

            // attribute ------------------------------------------------------
            "attribute" => {
                unimplemented!();

                //let mut ops = self.traverse(&node.borrow_by_key("children")).into_ops(self, true);
                //ops.push(Op::LoadAttr);

                //TraversalResult::Ops(ops)
            }
            // begin ----------------------------------------------------------
            "begin" | "end" => {
                if self.scopes[0].variables.is_none() {
                    self.errors.push(Error::new(
                        self.traverse_node_offset(node),
                        format!("'{}' may only be used in parselet scope", emit),
                    ))
                }

                let ops = self
                    .traverse(&node.borrow_by_key("children"))
                    .into_ops(self, true);

                if emit == "begin" {
                    self.scopes[0].begin.extend(ops);
                } else {
                    self.scopes[0].end.extend(ops);
                }

                TraversalResult::Empty
            }

            // block ----------------------------------------------------------
            "block" => {
                if let Some(children) = node.get("children") {
                    let body = self.traverse(&children.borrow()).into_ops(self, true);
                    TraversalResult::Ops(vec![Block::new(body)])
                } else {
                    TraversalResult::Empty
                }
            }

            // call -----------------------------------------------------------
            "call" => {
                let children = node.borrow_by_key("children");
                let children = children.to_list();

                let mut ops = Vec::new();
                let mut args = 0;
                let mut nargs = 0;

                if children.len() > 1 {
                    let params = children[1].borrow().to_list();

                    for param in &params {
                        let param = param.borrow();
                        let param = param.get_dict().unwrap();

                        let emit = param.borrow_by_key("emit");

                        match emit.get_string().unwrap() {
                            "param" => {
                                if nargs > 0 {
                                    self.errors.push(Error::new(
                                        self.traverse_node_offset(node),
                                        format!(
                                            "Sequencial arguments need to be specified before named arguments."
                                        ),
                                    ));

                                    continue;
                                }

                                ops.extend(
                                    self.traverse(&param.borrow_by_key("children"))
                                        .into_ops(self, false),
                                );
                                args += 1;
                            }

                            "param_named" => {
                                let children = param.borrow_by_key("children").to_list();

                                ops.extend(
                                    self.traverse(&children.borrow_by_idx(1))
                                        .into_ops(self, false),
                                );

                                let ident = children.borrow_by_idx(0);
                                let ident =
                                    ident.get_dict().unwrap().borrow_by_key("value").to_string();
                                ops.push(Op::LoadStatic(
                                    self.define_static(Value::String(ident).into_refvalue()),
                                ));

                                nargs += 1;
                            }

                            other => unimplemented!("Unhandled parameter type {:?}", other),
                        }
                    }
                }

                // When calling with nargs, create a nargs dict first
                if nargs > 0 {
                    ops.push(Op::MakeDict(nargs));
                }

                // Push call position here
                if let Some(offset) = self.traverse_node_offset(node) {
                    ops.push(Op::Offset(Box::new(offset)));
                }

                // Perform static call or resolved rvalue call
                let callee = self.traverse(&children[0].borrow());

                if let TraversalResult::Identifier(ident, offset) = callee {
                    if Self::identifier_is_consumable(&ident) {
                        self.scopes[0].consuming = true;
                    }

                    ops.extend(
                        Usage::Call {
                            name: ident.to_string(),
                            args,
                            nargs,
                            offset,
                        }
                        .resolve_or_dispose(self),
                    )
                } else {
                    ops.extend(callee.into_ops(self, false));

                    if args == 0 && nargs == 0 {
                        ops.push(Op::Call);
                    } else if args > 0 && nargs == 0 {
                        ops.push(Op::CallArg(args));
                    } else {
                        ops.push(Op::CallArgNamed(args))
                    }
                }

                TraversalResult::Ops(ops)
            }

            // capture --------------------------------------------------------
            "capture_alias" | "capture_expr" => {
                let children = node.borrow_by_key("children");

                let mut ops = self.traverse(&children).into_ops(self, false);
                ops.push(Op::LoadCapture);
                TraversalResult::Ops(ops)
            }

            "capture_index" => {
                let children = node.borrow_by_key("children");

                let children = children.get_dict().unwrap();
                let index = self.traverse_node_value(children);
                TraversalResult::Ops(vec![Op::LoadFastCapture(index.to_addr())])
            }

            // constant -------------------------------------------------------
            "constant" => {
                let children = node.borrow_by_key("children");
                let children = children.get_list();

                let (ident, value) = children.unwrap().borrow_first_2();

                let ident = ident.get_dict().unwrap();
                let ident = ident.borrow_by_key("value");
                let ident = ident.get_string().unwrap();

                if let Err(mut error) = Self::identifier_is_valid(ident) {
                    if let Some(offset) = self.traverse_node_offset(node) {
                        error.patch_offset(offset);
                    }

                    self.errors.push(error);
                    return TraversalResult::Empty;
                }

                // Distinguish between pure values or an expression
                let node = value.get_dict().unwrap();

                // fixme: Restricted to pure values currently.
                let value = self.traverse_node_static(Some(ident.to_string()), node);

                if value.borrow().is_consuming() {
                    if !Self::identifier_is_consumable(ident) {
                        self.errors.push(Error::new(
                            self.traverse_node_offset(node),
                            format!(
                                "Cannot assign constant '{}' as consumable. Use identifier starting in upper-case, e.g. '{}{}'",
                                ident, &ident[0..1].to_uppercase(), &ident[1..]
                            )
                        ));
                    }
                } else if Self::identifier_is_consumable(ident) {
                    self.errors.push(Error::new(
                        self.traverse_node_offset(node),
                        format!(
                            "Cannot assign to constant '{}'. Use identifier starting in lower-case, e.g. '{}{}'",
                            ident, &ident[0..1].to_lowercase(), &ident[1..]
                        ),
                    ));
                }

                //println!("{} : {:?}", ident, value);
                self.set_constant(ident, value);

                // Try to resolve usage of newly introduced constant in current scope
                self.resolve_scope();

                TraversalResult::Empty
            }

            // identifier -----------------------------------------------------
            "identifier" => {
                let name = node.borrow_by_key("value").to_string();
                TraversalResult::Identifier(name, self.traverse_node_offset(node))
            }

            // index ----------------------------------------------------------
            "index" => {
                let mut ops = self
                    .traverse(&node.borrow_by_key("children"))
                    .into_ops(self, true);
                ops.push(Op::LoadIndex); // todo: in case value is an integer, use LoadFastIndex
                TraversalResult::Ops(ops)
            }

            // inplace --------------------------------------------------------
            inplace if inplace.starts_with("inplace_") => {
                let children = node.borrow_by_key("children");
                let lvalue = children.get_dict().unwrap();

                let mut ops = Vec::new();

                ops.extend(
                    self.traverse_node_lvalue(lvalue, false, false)
                        .into_ops(self, false),
                );

                let parts: Vec<&str> = inplace.split("_").collect();

                match parts[1] {
                    "pre" => {
                        ops.push(if parts[2] == "inc" {
                            Op::IInc
                        } else {
                            Op::IDec
                        });
                    }
                    "post" => {
                        ops.extend(vec![
                            Op::Dup,
                            Op::Rot2,
                            if parts[2] == "inc" {
                                Op::IInc
                            } else {
                                Op::IDec
                            },
                            Op::Drop,
                        ]);
                    }
                    _ => unreachable!(),
                }

                TraversalResult::Ops(ops)
            }

            // main -----------------------------------------------------------
            "main" => {
                let children = node.borrow_by_key("children");

                let body = self.traverse(&children).into_ops(self, true);
                let main = self.create_parselet(
                    Some("__main__".to_string()),
                    Vec::new(),
                    if body.len() > 0 {
                        Block::new(body)
                    } else {
                        Op::Nop
                    },
                    None,
                    false,
                    true,
                );

                self.define_static(main.into_value().into_refvalue());
                TraversalResult::Empty
            }

            // operator ------------------------------------------------------
            op if op.starts_with("op_") => {
                let parts: Vec<&str> = emit.split("_").collect();
                let mut ops = Vec::new();

                let op = match parts[1] {
                    "accept" | "repeat" => {
                        let children = node.borrow_by_key("children");

                        match self.traverse_node(&children.get_dict().unwrap()) {
                            TraversalResult::Value(value)
                                if matches!(&*value.borrow(), Value::Void) =>
                            {
                                if parts[1] == "repeat" {
                                    Op::Repeat
                                } else {
                                    Op::Accept
                                }
                            }
                            result => {
                                ops.extend(result.into_ops(self, false));
                                if parts[1] == "repeat" {
                                    Op::LoadRepeat
                                } else {
                                    Op::LoadAccept
                                }
                            }
                        }
                    }

                    "binary" => {
                        let children = node.borrow_by_key("children");
                        let children = children.get_list().unwrap();
                        assert_eq!(children.len(), 2);

                        let (left, right) = children.borrow_first_2();

                        let left = self.traverse_node(&left.get_dict().unwrap());
                        let right = self.traverse_node(&right.get_dict().unwrap());

                        // When both results are values, calculate in-place
                        if let (Ok(left), Ok(right)) =
                            (left.get_evaluable_value(), right.get_evaluable_value())
                        {
                            return TraversalResult::Value(
                                match parts[2] {
                                    "add" => &*left.borrow() + &*right.borrow(),
                                    "sub" => &*left.borrow() - &*right.borrow(),
                                    "mul" => &*left.borrow() * &*right.borrow(),
                                    "div" => &*left.borrow() / &*right.borrow(),
                                    _ => {
                                        unimplemented!("op_binary_{}", parts[2]);
                                    }
                                }
                                .into_refvalue(),
                            );
                        }

                        // Otherwise, generate operational code
                        ops.extend(left.into_ops(self, true));
                        ops.extend(right.into_ops(self, true));

                        match parts[2] {
                            "add" => Op::Add,
                            "sub" => Op::Sub,
                            "mul" => Op::Mul,
                            "div" => Op::Div,
                            _ => {
                                unimplemented!("op_binary_{}", parts[2]);
                            }
                        }
                    }

                    "unary" => {
                        let children = node.borrow_by_key("children");
                        let children = children.get_dict().unwrap();

                        let res = self.traverse_node(children);
                        if let Ok(value) = res.get_evaluable_value() {
                            return TraversalResult::Value(
                                match parts[2] {
                                    "not" => !&*value.borrow(),
                                    "neg" => -&*value.borrow(),
                                    _ => {
                                        unimplemented!("op_unary_{}", parts[2]);
                                    }
                                }
                                .into_refvalue(),
                            );
                        }

                        ops.extend(res.into_ops(self, true));

                        match parts[2] {
                            "not" => Op::Not,
                            "neg" => Op::Neg,
                            _ => {
                                unimplemented!("op_unary_{}", parts[2]);
                            }
                        }
                    }

                    "compare" | "logical" => {
                        let children = node.borrow_by_key("children");
                        let children = children.get_list().unwrap();
                        assert_eq!(children.len(), 2);

                        let (left, right) = children.borrow_first_2();
                        let left = self.traverse_node(&left.get_dict().unwrap());
                        let right = self.traverse_node(&right.get_dict().unwrap());

                        // When both results are values, compare in-place
                        if let (Ok(left), Ok(right)) =
                            (left.get_evaluable_value(), right.get_evaluable_value())
                        {
                            return TraversalResult::Value(
                                if match parts[2] {
                                    "equal" => &*left.borrow() == &*right.borrow(),
                                    "unequal" => &*left.borrow() != &*right.borrow(),
                                    "lowerequal" => &*left.borrow() <= &*right.borrow(),
                                    "greaterequal" => &*left.borrow() >= &*right.borrow(),
                                    "lower" => &*left.borrow() < &*right.borrow(),
                                    "greater" => &*left.borrow() > &*right.borrow(),
                                    "and" => left.borrow().is_true() && right.borrow().is_true(),
                                    "or" => left.borrow().is_true() || right.borrow().is_true(),
                                    _ => {
                                        unimplemented!("op_compare_{}", parts[2]);
                                    }
                                } {
                                    Value::True.into_refvalue()
                                } else {
                                    Value::False.into_refvalue()
                                },
                            );
                        }

                        // Otherwise, generate operational code
                        ops.extend(left.into_ops(self, false));
                        ops.extend(right.into_ops(self, false));

                        match parts[2] {
                            "equal" => Op::Equal,
                            "unequal" => Op::NotEqual,
                            "lowerequal" => Op::LowerEqual,
                            "greaterequal" => Op::GreaterEqual,
                            "lower" => Op::Lower,
                            "greater" => Op::Greater,
                            "and" => Op::LogicalAnd,
                            "or" => Op::LogicalOr,
                            _ => {
                                unimplemented!("op_compare_{}", parts[2]);
                            }
                        }
                    }

                    "mod" => {
                        let children = node.borrow_by_key("children");
                        let children = children.get_dict().unwrap();

                        let res = self.traverse_node(children);

                        // Special operations for Token::Char
                        if let TraversalResult::Value(value) = &res {
                            if let Value::Token(token) = &*value.borrow() {
                                if let Token::Char(mut ccl) = *token.clone() {
                                    match parts[2] {
                                        // mod_pos on Token::Char becomes Token::Chars
                                        "pos" => {
                                            return TraversalResult::Value(
                                                Token::Chars(ccl).into_value().into_refvalue(),
                                            )
                                        }
                                        // mod_not on Token::Char becomes negated Token::Char
                                        "not" => {
                                            ccl.negate();
                                            return TraversalResult::Value(
                                                Token::Char(ccl).into_value().into_refvalue(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        let op = Op::from_vec(res.into_ops(self, true));

                        match parts[2] {
                            "pos" => op.into_positive(),
                            "kle" => op.into_kleene(),
                            "opt" => op.into_optional(),
                            "peek" => Peek::new(op).into_op(),
                            "expect" => Expect::new(op, None).into_op(),
                            "not" => Not::new(op).into_op(),
                            _ => unreachable!(),
                        }
                    }

                    "if" | "ifelse" => {
                        let children = node.borrow_by_key("children");
                        let children = children.get_list().unwrap();

                        let condition = self.traverse(&children[0].borrow());
                        let then = self.traverse(&children[1].borrow());
                        let eelse = if children.len() == 3 {
                            Some(self.traverse(&children[2].borrow()))
                        } else {
                            None
                        };

                        // Compile time evaluation; When the if fails, it doesn't need
                        // to be compiled into the program.
                        if let Ok(value) = condition.get_evaluable_value() {
                            if value.borrow().is_true() {
                                return then;
                            } else if let Some(eelse) = eelse {
                                return eelse;
                            }

                            return TraversalResult::Empty;
                        }

                        ops.extend(condition.into_ops(self, false));
                        Op::If(Box::new((
                            Op::from_vec(then.into_ops(self, true)),
                            eelse.and_then(|eelse| Some(Op::from_vec(eelse.into_ops(self, true)))),
                        )))
                    }

                    _ => {
                        unimplemented!("{} missing", op);
                    }
                };
                ops.push(op);

                TraversalResult::Ops(ops)
            }

            // rvalue ---------------------------------------------------------
            "rvalue" => {
                let children = node.borrow_by_key("children");
                let children = children.to_list();

                let mut ops = Vec::new();

                for node in children.iter() {
                    ops.extend(self.traverse(&node.borrow()).into_ops(self, false));
                }

                assert!(ops.len() > 0);
                TraversalResult::Ops(ops)
            }

            // sequence  ------------------------------------------------------
            "sequence" | "collection" => {
                let children = node.borrow_by_key("children");
                let children = children.to_list();

                let mut ops = Vec::new();

                for node in &children {
                    ops.extend(self.traverse(&node.borrow()).into_ops(self, true))
                }

                if ops.len() > 0 {
                    if emit == "collection" {
                        ops.push(Op::MakeCollection(children.len()));
                        TraversalResult::Ops(ops)
                    } else {
                        TraversalResult::Ops(vec![Sequence::new(ops)])
                    }
                } else {
                    TraversalResult::Empty
                }
            }

            // value ---------------------------------------------------------
            value if value.starts_with("value_") => {
                TraversalResult::Value(self.traverse_node_value(node).into_refvalue())
            }

            // ---------------------------------------------------------------
            _ => {
                // When there are children, try to traverse recursively
                if let Some(children) = node.get("children") {
                    self.traverse(&children.borrow())
                }
                // Otherwise, report unhandled node!
                else {
                    unreachable!("No handling for {:?}", node);
                }
            }
        }
    }
}
