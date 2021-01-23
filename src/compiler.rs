use std::collections::HashMap;
use std::cell::RefCell;
use crate::tokay::*;
use crate::value::{Value, RefValue, BorrowByKey, BorrowByIdx, Dict};
use crate::builtin;


/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Scoped blocks (parselets) introduce new variable scopes.
*/
struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>
}


/** Tokay compiler instance, with related objects. */
pub struct Compiler {
    scopes: Vec<Scope>,                     // Current compilation scopes
    pub statics: RefCell<Vec<RefValue>>,    // Static values created during compile
}

impl Compiler {
    pub fn new() -> Self {
        let mut compiler = Self{
            scopes: vec![
                Scope{
                    variables: Some(HashMap::new()),
                    constants: HashMap::new()
                }
            ],
            statics: RefCell::new(Vec::new())
        };

        builtin::register(&mut compiler);

        compiler
    }

    pub fn resolve(&self, parselet: usize, statics_start: usize) {
        let statics = self.statics.borrow();

        // Resolve parselets inside parselet defined from statics_start
        for i in statics_start..statics.len() {
            let value = &*statics[i].borrow();

            if let Value::Parselet(p) = value {
                p.borrow_mut().resolve(
                    &self, i == parselet, false
                );
            }
        }
    }

    /** Converts the compiled information into a Program. */
    pub fn into_program(mut self) -> Program {
        // Close any open scopes
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        // Resolve constants and globals from all scopes
        let statics = self.statics.borrow().to_vec();

        for i in 0..statics.len() {
            let value = &*statics[i].borrow();

            if let Value::Parselet(p) = value {
                p.borrow_mut().resolve(&self, false, true);
            }
        }

        // Finalize
        Parselet::finalize(&statics);

        // Drain parselets into the new program
        Program::new(
            statics
        )
    }

