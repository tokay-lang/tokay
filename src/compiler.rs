use std::collections::HashMap;
use std::cell::RefCell;
use crate::token::{Match};
use crate::tokay::{Program, Parselet, Item};
use crate::value::{Value, RefValue};

struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>,
    parselets: usize
}

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

    pub fn into_program(mut self) -> Program {
        // Close any open scopes
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        // Resolve last scope
        self.resolve();

        // Finalize tokay program
        fn finalize(parselets: &Vec<RefCell<Parselet>>, 
            leftrec: &mut bool,
            nullable: &mut bool,
            item: &mut Item)
        {
            match item {
                Item::Name(name) => panic!("OH no, there is Name({}) still!", name),
                Item::Token(_) => {
                    *nullable = false;
                },
                Item::Call(idx) => {
                    if let Ok(mut parselet) = parselets[*idx].try_borrow_mut() {
                        let mut my_leftrec = parselet.leftrec;
                        let mut my_nullable = parselet.nullable;

                        finalize(
                            parselets,
                            &mut my_leftrec,
                            &mut my_nullable,
                            &mut parselet.body
                        );

                        parselet.leftrec = my_leftrec;
                        parselet.nullable = my_nullable;

                        *nullable = parselet.nullable;
                    }
                    else {
                        *leftrec = true;
                    }
                },

                Item::Sequence(sequence) => {
                    for (item, _) in sequence.items.iter_mut() {
                        finalize(
                            parselets,
                            &mut sequence.leftrec,
                            &mut sequence.nullable,
                            item
                        );

                        if !sequence.nullable {
                            break
                        }
                    }

                    *leftrec = sequence.leftrec;
                    *nullable = sequence.nullable;
                },

                Item::Block(block) => {
                    *nullable = false;

                    for item in block.items.iter_mut() {
                        let mut my_nullable = true;
                        let mut my_leftrec = true;

                        finalize(
                            parselets,
                            &mut my_leftrec,
                            &mut my_nullable,
                            item
                        );

                        if !my_nullable {
                            *nullable = false;
                        }

                        if my_leftrec {
                            block.leftrec = true;
                        }
                    }

                    *leftrec = block.leftrec;
                }

                _ => {}
            }
        }

        let mut changes = true;
        let mut loops = 0;

        while changes {
            changes = false;

            for i in 0..self.parselets.len() {
                let mut parselet = self.parselets[i].borrow_mut();
                let mut leftrec = parselet.leftrec;
                let mut nullable = parselet.nullable;

                finalize(
                    &self.parselets,
                    &mut leftrec,
                    &mut nullable,
                    &mut parselet.body
                );

                if !parselet.leftrec && leftrec {
                    parselet.leftrec = true;
                    changes = true;
                }

                if parselet.nullable && !nullable {
                    parselet.nullable = nullable;
                    changes = true;
                }
            }

            loops += 1;
        }

        println!("finalization finished after {} loops", loops);

        Program::new(self.parselets.drain(..).map(|p| p.into_inner()).collect())
    }

    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(0,
            Scope{
                variables: if variables { Some(HashMap::new()) } else { None },
                constants: HashMap::new(),
                parselets: self.parselets.len()
            }
        );
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Can't pop main scope");
        }

        self.resolve();
        self.scopes.remove(0);
    }

    pub fn get_local(&mut self, name: &str) -> Option<usize> {
        for scope in &mut self.scopes {
            if let Some(variables) = &mut scope.variables {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr)
                }
                else {
                    let addr = variables.len();
                    variables.insert(name.to_string(), addr);
                    return Some(addr)
                }
            }
        }

        None
    }

    pub fn set_constant(&mut self, name: &str, value: RefValue) {
        assert!(Self::is_constant(name));

        let addr = self.define_value(value);

        self.scopes.first_mut().unwrap().constants.insert(
            name.to_string(), addr
        );
    }

    pub fn get_constant(&self, name: &str) -> Option<RefValue> {
        assert!(Self::is_constant(name));

        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(self.values[*addr].clone());
            }
        }

        None
    }

    pub fn define_value(&mut self, value: RefValue) -> usize
    {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn define_parselet(&mut self, parselet: Parselet) -> usize
    {
        self.parselets.push(RefCell::new(parselet));
        self.parselets.len() - 1
    }

    pub fn resolve_item(&self, item: &mut Item, strict: bool) {
        match item {
            Item::Name(name) => {
                for scope in &self.scopes {
                    if let Some(addr) = scope.constants.get(name) {
                        let value = self.values[*addr].borrow();

                        match &*value {
                            Value::Parselet(p) => {
                                println!("resolved {:?} as {:?}", name, *addr);
                                *item = Item::Call(*p);
                                return;
                            },
                            Value::String(s) => {
                                *item = Item::Token(Match::new_touch(&s.clone()));
                                return;
                            },
                            _ => {
                                unimplemented!("Cannot resolve {:?}", value);
                            }
                        }
                    }
                    else if strict {
                        panic!("Cannot resolve {:?}", name);
                    }
                }

                if strict {
                    panic!("Cannot resolve {:?}", name);
                }
            },

            Item::Sequence(ref mut sequence) => {
                for (item, _) in sequence.items.iter_mut() {
                    self.resolve_item(item, strict);
                }
            },

            Item::Block(ref mut block) => {
                for item in block.items.iter_mut() {
                    self.resolve_item(item, strict);
                }
            },

            _ => {}
        };
    }

    pub fn resolve(&mut self) {
        let scope = self.scopes.first().unwrap();

        for i in scope.parselets..self.parselets.len() {
            self.resolve_item(&mut self.parselets[i].borrow_mut().body, true);
        }
    }

    pub fn is_constant(name: &str) -> bool {
        let ch = name.chars().nth(0).unwrap();
        ch.is_uppercase() || ch == '_'
    }
}


