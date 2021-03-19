use super::*;
use crate::builtin;
use crate::reader::{Reader, Offset};
use crate::error::Error;
use crate::value::{BorrowByIdx, BorrowByKey, Dict, RefValue, Value};
use crate::vm::*;
use std::cell::RefCell;
use std::collections::HashMap;

/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Scoped blocks (parselets) introduce new variable scopes.
*/
#[derive(Debug)]
struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>,
    begin: Vec<Op>,
    end: Vec<Op>,
    usage_start: usize,
}

/** Tokay compiler instance, with related objects. */
pub struct Compiler {
    parser: Option<parser::Parser>,  //Tokay parser
    pub(super) statics: RefCell<Vec<RefValue>>, // Static values and parselets collected during compile
    scopes: Vec<Scope>,                         // Current compilation scopes
    pub(super) usages: Vec<Result<Vec<Op>, Usage>>, // Usages of symbols in parselets
}

impl Compiler {
    pub fn new() -> Self {
        // Compiler initialization
        Self {
            parser: None,
            statics: RefCell::new(Vec::new()),
            scopes: Vec::new(),
            usages: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        // Preparation of the global scope (this is currenlty required on every compile)
        assert!(self.scopes.len() == 0);

        self.push_scope(true);  // Global scope
        builtin::register(self);  // Builtins
    }

    /** Compile a Tokay program from source into a Program struct. */
    pub fn compile(&mut self, reader: Reader) -> Option<Program> {
        self.init();

        // Create a parser when not already done
        if self.parser.is_none() {
            self.parser = Some(Parser::new());
        }

        let ast = match self.parser.as_ref().unwrap().parse(reader) {
            Ok(ast) => ast,
            Err(error) => {
                println!("{}", error);
                return None
            }
        };

        self.traverse(&ast);
        let program = match self.to_program() {
            Ok(program) => program,
            Err(errors) => {
                for error in errors {
                    println!("{}", error);
                }

                return None
            }
        };

        Some(program)
    }

    /** Converts the compiled information into a Program. */
    pub(super) fn to_program(&mut self) -> Result<Program, Vec<Error>> {
        let mut errors = Vec::new();

        // Close any open scopes except the main scope.
        while self.scopes.len() > 0 {
            self.pop_scope();
        }

        let mut usages = self
            .usages
            .drain(..)
            .map(|usage| {
                match usage {
                    Ok(usage) => usage,
                    Err(usage) => {
                        let error = match usage {
                            Usage::Symbol{name, offset} => {
                                Error::new(offset, format!("Unresolved symbol '{}'", name))
                            }

                            Usage::Call{name, args: _, nargs: _, offset} => {
                                Error::new(offset, format!("Unresolved call to '{}'", name))
                            }
                        };

                        errors.push(error);
                        vec![Op::Nop]  // Dummy instruction
                    }
                }
            })
            .collect();

        // Stop when errors are found
        if errors.len() > 0 {
            return Err(errors);
        }

        let statics = self.statics.borrow().to_vec();

        /*
        Finalize the program according to a grammar's view;

        Detect both left-recursive and nullable (=no input consuming)
        structures inside the parselet static call chains, and insert
        resolved usages.
        */
        let mut changes = true;
        //let mut loops = 0;

        while changes {
            changes = false;

            for i in 0..statics.len() {
                if let Value::Parselet(parselet) = &*statics[i].borrow() {
                    let mut parselet = parselet.borrow_mut();
                    let mut leftrec = parselet.leftrec;
                    let mut nullable = parselet.nullable;

                    parselet
                        .body
                        .finalize(&statics, &mut usages, &mut leftrec, &mut nullable);

                    if !parselet.leftrec && leftrec {
                        parselet.leftrec = true;
                        changes = true;
                    }

                    if parselet.nullable && !nullable {
                        parselet.nullable = nullable;
                        changes = true;
                    }
                }
            }

            //loops += 1;
        }

        //println!("finalization finished after {} loops", loops);

        // Make program from statics
        Ok(Program::new(statics))
    }

    /// Introduces a new scope, either for variables or constants only.
    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(
            0,
            Scope {
                variables: if variables {
                    Some(HashMap::new())
                } else {
                    None
                },
                constants: HashMap::new(),
                begin: Vec::new(),
                end: Vec::new(),
                usage_start: self.usages.len(),
            },
        );
    }

