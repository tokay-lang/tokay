/*! Macros for Rust-internal Tokay program definition

    This is a minimalistic Tokay compiler implemented with Rust macros to
    bootstrap the self-hosted Tokay parser defined in the parser-module.

    It can also be used for low-level tests of the VM's parsing behavior,
    see tests.
*/

/** Macro to define a Tokay program embedded in Rust code without using the Tokay parser.
In fact, this macro is used to bootstrap the Tokay parser as a Tokay program itself.
*/
#[macro_export]
macro_rules! tokay {

    // Tokay
    ( { $( $items:tt ),+ } ) => {
        {
            let mut compiler = Compiler::new();
            compiler.debug = 0;  // unset debug always

            compiler.push_parselet();  // Main
            compiler.mark_consuming();

            let main = tokay!(compiler, { $( $items ),* });

            let parselet = compiler.pop_parselet(
                Some("__main__".to_string()),
                Vec::new(),
                main.unwrap_or(ImlOp::Nop)
            ).into_value().into_refvalue();

            compiler.define_static(parselet);

            match compiler.to_program() {
                Ok(program) => {
                    if compiler.debug > 0 {
                        program.dump();
                    }
                    program
                },
                Err(errors) => {
                    for error in errors {
                        println!("{}", error);
                    }

                    panic!("Errors in compile!");
                }
            }
        }
    };

    // Assign a value
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            let name = stringify!($name).to_string();
            let value = Value::String($value.to_string()).into_refvalue();

            $compiler.set_constant(&name, value);

            //println!("assign {} = {}", stringify!($name), stringify!($value));
            None
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = { $( $item:tt ),* } ) ) => {
        {
            $compiler.push_parselet();
            $compiler.mark_consuming();

            let items = vec![
                $(
                    tokay!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            //let body = Repeat::new(body, 0, 0, true);  //this became obsolete by the compiler!

            let mut parselet = $compiler.pop_parselet(
                Some("_".to_string()),
                Vec::new(),
                body
            );

            parselet.silent = true;  // mark as silent parselet
            $compiler.set_constant("_", parselet.into_value().into_refvalue());

            //println!("assign _ = {}", stringify!($item));
            None
        }
    };

    // Assign parselet
    ( $compiler:expr, ( $name:ident = { $( $item:tt ),* } ) ) => {
        {
            let name = stringify!($name).to_string();

            if !ast::identifier_is_consumable(&name) {
                panic!("Parselet identifier must begin with an upper-case letter or underscore!");
            }

            $compiler.push_parselet();
            $compiler.mark_consuming();

            let items = vec![
                $(
                    tokay!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            let parselet = $compiler.pop_parselet(
                Some(stringify!($name).to_string()),
                Vec::new(),
                body
            ).into_value().into_refvalue();

            $compiler.define_static(parselet.clone());
            $compiler.set_constant(&name, parselet);

            None
        }
    };

    // Sequence
    ( $compiler:expr, [ $( $item:tt ),* ] ) => {
        {
            //println!("sequence");
            let items = vec![
                $(
                    tokay!($compiler, $item)
                ),*
            ];

            Some(
                Sequence::new(
                    items.into_iter()
                        .filter(|item| item.is_some())
                        .map(|item| item.unwrap())
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
                    tokay!($compiler, $item)
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
        Some(tokay!($compiler, $item).unwrap().into_kleene())
    };

    // Positive
    ( $compiler:expr, (pos $item:tt) ) => {
        Some(tokay!($compiler, $item).unwrap().into_positive())
    };

    // Optional
    ( $compiler:expr, (opt $item:tt) ) => {
        Some(tokay!($compiler, $item).unwrap().into_optional())
    };

    // Not
    ( $compiler:expr, (not $item:tt) ) => {
        Some(Not::new(tokay!($compiler, $item).unwrap()))
    };

    // Peek
    ( $compiler:expr, (peek $item:tt) ) => {
        Some(Peek::new(tokay!($compiler, $item).unwrap()))
    };

    // Expect
    ( $compiler:expr, (expect $item:tt) ) => {
        {
            let mut msg = "Expecting ".to_string();
            msg.push_str(stringify!($item));
            Some(Expect::new(tokay!($compiler, $item).unwrap(), Some(msg)))
        }
    };

    // Expect with literal
    ( $compiler:expr, (expect $item:tt, $msg:literal) ) => {
        Some(Expect::new(tokay!($compiler, $item).unwrap(), Some($msg.to_string())))
    };

    // Value
    ( $compiler:expr, (value $value:tt) ) => {
        Some(ImlOp::from(Op::LoadStatic($compiler.define_static(value!($value)))))
    };

    // Token
    ( $compiler:expr, (token $token:tt) ) => {
        {
            Some(ImlOp::from(Op::CallStatic($compiler.define_static($token.into_value().into_refvalue()))))
        }
    };

    // Call with parameters
    ( $compiler:expr, (call $ident:ident [ $( $param:tt ),* ] ) ) => {
        {
            let mut items = vec![
                $(
                    tokay!($compiler, $param).unwrap()
                ),*
            ];

            let name = stringify!($ident).to_string();

            let item = Usage::Call{
                name,
                args: items.len(),
                nargs: 0,
                offset: None
            }.resolve_or_dispose(&mut $compiler);

            items.extend(item);

            //println!("call = {} {:?}", stringify!($ident), items);
            Some(ImlOp::from_vec(items))
        }
    };

    // Call without parameters
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let name = stringify!($ident);

            let item = Usage::CallOrCopy{
                name: name.to_string(),
                offset: None
            }.resolve_or_dispose(&mut $compiler);

            Some(ImlOp::from_vec(item))
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let item = Usage::CallOrCopy{
                name: "_".to_string(),
                offset: None
            }.resolve_or_dispose(&mut $compiler);

            assert!(item.len() == 1); // Can only process statics here!
            Some(item.into_iter().next().unwrap())
        }
    };

    // Match
    ( $compiler:expr, (MATCH $literal:literal) ) => {
        {
            let token = Token::Match($literal.to_string()).into_value();
            Some(ImlOp::from(Op::CallStatic($compiler.define_static(token.into_refvalue()))))
        }
    };

    // Touch
    ( $compiler:expr, $literal:literal ) => {
        {
            let token = Token::Touch($literal.to_string()).into_value();
            Some(ImlOp::from(Op::CallStatic($compiler.define_static(token.into_refvalue()))))
        }
    };

    // Fallback
    ( $compiler:expr, $expr:tt ) => {
        {
            //println!("expr = {}", stringify!($expr));
            Some(ImlOp::from($expr))
        }
    };
}
