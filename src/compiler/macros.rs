/*
    This is a minimalistic Tokay compiler
    implemented with Rust macros to
    bootstrap the self-hosted tokay parser.
*/

#[macro_export]
macro_rules! compile_item {

    // Assign a value
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            let name = stringify!($name).to_string();
            let value = Value::String($value.to_string()).into_ref();
            let addr = $compiler.define_static(value);

            $compiler.set_constant(
                &name,
                addr
            );

            //println!("assign {} = {}", stringify!($name), stringify!($value));
            None
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = { $( $item:tt ),* } ) ) => {
        {
            $compiler.push_scope(true);

            let items = vec![
                $(
                    compile_item!($compiler, $item)
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
                    body,
                    $compiler.get_locals(),
                    Op::Nop,
                    Op::Nop
                ).into_refvalue()
            );

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

            let items = vec![
                $(
                    compile_item!($compiler, $item)
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
                    body,
                    $compiler.get_locals(),
                    Op::Nop,
                    Op::Nop,
                ).into_refvalue()
            );

            $compiler.pop_scope();

            $compiler.set_constant(
                &name,
                parselet
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
                    compile_item!($compiler, $item)
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
                    compile_item!($compiler, $item)
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
        Some(compile_item!($compiler, $item).unwrap().into_kleene())
    };

    // Positive
    ( $compiler:expr, (pos $item:tt) ) => {
        Some(compile_item!($compiler, $item).unwrap().into_positive())
    };

    // Optional
    ( $compiler:expr, (opt $item:tt) ) => {
        Some(compile_item!($compiler, $item).unwrap().into_optional())
    };

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let name = stringify!($ident);

            let item = Usage::Symbol(
                name.to_string()
            ).resolve_or_dispose(&mut $compiler);

            assert!(item.len() == 1); // Can only process statics here!
            Some(item.into_iter().next().unwrap())
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let item = Usage::Symbol(
                "_".to_string()
            ).resolve_or_dispose(&mut $compiler);

            assert!(item.len() == 1); // Can only process statics here!
            Some(item.into_iter().next().unwrap())
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
macro_rules! compile {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();
            let main = compile_item!(compiler, $( $items ),*);

            if let Some(main) = main {
                compiler.define_static(
                    Parselet::new(
                        main,
                        compiler.get_locals(),
                        Op::Nop,
                        Op::Nop,
                    ).into_refvalue()
                );
            }

            compiler.into_program()
        }
    }
}