    fn take_scope(&mut self) -> Scope {
        if self.scopes.len() == 0 {
            panic!("No more scopes to pop!");
        }

        for i in self.scopes[0].usage_start..self.usages.len() {
            if let Err(usage) = &self.usages[i] {
                let res = usage.try_resolve(&self);

                if let Some(res) = res {
                    self.usages[i] = Ok(res)
                }
            }
        }

        self.scopes.remove(0)
    }

    /** Pops current scope. */
    pub fn pop_scope(&mut self) {
        self.take_scope();
    }

    /// Returns the total number of locals in current scope.
    pub fn get_locals(&self) -> usize {
        if let Some(locals) = &self.scopes.first().unwrap().variables {
            locals.len()
        } else {
            0
        }
    }

    /**
    Retrieve address of a local variable under a given name.
    */
    pub fn get_local(&self, name: &str) -> Option<usize> {
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
    pub fn new_local(&mut self, name: &str) -> usize {
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

    /**
    Retrieve address of a global variable.
    */
    pub fn get_global(&self, name: &str) -> Option<usize> {
        let variables = self.scopes.last().unwrap().variables.as_ref().unwrap();

        if let Some(addr) = variables.get(name) {
            Some(*addr)
        } else {
            None
        }
    }

    /** Set constant to name in current scope. */
    pub fn set_constant(&mut self, name: &str, addr: usize) {
        self.scopes
            .first_mut()
            .unwrap()
            .constants
            .insert(name.to_string(), addr);
    }

    /** Get constant value, either from current or preceding scope. */
    pub fn get_constant(&self, name: &str) -> Option<usize> {
        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(*addr);
            }
        }

        None
    }

    /** Defines a new static value.

    Statics are moved into the program later on. */
    pub fn define_static(&self, value: RefValue) -> usize {
        let mut statics = self.statics.borrow_mut();
        // todo: check for existing value, and reuse it again instead of
        // naively adding the same value multiple times
        statics.push(value);
        statics.len() - 1
    }

