use std::collections::HashMap;
use std::cell::RefCell;
use crate::tokay::*;
use crate::value::{Value, RefValue, BorrowByKey, BorrowByIdx, Dict};


/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Scoped blocks (parselets) introduce new variable scopes.
*/
struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>,
    parselets: usize
}


/** Tokay compiler instance, with related objects. */
pub struct Compiler {
    scopes: Vec<Scope>,                     // Current compilation scopes
    values: Vec<RefValue>,                  // Constant values collected during compile
    parselets: Vec<RefCell<Parselet>>       // Parselets
}

impl Compiler {
    pub fn new() -> Self {
        Self{
            scopes: vec![
                Scope{
                    variables: Some(HashMap::new()),
                    constants: HashMap::new(),
                    parselets: 0
                }
            ],
            values: Vec::new(),
            parselets: Vec::new()
        }
    }

    /** Converts the compiled information into a Program. */
    pub fn into_program(mut self) -> Program {
        // Close any open scopes
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        // Resolve last scope
        self.resolve(true);

        // Finalize
        Parselet::finalize(&self.parselets);

        // Drain parselets into the new program
        Program::new(
            self.parselets.drain(..).map(|p| p.into_inner()).collect(),
            self.values,
            //self.scopes[0].variables.len()  # fixme: these are the globals...
        )
    }

    /// Introduces a new scope, either for variables or constants only.
    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(0,
            Scope{
                variables: if variables { Some(HashMap::new()) } else { None },
                constants: HashMap::new(),
                parselets: self.parselets.len()
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

        self.resolve(false);
        self.scopes.remove(0);
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
    Retrieve address of a local variable under a given name;
    The define-parameter for automatic variable inseration
    in case it doesn't exist.
    */
    pub fn get_local(&mut self, name: &str, define: bool)
        -> Option<usize>
    {
        for scope in &mut self.scopes {
            // Check for scope with variables
            if let Some(variables) = &mut scope.variables {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr)
                }
                else if define {
                    let addr = variables.len();
                    variables.insert(name.to_string(), addr);
                    return Some(addr)
                }
                else {
                    break
                }
            }
        }

        None
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
    pub fn set_constant(&mut self, name: &str, value: RefValue) {
        assert!(Self::is_constant(name));

        let addr = self.define_value(value);

        self.scopes.first_mut().unwrap().constants.insert(
            name.to_string(), addr
        );
    }

    /** Get constant, either from current or preceding scope. */
    pub fn get_constant(&self, name: &str) -> Option<RefValue> {
        assert!(Self::is_constant(name));

        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(self.values[*addr].clone());
            }
        }