    /// Introduces a new scope, either for variables or constants only.
    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(0,
            Scope{
                variables: if variables { Some(HashMap::new()) } else { None },
                constants: HashMap::new()
            }
        );
    }

    /** Pops current scope. Returns number of locals defined.

    The final (main) scope cannot be dropped, the function panics when
    this is tried. */
    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Can't pop main scope");
        }

        self.scopes.remove(0);
    }

    /// Returns current scope depth
    pub fn get_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Returns the total number of locals in current scope.
    pub fn get_locals(&self) -> usize {
        if let Some(locals) = &self.scopes.first().unwrap().variables {
            locals.len()
        }
        else {
            0
        }
    }

    /**
    Retrieve address of a local variable under a given name.
    */
    pub fn get_local(&self, name: &str) -> Option<usize>
    {
        for scope in &self.scopes {
            // Check for scope with variables
            if let Some(variables) = &scope.variables {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr)
                }

                break
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
                    return *addr
                }

                let addr = variables.len();
                variables.insert(name.to_string(), addr);
                return addr
            }
        }

        unreachable!("This should not be possible")
    }

    /**
    Retrieve address of a global variable.
    */
    pub fn get_global(&self, name: &str) -> Option<usize>
    {
        let variables = self.scopes.last().unwrap().variables.as_ref().unwrap();

        if let Some(addr) = variables.get(name) {
            Some(*addr)
        }
        else {
            None
        }
    }

    /** Set constant to name in current scope. */
    pub fn set_constant(&mut self, name: &str, addr: usize) {
        self.scopes.first_mut().unwrap().constants.insert(
            name.to_string(), addr
        );
    }

    /** Get constant value, either from current or preceding scope. */
    pub fn get_constant(&self, name: &str) -> Option<usize> {
        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(*addr)
            }
        }

        None
    }

    /** Defines a new static value.

    Statics are moved into the program later on. */
    pub fn define_static(&self, value: RefValue) -> usize
    {
        let mut statics = self.statics.borrow_mut();
        // todo: check for existing value, and reuse it again instead of
        // naively adding the same value multiple times
        statics.push(value);
        statics.len() - 1
    }

    /** Check if a str defines a constant or not. */
    pub fn is_constant(name: &str) -> bool {
        let ch = name.chars().nth(0).unwrap();
        ch.is_uppercase() || ch == '_'
    }

    pub fn gen_store(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name) {
            Op::StoreFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::StoreGlobal(addr)
        }
        else {
            Op::StoreFast(self.new_local(name))
        }
    }

    pub fn gen_load(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name) {
            Op::LoadFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::LoadGlobal(addr)
        }
        else {
            Op::LoadFast(self.new_local(name))
        }
    }

    /* Tokay AST node traversal */

    // Traverse either a node or a list from the AST
    pub fn traverse(&mut self, value: &Value) -> Vec<Op> {
        let mut ret = Vec::new();

        if let Some(list) = value.get_list() {
            for item in list.iter() {
                ret.extend(self.traverse(&item.borrow()));
            }
        }
        else if let Some(dict) = value.get_dict() {
            ret.extend(self.traverse_node(dict));
        }
        else {
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
            },
            "value_integer" => {
                let value = node.borrow_by_key("value").to_string();
                Value::Integer(
                    match value.parse::<i64>() {
                        Ok(i) => i,
                        Err(_) => 0
                    }
                )
            }
            "value_float" => {
                let value = node.borrow_by_key("value").to_string();
                Value::Float(
                    match value.parse::<f64>() {
                        Ok(f) => f,
                        Err(_) => 0.0
                    }
                )
            }
            "value_true" => Value::True,
            "value_false" => Value::False,
            "value_null" => Value::Null,
            "value_void" => Value::Void,
            "value_parselet" => {
                let parselet = self.traverse_node_parselet(node);
                return self.statics.borrow()[parselet].clone();
            },
            _ => unimplemented!("unhandled value node {}", emit)
        }.into_ref()
    }

    // Traverse a parselet node into a parselet address
    fn traverse_node_parselet(&mut self, node: &Dict) -> usize {
        let statics_start = self.statics.borrow().len();
        self.push_scope(true);

        let children = node.borrow_by_key("children");

        let (args, body) = if let Some(children) = children.get_list() {
            (Some(children[0].borrow()), children[1].borrow())
        }
        else {
            (None, children)
        };

        // Create signature
        let mut sig: Vec<(String, Option<usize>)> = Vec::new();

        if let Some(args) = args {
            for param in args.to_list() {
                let param = param.borrow();
                let param = param.get_dict().unwrap();
                sig.push((
                    param.borrow_by_key("value").get_string().unwrap().to_string(),
                    None
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

        println!("sig = {:?}", sig);

        // Body
        let body = self.traverse_node(&body.get_dict().unwrap());

        let parselet = self.define_static(
            Parselet::new(
                body.into_iter().next().unwrap_or(Op::Nop), self.get_locals()
            ).into_refvalue()
        );

        self.resolve(parselet, statics_start);
        self.pop_scope();

        parselet
    }

    // Main traversal function, running recursively through the AST
    pub fn traverse_node(&mut self, node: &Dict) -> Vec<Op> {
        // Normal node processing...
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        let mut ret = Vec::new();

        println!("emit = {:?}", emit);

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
            },

            // assign_constant ------------------------------------------------
            "assign_constant" => {
                let children = node.borrow_by_key("children");
                let children = children.get_list();

                let (constant, value) = children.unwrap().borrow_first_2();

                let constant = constant.get_dict().unwrap();
                let constant = constant.borrow_by_key("value");

                let value = self.traverse_node_value(value.get_dict().unwrap());
                self.set_constant(
                    constant.get_string().unwrap(),
                    self.define_static(value)
                );

                None
            },

            // block ----------------------------------------------------------
            "block" => {
                if let Some(children) = node.get("children") {
                    let body = self.traverse(&children.borrow());
                    Some(Block::new(body))
                }
                else {
                    None
                }
            },

            // call -----------------------------------------------------------
            call if call.starts_with("call_") => {
                let children = node.borrow_by_key("children");

                let mut item = if call == "call_or_load" {
                    let ident = children.get_dict().unwrap();
                    let ident = ident.borrow_by_key("value");

                    Op::Symbol(
                        ident.to_string()
                    ).into_op()
                }
                else {
                    let children = children.to_list();

                    let mut ops = if children.len() > 1 {
                        self.traverse(&children[1].borrow())
                    }
                    else {
                        Vec::new()
                    };

                    if call == "call_identifier" {
                        let ident = children[0].borrow();
                        let ident = ident.get_dict().unwrap();
                        let ident = ident.borrow_by_key("value");

                        Op::Symbol(ident.to_string()).into_op()
                    }
                    else if call == "call_rvalue" {
                        unimplemented!();
                        /*
                        ops.extend(self.traverse(&children[0].borrow()));

                        Call::Dynamic(ops).into_op()
                        */
                    }
                    else {
                        unimplemented!("{:?} is unhandled", call);
                    }
                };

                item.resolve(self, true, false);
                Some(item)
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
                                ret.push(self.gen_load(name))
                            }
                            else {
                                ret.push(self.gen_store(name))
                            }
                        },
                        other => {
                            unimplemented!(
                                "{:?} not implemented for lvalue", other);
                        }
                    }
                }

                None
            },

            // main -----------------------------------------------------------
            "main" => {
                let children = node.borrow_by_key("children");
                let main = self.traverse(&children);

                if main.len() > 0 {
                    self.define_static(
                        Parselet::new(
                            Block::new(main),
                            self.get_locals()
                        ).into_refvalue()
                    );
                }

                None
            },

            // match ----------------------------------------------------------
            "match" => {
                let value = node.borrow_by_key("value");
                Some(Match::new(value.get_string().unwrap().clone()))
            },

            // match_silent ---------------------------------------------------
            "match_silent" => {
                let value = node.borrow_by_key("value");
                Some(Match::new_silent(value.get_string().unwrap().clone()))
            },

            // modifier -------------------------------------------------------
            modifier if modifier.starts_with("mod_") => {
                let children = node.borrow_by_key("children");
                let op = self.traverse_node(children.get_dict().unwrap());
                assert_eq!(op.len(), 1);

                let op = op.into_iter().next().unwrap();

                match &modifier[4..] {
                    "peek" => Some(Op::Peek(op.into_box())),
                    "not" => Some(Op::Not(op.into_box())),
                    "kleene" => Some(op.into_kleene()),
                    "positive" => Some(op.into_positive()),
                    "optional" => Some(op.into_optional()),
                    _ => unimplemented!("{} not implemented", modifier)
                }
            },

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
                }
                else if parts[1] == "unary" {
                    let children = node.borrow_by_key("children");
                    let children = children.get_dict().unwrap();
                    ret.extend(self.traverse_node(children));

                    //fixme: unary operators
                    unimplemented!("unary missing");
                    //None
                }
                else if parts[1] == "accept" || parts[1] == "return" {
                    let children = node.borrow_by_key("children");
                    ret.extend(
                        self.traverse_node(
                            &children.get_dict().unwrap()
                        )
                    );

                    Some(Op::LoadAccept)
                }
                else {
                    None
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

                            if let Some(addr) = self.get_constant(name) {
                                ret.push(Op::LoadStatic(addr));
                            }
                            else {
                                ret.push(self.gen_load(name));
                            }
                        }

                        val if val.starts_with("value_") => {
                            ret.extend(self.traverse_node(item))
                        }

                        _ => {
                            unreachable!();
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
                    Some(
                        Sequence::new(
                            items.into_iter()
                                .map(|item| (item, None))
                                .collect()
                        )
                    )
                }
                else {
                    None
                }
            },

            // value ---------------------------------------------------------

            val if val.starts_with("value_") => {
                let value = self.traverse_node_value(node);
                Some(Op::LoadStatic(self.define_static(value)))
            },

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