    /* Generates code for a symbol store, which means:

        1. look-up local variable, and store into
        2. look-up global variable, and store into
        3. create local variable, and store into
    */
    pub fn gen_store(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name) {
            Op::StoreFast(addr)
        } else if let Some(addr) = self.get_global(name) {
            Op::StoreGlobal(addr)
        } else {
            Op::StoreFast(self.new_local(name))
        }
    }

    /* Generates code for a symbol load. */
    pub fn gen_load(&mut self, name: &str, offset: Option<Offset>) -> Vec<Op> {
        Usage::Symbol {
            name: name.to_string(),
            offset,
        }
        .resolve_or_dispose(self)
    }

    /* Tokay AST node traversal */

    pub fn print(ast: &Value) {
        fn print(value: &Value, indent: usize) {
            match value {
                Value::Dict(d) => {
                    let emit = d["emit"].borrow();
                    let emit = emit.get_string().unwrap();

                    let row = d.get("row").and_then(|row| Some(row.borrow().to_addr()));
                    let col = d.get("col").and_then(|col| Some(col.borrow().to_addr()));
                    let end_row = d
                        .get("end_row")
                        .and_then(|row| Some(row.borrow().to_addr()));
                    let end_col = d
                        .get("end_col")
                        .and_then(|col| Some(col.borrow().to_addr()));

                    let value = d.get("value");
                    let children = d.get("children");

                    if let (Some(row), Some(col), Some(end_row), Some(end_col)) =
                        (row, col, end_row, end_col)
                    {
                        print!(
                            "{:indent$}{} [{}:{} - {}:{}]",
                            "",
                            emit,
                            row,
                            col,
                            end_row,
                            end_col,
                            indent = indent
                        );
                    } else if let (Some(row), Some(col)) = (row, col) {
                        print!("{:indent$}{} [{}:{}]", "", emit, row, col, indent = indent);
                    } else {
                        print!("{:indent$}{}", "", emit, indent = indent);
                    }

                    if let Some(value) = value {
                        print!(" {:?}", value.borrow());
                    }
                    print!("\n");

                    if let Some(children) = children {
                        print(&children.borrow(), indent + 1);
                    }
                }

                Value::List(l) => {
                    for item in l.iter() {
                        print(&item.borrow(), indent);
                    }
                }

                other => unimplemented!("{:?} is not implemented", other),
            }
        }

        print(ast, 0);
    }

    // Traverse either a node or a list from the AST
    pub fn traverse(&mut self, value: &Value) -> Vec<Op> {
        let mut ret = Vec::new();

        if let Some(list) = value.get_list() {
            for item in list.iter() {
                ret.extend(self.traverse(&item.borrow()));
            }
        } else if let Some(dict) = value.get_dict() {
            ret.extend(self.traverse_node(dict));
        } else {
            unimplemented!("traverse() cannot traverse {:?}", value);
        }

        ret
    }

    // Traverse a value node into a RefValue instance
    fn traverse_node_value(&mut self, node: &Dict) -> RefValue {
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        match emit {
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
            "value_parselet" => self.traverse_node_parselet(node),
            _ => unimplemented!("unhandled value node {}", emit),
        }
        .into_refvalue()
    }

    // Traverse a parselet node into a parselet address
    fn traverse_node_parselet(&mut self, node: &Dict) -> Value {
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

        if let Some(args) = args {
            for node in args.to_list() {
                let node = node.borrow();
                let node = node.get_dict().unwrap();

                let children = node.borrow_by_key("children").to_list();

                let ident = children.borrow_by_idx(0);
                let ident = ident.get_dict().unwrap().borrow_by_key("value").to_string();

                assert!(
                    ident.chars().nth(0).unwrap().is_lowercase(),
                    "Only lower-case parameter names are allowed currently"
                );
                self.new_local(&ident);

                assert!(children.len() <= 2);
                let default = if children.len() == 2 {
                    let default = children.borrow_by_idx(1);
                    let value = self.traverse_node_value(default.get_dict().unwrap());
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
        let locals = self.get_locals();
        let scope = self.take_scope();

        Parselet::new(
            sig,
            locals,
            Op::from_vec(scope.begin),
            Op::from_vec(scope.end),
            body.into_iter().next().unwrap_or(Op::Nop),
        )
        .into_value()
    }

    // Insert offset positions
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

    // Main traversal function, running recursively through the AST
    pub fn traverse_node(&mut self, node: &Dict) -> Vec<Op> {
        // Normal node processing...
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        let mut ret = Vec::new();

        //println!("emit = {:?}", emit);

        let op = match emit {
            // assign ---------------------------------------------------------
            "assign" => {
                let children = node.borrow_by_key("children");
                let children = children.get_list();

                let (lvalue, rvalue) = children.unwrap().borrow_first_2();

                let rvalue = rvalue.get_dict().unwrap();
                let lvalue = lvalue.get_dict().unwrap();

                ret.extend(self.traverse_node(rvalue));
                ret.extend(self.traverse_node(lvalue));

                None
            }

            // assign_constant ------------------------------------------------
            "assign_constant" => {
                let children = node.borrow_by_key("children");
                let children = children.get_list();

                let (constant, value) = children.unwrap().borrow_first_2();

                let constant = constant.get_dict().unwrap();
                let constant = constant.borrow_by_key("value");

                let value = self.traverse_node_value(value.get_dict().unwrap());
                self.set_constant(constant.get_string().unwrap(), self.define_static(value));

                None
            }

            // begin ----------------------------------------------------------
            "begin" | "end" => {
                if self.scopes[0].variables.is_none() {
                    panic!("'{}' may only be used in parselet scope", emit);
                }

                if let Some(children) = node.get("children") {
                    let ops = self.traverse(&children.borrow());

                    if emit == "begin" {
                        self.scopes[0].begin.extend(ops);
                    } else {
                        self.scopes[0].end.extend(ops);
                    }
                }

                None
            }

            // block ----------------------------------------------------------
            "block" => {
                if let Some(children) = node.get("children") {
                    let body = self.traverse(&children.borrow());
                    Some(Block::new(body))
                } else {
                    None
                }
            }

            // call -----------------------------------------------------------
            call if call.starts_with("call_") => {
                let children = node.borrow_by_key("children");

                let usage = if call == "call_or_load" {
                    let ident = children.get_dict().unwrap();
                    let ident = ident.borrow_by_key("value");

                    Usage::Symbol {
                        name: ident.to_string(),
                        offset: self.traverse_node_offset(node),
                    }
                } else {
                    let children = children.to_list();

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
                                        panic!("Sequencial parameters must be given first!");
                                    }

                                    ret.extend(self.traverse(&param.borrow_by_key("children")));
                                    args += 1;
                                }

                                "param_named" => {
                                    let children = param.borrow_by_key("children").to_list();

                                    ret.extend(self.traverse(&children.borrow_by_idx(1)));

                                    let ident = children.borrow_by_idx(0);
                                    let ident = ident
                                        .get_dict()
                                        .unwrap()
                                        .borrow_by_key("value")
                                        .to_string();
                                    ret.push(Op::LoadStatic(
                                        self.define_static(Value::String(ident).into_refvalue()),
                                    ));

                                    nargs += 1;
                                }

                                other => panic!("Unhandled parameter type {:?}", other),
                            }
                        }
                    }

                    if call == "call_identifier" {
                        let ident = children[0].borrow();
                        let ident = ident.get_dict().unwrap().borrow_by_key("value");

                        Usage::Call {
                            name: ident.to_string(),
                            args,
                            nargs,
                            offset: self.traverse_node_offset(node),
                        }
                    } else if call == "call_rvalue" {
                        unimplemented!();
                    } else {
                        unimplemented!("{:?} is unhandled", call);
                    }
                };

                if let Some(offset) = self.traverse_node_offset(node) {
                    ret.push(Op::Offset(Box::new(offset))); // Push call position here
                }

                ret.extend(usage.resolve_or_dispose(self)); // Push usage or resolved call

                None
            }

            // lvalue ---------------------------------------------------------
            "lvalue" => {
                let children = node.borrow_by_key("children").to_list();

                for (i, item) in children.iter().enumerate() {
                    let item = item.borrow();
                    let item = item.get_dict().unwrap();

                    let emit = item.borrow_by_key("emit");
                    let emit = emit.get_string().unwrap();

                    match emit {
                        capture if capture.starts_with("capture") => {
                            let children = item.borrow_by_key("children");

                            match capture {
                                "capture" => {
                                    ret.extend(self.traverse(&children));
                                    ret.push(Op::StoreCapture)
                                }

                                "capture_index" => {
                                    let children = children.get_dict().unwrap();
                                    let index = self.traverse_node_value(children);
                                    ret.push(Op::StoreFastCapture(index.borrow().to_addr()));
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

                            if self.get_constant(name).is_some() {
                                panic!("Cannot assign to {} as it is declared as constant", name)
                            }

                            if i < children.len() - 1 {
                                ret.extend(self.gen_load(name, self.traverse_node_offset(item)))
                            } else {
                                ret.push(self.gen_store(name))
                            }
                        }
                        other => {
                            unimplemented!("{:?} not implemented for lvalue", other);
                        }
                    }
                }

                None
            }

            // main -----------------------------------------------------------
            "main" => {
                let children = node.borrow_by_key("children");

                let main = self.traverse(&children);
                let locals = self.get_locals();
                let scope = self.take_scope();

                if main.len() > 0 {
                    self.define_static(
                        Parselet::new(
                            Vec::new(),
                            locals,
                            Op::from_vec(scope.begin),
                            Op::from_vec(scope.end),
                            Block::new(main),
                        )
                        .into_value()
                        .into_refvalue(),
                    );
                }

                None
            }

            // match ----------------------------------------------------------
            "match" => {
                let value = node.borrow_by_key("value");
                Some(Match::new(value.get_string().unwrap().clone()))
            }

            // touch ----------------------------------------------------------
            "touch" => {
                let value = node.borrow_by_key("value");
                Some(Match::new_silent(value.get_string().unwrap().clone()))
            }

            // modifier -------------------------------------------------------
            modifier if modifier.starts_with("mod_") => {
                let children = node.borrow_by_key("children");
                let op = self.traverse_node(children.get_dict().unwrap());
                assert_eq!(op.len(), 1);

                let op = op.into_iter().next().unwrap();

                match &modifier[4..] {
                    "not" => Some(Not::new(op)),
                    "peek" => Some(Peek::new(op)),
                    "kleene" => Some(op.into_kleene()),
                    "positive" => Some(op.into_positive()),
                    "optional" => Some(op.into_optional()),
                    _ => unimplemented!("{} not implemented", modifier),
                }
            }

            // operator ------------------------------------------------------
            op if op.starts_with("op_") => {
                let parts: Vec<&str> = emit.split("_").collect();

                if parts[1] == "binary" {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();
                    assert_eq!(children.len(), 2);

                    let (left, right) = children.borrow_first_2();
                    ret.extend(self.traverse_node(&left.get_dict().unwrap()));
                    ret.extend(self.traverse_node(&right.get_dict().unwrap()));

                    match parts[2] {
                        "add" => Some(Op::Add),
                        "sub" => Some(Op::Sub),
                        "mul" => Some(Op::Mul),
                        "div" => Some(Op::Div),
                        _ => {
                            unimplemented!("op_binary_{}", parts[2]);
                        }
                    }
                } else if parts[1] == "unary" {
                    let children = node.borrow_by_key("children");
                    let children = children.get_dict().unwrap();
                    ret.extend(self.traverse_node(children));

                    match parts[2] {
                        "not" => Some(Op::Not),
                        _ => {
                            unimplemented!("op_unary_{}", parts[2]);
                        }
                    }
                } else if parts[1] == "accept" || parts[1] == "return" {
                    let children = node.borrow_by_key("children");
                    ret.extend(self.traverse_node(&children.get_dict().unwrap()));

                    Some(Op::LoadAccept)
                } else if parts[1] == "if" || parts[1] == "ifelse" {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();

                    ret.extend(self.traverse(&children[0].borrow()));
                    let then = Op::from_vec(self.traverse(&children[1].borrow()));
                    let eelse = if children.len() == 3 {
                        Some(Op::from_vec(self.traverse(&children[2].borrow())))
                    } else {
                        None
                    };

                    Some(Op::If(Box::new((then, eelse))))
                } else {
                    unimplemented!("{} missing", op);
                }
            }

            // rvalue ---------------------------------------------------------
            "rvalue" => {
                let children = node.borrow_by_key("children").to_list();

                for item in children.iter() {
                    let item = item.borrow();
                    let item = item.get_dict().unwrap();

                    let emit = item.borrow_by_key("emit");
                    let emit = emit.get_string().unwrap();

                    match emit {
                        capture if capture.starts_with("capture") => {
                            let children = item.borrow_by_key("children");

                            match capture {
                                "capture" => {
                                    ret.extend(self.traverse(&children));
                                    ret.push(Op::LoadCapture)
                                }

                                "capture_index" => {
                                    let children = children.get_dict().unwrap();
                                    let index = self.traverse_node_value(children);
                                    ret.push(Op::LoadFastCapture(index.borrow().to_addr()));
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

                            ret.extend(self.gen_load(name, self.traverse_node_offset(node)));
                        }

                        _ => ret.extend(self.traverse_node(item)),
                    }
                }

                None
            }

            // sequence ------------------------------------------------------
            "sequence" => {
                let children = node.borrow_by_key("children");
                let items = self.traverse(&children);
                //todo: Handle aliases...

                if items.len() > 0 {
                    Some(Sequence::new(
                        items.into_iter().map(|item| (item, None)).collect(),
                    ))
                } else {
                    None
                }
            }

            // value ---------------------------------------------------------
            val if val.starts_with("value_") => {
                let value = self.traverse_node_value(node);
                Some(Op::LoadStatic(self.define_static(value)))
            }

            // ---------------------------------------------------------------
            _ => {
                // When there are children, try to traverse recursively
                if let Some(children) = node.get("children") {
                    ret.extend(self.traverse(&children.borrow()));
                    None
                }
                // Otherwise, report unhandled node!
                else {
                    unreachable!("No handling for {:?}", node);
                }
            }
        };

        if let Some(op) = op {
            ret.push(op);
        }

        ret
    }
}