#[macro_export]
macro_rules! tokay_item {
    // Rust
    ($compiler:expr, |$var:ident| $code:block) => {
        Item::Rust(|$var| $code)
    };

    // Assign string
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {            
            $compiler.set_constant(
                stringify!($name),
                Value::String($value.to_string()).into_ref()
            );

            //println!("assign {} = {}", stringify!($name), stringify!($value));
            Item::Nop
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = $item:tt ) ) => {
        {
            let item = tokay_item!($compiler, $item);
            let parselet = $compiler.define_parselet(
                Parselet::new(item)
            );
            
            $compiler.set_constant(
                "_",
                Value::Parselet(parselet).into_ref()
            );

            //println!("assign _ = {}", stringify!($item));
            Item::Nop
        }
    };

    // Assign parselet
    ( $compiler:expr, ( $name:ident = $item:tt ) ) => {
        {
            let item = tokay_item!($compiler, $item);
            let parselet = $compiler.define_parselet(
                Parselet::new(item)
            );
            
            $compiler.set_constant(
                stringify!($name),
                Value::Parselet(parselet).into_ref()
            );

            //println!("assign {} = {}", stringify!($name), stringify!($item));
            Item::Nop
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

            Item::Sequence(
                Box::new(
                    Sequence::new(
                        items.into_iter().filter(
                            // Remove any Nops
                            |item| !matches!(item, Item::Nop)).map(
                                // Turn into (item, None) tuples
                                |item| (item, None)).collect()
                    )
                )
            )
        }
    };

    // Parselet
    ( $compiler:expr, { $( $item:tt ),* } ) => {
        {
            $compiler.push_scope(true);
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let block = Block::new(
                items.into_iter().filter(
                    // Remove any Nops
                    |item| !matches!(item, Item::Nop)).collect()
            );

            let mut item = Item::Block(
                Box::new(
                    block
                )
            );

            $compiler.pop_scope();
            item
        }
    };

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let mut item = Item::Name(stringify!($ident).to_string());
            $compiler.resolve_item(&mut item, false);
            item
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let mut item = Item::Name("_".to_string());
            $compiler.resolve_item(&mut item, false);
            item
        }
    };

    // Match / Touch
    ( $compiler:expr, $literal:literal ) => {
        {
            //println!("match = {:?} {:?}", stringify!($literal), &stringify!($literal)[1..lit.len() -1]);
            let lit = stringify!($literal);
            
            if &lit[0..1] == "'" {
                Item::Token(
                    Match::new_touch(&lit[1..lit.len() - 1])
                )
            } else {
                Item::Token(
                    Match::new(&lit[1..lit.len() - 1])
                )
            }
        }
    };

    // Fallback
    ( $compiler:expr, $expr:tt ) => {
        {
            //println!("expr = {}", stringify!($expr));
            $expr
        }
    };
}


#[macro_export]
macro_rules! tokay {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();

            {
                let main = tokay_item!(compiler, $( $items ),*);
                compiler.define_parselet(Parselet::new(main));
            }

            compiler.into_program()
        }
    }
}
