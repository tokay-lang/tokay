use std::collections::HashMap;
use std::cell::RefCell;
use crate::tokay::{Program, Parselet};
use crate::value::RefValue;


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
            self.parselets.drain(..).map(|p| p.into_inner()).collect()
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

    /** Pops current scope.

    The final (main) scope cannot be dropped, the function panics when
    this is tried. */
    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Can't pop main scope");
        }

        self.resolve(false);
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
}


/* A minimalistic Tokay compiler as Rust macros. */

#[macro_export]
macro_rules! tokay_item {

    // Assign string
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            $compiler.set_constant(
                stringify!($name),
                Value::String($value.to_string()).into_ref()
            );

            //println!("assign {} = {}", stringify!($name), stringify!($value));
            None
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = $item:tt ) ) => {
        {
            let item = tokay_item!($compiler, $item).unwrap();
            let item = Repeat::new(item, 0, 0, true);

            let parselet = $compiler.define_parselet(
                Parselet::new_muted(item)
            );

            $compiler.set_constant(
                "_",
                Value::Parselet(parselet).into_ref()
            );

            //println!("assign _ = {}", stringify!($item));
            None
        }
    };

    // Assign parselet
    ( $compiler:expr, ( $name:ident = $item:tt ) ) => {
        {
            let item = tokay_item!($compiler, $item).unwrap();
            let parselet = $compiler.define_parselet(
                Parselet::new(item, Some(stringify!($name).to_string()))
            );

            $compiler.set_constant(
                stringify!($name),
                Value::Parselet(parselet).into_ref()
            );

            //println!("assign {} = {}", stringify!($name), stringify!($item));
            None
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

    // Parselet
    ( $compiler:expr, { $( $item:tt ),* } ) => {
        {
            $compiler.push_scope(true);
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            $compiler.pop_scope();

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

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let mut item = Op::Name(stringify!($ident).to_string());
            item.resolve(&$compiler, false);
            Some(item)
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

            {
                let main = tokay_item!(compiler, $( $items ),*).unwrap(); //todo: unwrap_or_else?

                let main = Repeat::positive(
                    Block::new(
                        vec![
                            main,
                            Char::any()
                        ]
                    )
                    //main
                );


                compiler.define_parselet(
                    Parselet::new(main, None)  //fixme: name of the file?
                );
            }

            compiler.into_program()
        }
    }
}
