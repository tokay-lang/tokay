use super::*;
use crate::builtin;
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
    pub(super) statics: RefCell<Vec<RefValue>>, // Static values and parselets collected during compile
    scopes: Vec<Scope>,                         // Current compilation scopes
    pub(super) usages: Vec<Result<Vec<Op>, Usage>>, // Usages of symbols in parselets
}

impl Compiler {
    pub fn new() -> Self {
        // Compiler initialization
        let mut compiler = Self {
            statics: RefCell::new(Vec::new()),
            scopes: Vec::new(),
            usages: Vec::new(),
        };

        // Preparation of global scope
        compiler.push_scope(true);
        builtin::register(&mut compiler);

        compiler
    }

    /** Converts the compiled information into a Program. */
    pub fn into_program(mut self) -> Program {
        // Close any open scopes
        while self.scopes.len() > 0 {
            self.pop_scope();
        }

        let mut usages = self
            .usages
            .into_iter()
            .map(|usage| {
                if let Err(usage) = usage {
                    panic!("Unresolved usage detected {:?}", usage);
                }

                usage.unwrap()
            })
            .collect();

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
        Program::new(statics)
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

    fn _pop_scope(&mut self) -> Scope {
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
        self._pop_scope();
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
        for scope in &self.scopes {
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
    pub fn gen_load(&mut self, name: &str) -> Vec<Op> {
        Usage::Symbol(name.to_string()).resolve_or_dispose(self)
    }

    /* Tokay AST node traversal */

    pub fn print(ast: &Value) {
        fn print(value: &Value, indent: usize) {
            match value {
                Value::Dict(d) => {
                    let emit = d["emit"].borrow();
                    let emit = emit.get_string().unwrap();
                    let value = d.get("value");
                    let children = d.get("children");

                    print!("{:indent$}{}", "", emit, indent = indent);
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
            p if p.starts_with("parselet") => {
                let parselet = self.traverse_node_parselet(node);
                return self.statics.borrow()[parselet].clone();
            }
            _ => unimplemented!("unhandled value node {}", emit),
        }
        .into_ref()
    }

    // Traverse a parselet node into a parselet address
    fn traverse_node_parselet(&mut self, node: &Dict) -> usize {
        self.push_scope(true);

        let children = node.borrow_by_key("children");

        let (args, body) = if let Some(children) = children.get_list() {
            (Some(children[0].borrow()), children[1].borrow())
        } else {
            (None, children)
        };

        // Create signature
        let mut sig: Vec<(String, Option<usize>)> = Vec::new();

        if let Some(args) = args {
            for param in args.to_list() {
                let param = param.borrow();
                let param = param.get_dict().unwrap();
                sig.push((
                    param
                        .borrow_by_key("value")
                        .get_string()
                        .unwrap()
                        .to_string(),
                    None,
                ));
            }

            /*
            // todo: Write macro?

            if let Some(param) = args.get_dict() {
                sig.push((
                    param.borrow_by_key("value").get_string().unwrap().to_string(),
                    None
                ));
            }
            else if let Some(params) = args.get_list() {
                for param in params {
                    let param = param.borrow();
                    let param = param.get_dict().unwrap();
                    sig.push((
                        param.borrow_by_key("value").get_string().unwrap().to_string(),
                        None
                    ));
                }
            }
            */
        }

        //println!("sig = {:?}", sig);

        // Body
        let body = self.traverse_node(&body.get_dict().unwrap());
        let locals = self.get_locals();
        let scope = self._pop_scope();

        self.define_static(
            Parselet::new(
                body.into_iter().next().unwrap_or(Op::Nop),
                locals,
                Op::from_vec(scope.begin),
                Op::from_vec(scope.end),
            )
            .into_refvalue(),
        )
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

                    Usage::Symbol(ident.to_string())
                } else {
                    let children = children.to_list();

                    let mut ops = if children.len() > 1 {
                        self.traverse(&children[1].borrow())
                    } else {
                        Vec::new()
                    };

                    if call == "call_identifier" {
                        let ident = children[0].borrow();
                        let ident = ident.get_dict().unwrap();
                        let ident = ident.borrow_by_key("value");

                        Usage::Symbol(ident.to_string())
                    } else if call == "call_rvalue" {
                        unimplemented!();
                    /*
                    ops.extend(self.traverse(&children[0].borrow()));

                    Call::Dynamic(ops).into_op()
                    */
                    } else {
                        unimplemented!("{:?} is unhandled", call);
                    }
                };

                ret.extend(usage.resolve_or_dispose(self));
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
                                ret.extend(self.gen_load(name))
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
                let scope = self._pop_scope();

                if main.len() > 0 {
                    self.define_static(
                        Parselet::new(
                            Block::new(main),
                            locals,
                            Op::from_vec(scope.begin),
                            Op::from_vec(scope.end),
                        )
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

                    //fixme: unary operators
                    unimplemented!("unary missing");
                } else if parts[1] == "accept" || parts[1] == "return" {
                    let children = node.borrow_by_key("children");
                    ret.extend(self.traverse_node(&children.get_dict().unwrap()));

                    Some(Op::LoadAccept)
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

                            ret.extend(self.gen_load(name));
                        }

                        val if val.starts_with("value_") => ret.extend(self.traverse_node(item)),

                        /*
                        val if val.starts_with("inplace_") => {
                            let parts: Vec<&str> = emit.split("_").collect();
                            match parts[1] {
                                "pre" => {
                                    self.traverse(item);
                                    match parts[2] {

                                    }
                                }
                            }
                        },
                        */
                        _ => {
                            unimplemented!("{:?} not implemented", emit);
                        }
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
                    unreachable!("No handling for {:?}", emit);
                }
            }
        };

        if let Some(op) = op {
            ret.push(op);
        }

        ret
    }
}