        None
    }

    /** Defines a new static value.

    Statics are moved into the program later on. */
    pub fn define_value(&mut self, value: RefValue) -> usize
    {
        // todo: check for existing value
        self.values.push(value);
        self.values.len() - 1
    }

    /** Defines a new parselet code element.

    Parselets are moved into the program later on. */
    pub fn define_parselet(&mut self, parselet: Parselet) -> usize
    {
        self.parselets.push(RefCell::new(parselet));
        self.parselets.len() - 1
    }

    /** Resolve all parseletes defined in the current scope. */
    pub fn resolve(&mut self, strict: bool) {
        let scope = self.scopes.first().unwrap();

        for i in scope.parselets..self.parselets.len() {
            self.parselets[i].borrow_mut().resolve(&self, strict);
        }
    }

    /** Check if a str defines a constant or not. */
    pub fn is_constant(name: &str) -> bool {
        let ch = name.chars().nth(0).unwrap();
        ch.is_uppercase() || ch == '_'
    }

    pub fn gen_store(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name, false) {
            Op::StoreFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::StoreGlobal(addr)
        }
        else {
            Op::StoreFast(self.get_local(name, true).unwrap())
        }
    }

    pub fn gen_load(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name, false) {
            Op::LoadFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::LoadGlobal(addr)
        }
        else {
            Op::LoadFast(self.get_local(name, true).unwrap())
        }
    }

    /* Tokay AST node traversal */

    // Traverse something from the AST
    pub fn traverse(&mut self, value: &Value) -> Vec<Op> {
        let mut ret = Vec::new();

        if let Some(list) = value.get_list() {
            for item in list.iter() {
                let item = item.borrow();
                let res = self.traverse_node(&item.get_dict().unwrap());

                if let Some(op) = res {
                    ret.push(op);
                }
            }
        }
        else if let Some(dict) = value.get_dict() {
            let res = self.traverse_node(dict);
            if let Some(op) = res {
                ret.push(op);
            }
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
            "parselet" => {
                Value::Parselet(
                    self.traverse_node_parselet(node)
                )
            },
            _ => unimplemented!("unhandled value node {}", emit)
        }.into_ref()
    }

    // Traverse a parselet node into a parselet address
    fn traverse_node_parselet(&mut self, node: &Dict) -> usize {
        // todo: handle parameters BEFORE scope push
        self.push_scope(true);

        let children = node.borrow_by_key("children");
        let body = self.traverse_node(&children.get_dict().unwrap());
        let parselet = self.define_parselet(
            Parselet::new(body.unwrap_or(Op::Nop), self.get_locals())
        );

        self.pop_scope();

        parselet
    }

    // Main traversal function, running recursively through the AST
    pub fn traverse_node(&mut self, node: &Dict) -> Option<Op> {
        // Normal node processing...
        let emit = node.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        println!("emit={:?}", emit);

        match emit {
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
                    value
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

            // call_constant --------------------------------------------------
            "call_constant" => {
                let children = node.borrow_by_key("children");
                let constant = children.get_dict().unwrap();
                let constant = constant.borrow_by_key("value");

                let mut item = Op::Name(constant.to_string());
                item.resolve(self, false);
                Some(item)
            },

            // main -----------------------------------------------------------
            "main" => {
                let children = node.borrow_by_key("children");
                let main = self.traverse(&children);

                if main.len() > 0 {
                    self.define_parselet(
                        Parselet::new(
                            Block::new(main),
                            self.get_locals()
                        )
                    );
                }

                None
            },

            // match ----------------------------------------------------------
            "match" => {
                let value = node.borrow_by_key("value");
                Some(Match::new(value.get_string().unwrap().clone()))
            },

            // modifier -------------------------------------------------------
            modifier if modifier.starts_with("mod_") => {
                let children = node.borrow_by_key("children");
                let op = self.traverse_node(
                    children.get_dict().unwrap()
                ).unwrap();

                match &modifier[4..] {
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
                    self.traverse_node(&left.get_dict().unwrap());
                    self.traverse_node(&right.get_dict().unwrap());

                    match parts[2] {
                        // todo...
                        "add" => println!("add"),
                        "mul" => println!("mul"),
                        _ => {
                            unimplemented!("op_binary_{}", parts[2]);
                        }
                    }
                }

                None
            }

            // parselet ------------------------------------------------------

            "parselet" => {
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
                Some(Op::LoadStatic(self.define_value(value)))
            },

            // ---------------------------------------------------------------

            _ => {
                // When there are children, try to traverse recursively
                if let Some(children) = node.get("children") {
                    let res = self.traverse(&children.borrow());

                    if res.len() <= 1 {
                        res.into_iter().next()
                    }
                    else {
                        unimplemented!("Don't know how to handle return list");
                    }
                }
                // Otherwise, report unhandled node!
                else {
                    panic!("No traversal function for {:?} found", emit);
                }
            }
        }
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

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    value
                );

                None
            }
            else {
                let addr = $compiler.define_value(value);

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

            let parselet = $compiler.define_parselet(
                Parselet::new_muted(body, $compiler.get_locals())
            );

            $compiler.pop_scope();

            $compiler.set_constant(
                "_",
                Value::Parselet(parselet).into_ref()
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

            let parselet = $compiler.define_parselet(
                Parselet::new(body, $compiler.get_locals())
            );

            $compiler.pop_scope();

            let parselet = Value::Parselet(parselet).into_ref();

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    parselet
                );

                None
            }
            else {
                let addr = $compiler.define_value(parselet);

                Some(
                    Sequence::new(
                        vec![
                            (Op::LoadStatic(addr), None),
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
                let mut item = Op::Name(name.to_string());
                item.resolve(&$compiler, false);
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
            let mut item = Op::Name("_".to_string());
            item.resolve(&$compiler, false);
            Some(item)
        }
    };

    // Match / Touch
    ( $compiler:expr, $literal:literal ) => {
        {
            Some(Match::new($literal))
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
                compiler.define_parselet(
                    Parselet::new(
                        main,
                        compiler.get_locals()
                    )
                );
            }

            compiler.into_program()
        }
    }
}