/* A minimalistic Tokay compiler as Rust macros. */

#[macro_export]
macro_rules! tokay_item {

    // Assign a value
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            let name = stringify!($name).to_string();
            let value = Value::String($value.to_string()).into_ref();
            let addr = $compiler.define_static(value);

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    addr
                );

                None
            }
            else {
                Some(
                    Sequence::new(
                        vec![
                            (Op::LoadStatic(addr), None),
                            ($compiler.gen_store(&name), None)
                        ]
                    )
                )
            }

            //println!("assign {} = {}", stringify!($name), stringify!($value));
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = { $( $item:tt ),* } ) ) => {
        {
            $compiler.push_scope(true);
            let statics_start = $compiler.statics.borrow().len();

            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            let body = Repeat::new(body, 0, 0, true);

            let parselet = $compiler.define_static(
                Parselet::new_silent(
                    body, $compiler.get_locals()
                ).into_refvalue()
            );

            $compiler.resolve(parselet, statics_start);
            $compiler.pop_scope();

            $compiler.set_constant(
                "_",
                parselet
            );

            //println!("assign _ = {}", stringify!($item));
            None
        }
    };

    // Assign parselet
    ( $compiler:expr, ( $name:ident = { $( $item:tt ),* } ) ) => {
        {
            let name = stringify!($name).to_string();

            $compiler.push_scope(true);
            let statics_start = $compiler.statics.borrow().len();

            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            let parselet = $compiler.define_static(
                Parselet::new(
                    body, $compiler.get_locals()
                ).into_refvalue()
            );

            $compiler.resolve(parselet, statics_start);
            $compiler.pop_scope();

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    parselet
                );

                None
            }
            else {
                Some(
                    Sequence::new(
                        vec![
                            (Op::LoadStatic(parselet), None),
                            ($compiler.gen_store(&name), None)
                        ]
                    )
                )
            }

            //println!("assign {} = {}", stringify!($name), stringify!($item));
        }
    };

    // Sequence
    ( $compiler:expr, [ $( $item:tt ),* ] ) => {
        {
            //println!("sequence");
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            Some(
                Sequence::new(
                    items.into_iter()
                        .filter(|item| item.is_some())
                        .map(|item| (item.unwrap(), None))
                        .collect()
                )
            )
        }
    };

    // Block
    ( $compiler:expr, { $( $item:tt ),* } ) => {
        {
            /*
            $(
                println!("{:?}", stringify!($item));
            )*
            */

            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            Some(
                Block::new(
                    items.into_iter()
                        .filter(|item| item.is_some())
                        .map(|item| item.unwrap())
                        .collect()
                )
            )
        }
    };

    // Kleene
    ( $compiler:expr, (kle $item:tt) ) => {
        Some(tokay_item!($compiler, $item).unwrap().into_kleene())
    };

    // Positive
    ( $compiler:expr, (pos $item:tt) ) => {
        Some(tokay_item!($compiler, $item).unwrap().into_positive())
    };

    // Optional
    ( $compiler:expr, (opt $item:tt) ) => {
        Some(tokay_item!($compiler, $item).unwrap().into_optional())
    };

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let name = stringify!($ident);

            if Compiler::is_constant(name) {
                let mut item = Op::Symbol(name.to_string());
                item.resolve(&$compiler, true, false);
                Some(item)
            }
            else {
                Some(
                    Sequence::new(
                        vec![
                            ($compiler.gen_load(name), None),
                            (Op::TryCall, None)
                        ]
                    )
                )
            }
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let mut item = Op::Symbol("_".to_string());
            item.resolve(&$compiler, false, false);
            Some(item)
        }
    };

    // Match / Touch
    ( $compiler:expr, $literal:literal ) => {
        {
            Some(Match::new_silent($literal))
        }
    };

    // Fallback
    ( $compiler:expr, $expr:tt ) => {
        {
            //println!("expr = {}", stringify!($expr));
            Some($expr)
        }
    };
}


#[macro_export]
macro_rules! tokay {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();
            let main = tokay_item!(compiler, $( $items ),*);

            if let Some(main) = main {
                compiler.define_static(
                    Parselet::new(
                        main,
                        compiler.get_locals()
                    ).into_refvalue()
                );
            }

            compiler.into_program()
        }
    }
}
