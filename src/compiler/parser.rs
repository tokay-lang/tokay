use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::value::Value;
use crate::vm::*;
use crate::{ccl, value};

/*
    This is a minimalistic Tokay compiler implemented with Rust macros to
    bootstrap the self-hosted Tokay parser defined below.
*/

macro_rules! compile_item {

    // Assign a value
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            let name = stringify!($name).to_string();
            let value = Value::String($value.to_string()).into_refvalue();
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

            let mut parselet = Parselet::new(
                Vec::new(),
                $compiler.get_locals(),
                Op::Nop,
                Op::Nop,
                body
            );
            parselet.silent = true;

            let parselet = $compiler.define_static(
                parselet.into_value().into_refvalue()
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
                    Vec::new(),
                    $compiler.get_locals(),
                    Op::Nop,
                    Op::Nop,
                    body,
                ).into_value().into_refvalue()
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

    // Not
    ( $compiler:expr, (not $item:tt) ) => {
        Some(Not::new(compile_item!($compiler, $item).unwrap()))
    };

    // Peek
    ( $compiler:expr, (peek $item:tt) ) => {
        Some(Peek::new(compile_item!($compiler, $item).unwrap()))
    };

    // Expect
    ( $compiler:expr, (expect $item:tt) ) => {
        {
            let mut msg = "Expecting ".to_string();
            msg.push_str(stringify!($item));
            Some(Expect::new(compile_item!($compiler, $item).unwrap(), Some(msg)))
        }
    };

    // Expect with literal
    ( $compiler:expr, (expect $item:tt, $msg:literal) ) => {
        Some(Expect::new(compile_item!($compiler, $item).unwrap(), Some($msg)))
    };

    // Value
    ( $compiler:expr, (value $value:tt) ) => {
        Some(Op::LoadStatic($compiler.define_static(value!($value))))
    };

    // Call with parameters
    ( $compiler:expr, (call $ident:ident [ $( $param:tt ),* ] ) ) => {
        {
            let mut items = vec![
                $(
                    compile_item!($compiler, $param).unwrap()
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
            Some(Op::from_vec(items))
        }
    };

    // Call without parameters
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let name = stringify!($ident);

            let item = Usage::Symbol{
                name: name.to_string(),
                offset: None
            }.resolve_or_dispose(&mut $compiler);

            Some(Op::from_vec(item))
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let item = Usage::Symbol{
                name: "_".to_string(),
                offset: None
            }.resolve_or_dispose(&mut $compiler);

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

macro_rules! compile {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();
            let main = compile_item!(compiler, $( $items ),*);

            if let Some(main) = main {
                compiler.define_static(
                    Parselet::new(
                        Vec::new(),
                        compiler.get_locals(),
                        Op::Nop,
                        Op::Nop,
                        main
                    ).into_value().into_refvalue()
                );
            }

            match compiler.to_program() {
                Ok(program) => {
                    //println!("{:#?}", program);
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
    }
}

/**
Implements a Tokay parser in Tokay itself, using the compiler macros from above.
This is the general place to change syntax and modify the design of the abstract syntax tree.
*/

pub struct Parser(Program);

impl Parser {
    pub(super) fn new() -> Self {
        Self(compile!({
        // ----------------------------------------------------------------------------

        // Whitespace & EOL

        (_ = {
            [" "],
            ["#", (Chars::until('\n')), "\n"],
            ["\\", "\n"]
        }),

        (T_EOL = {
            [(Chars::char('\n')), _, (Op::Skip)],
            [(Chars::char(';')), _, (Op::Skip)]
        }),

        // Prime Tokens (might probably be replaced by something native, pluggable one)

        (T_Identifier = {
            [
                (Chars::new_silent(ccl!['A'..='Z', 'a'..='z', '_'..='_'])),
                (Repeat::optional_silent(
                    Chars::span(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
                )),
                (call collect[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_String = {
            [
                "\"",
                (Chars::until('"')),     //fixme: Escape sequences (using Until built-in parselet)
                "\""
            ]
        }),

        (T_Match = {
            [
                "\'",
                (Chars::until('\'')),    //fixme: Escape sequences (using Until built-in parselet)
                "\'"
            ]
        }),

        (T_Integer = {
            // todo: implement as built-in Parselet
            [(Chars::span(ccl!['0'..='9'])), (call collect[(value "value_integer")])]
        }),

        (T_Float = {
            // todo: implement as built-in Parselet
            [(Chars::span(ccl!['0'..='9'])), ".",
                (Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
                    (call collect[(value "value_float"), (Op::LoadFastCapture(0))])],
            [(Repeat::optional_silent(Chars::span(ccl!['0'..='9']))),
                ".", (Chars::span(ccl!['0'..='9'])),
                    (call collect[(value "value_float"), (Op::LoadFastCapture(0))])]
        }),

        // Statics, Variables & Constants

        (Tail = {
            [".", _, T_Identifier, _, (call collect[(value "attribute")])],
            ["[", _, Expression, "]", _, (call collect[(value "index")])]
        }),

        (Capture = {
            ["$", T_Identifier, _, (call collect[(value "capture_alias")])],
            ["$", T_Integer, _, (call collect[(value "capture_index")])],
            ["$", "(", _, Expression, ")", _, (call collect[(value "capture")])],
            ["$", (call error[(value "'$': Expecting identifier, integer or (expression)")])]
        }),

        (Variable = {
            T_Identifier,
            Capture
        }),

        (Lvalue = {
            [Variable, _, (kle Tail), (call collect[(value "lvalue")])]
        }),

        (Inplace = {
            /* todo: drafted support for inplace increment and decrement operators,
            these are not supported by the compiler, yet. */

            [Lvalue, "++", (call collect[(value "inplace_post_inc")])],
            [Lvalue, "--", (call collect[(value "inplace_post_dec")])],
            ["++", Lvalue, (call collect[(value "inplace_pre_inc")])],
            ["--", Variable, (call collect[(value "inplace_pre_dec")])],
            Variable
        }),

        (Rvalue = {
            [Inplace, _, (kle Tail), (call collect[(value "rvalue")])]
        }),

        (Parameter = {
            [T_Identifier, _, "=", _, Expression, (call collect[(value "param_named")])],
            [Expression, (call collect[(value "param")])]
        }),

        (Parameters = {
            (pos [Parameter, (opt [",", _])])
        }),

        (Call = {
            [T_Identifier, "(", _, (opt Parameters), ")", _, (call collect[(value "call_identifier")])]
            //[Rvalue, "(", _, (opt Parameters), ")", _, (call collect[(value "call_rvalue")])]
        }),

        (Literal = {
            ["true", _, (call collect[(value "value_true")])],
            ["false", _, (call collect[(value "value_false")])],
            ["void", _, (call collect[(value "value_void")])],
            ["null", _, (call collect[(value "value_null")])],
            [T_String, _, (call collect[(value "value_string")])],
            [T_Float, _],
            [T_Integer, _]
        }),

        (Token = {
            ["peek", _, Token, (call collect[(value "mod_peek")])],
            ["not", _, Token, (call collect[(value "mod_not")])],
            ["pos", _, Token, (call collect[(value "mod_positive")])],      // fixme: not final!
            ["kle", _, Token, (call collect[(value "mod_kleene")])],        // fixme: not final!
            ["opt", _, Token, (call collect[(value "mod_optional")])],      // fixme: not final!
            ["'", T_Match, "'", _, (call collect[(value "match")])],
            [T_Match, _, (call collect[(value "touch")])]
            // fixme: consumable token identifiers?
        }),

        (Value = {
            Literal,
            Parselet
        }),

        // Expression & Flow

        (Atomic = {
            ["(", _, Expression, (expect ")"), _],
            Literal,
            Token,
            Call,
            Rvalue,
            Block,
            Parselet
        }),

        (Unary = {
            ["-", _, Atomic, (call collect[(value "op_unary_sub")])],
            ["+", _, Atomic, (call collect[(value "op_unary_add")])],
            ["!", _, Atomic, (call collect[(value "op_unary_not")])],
            Atomic
        }),

        (MulDiv = {
            [MulDiv, "*", _, (expect Unary), (call collect[(value "op_binary_mul")])],
            [MulDiv, "/", _, (expect Unary), (call collect[(value "op_binary_div")])],
            Unary
        }),

        (AddSub = {
            [AddSub, "+", _, (expect MulDiv), (call collect[(value "op_binary_add")])],
            [AddSub, "-", _, (expect MulDiv), (call collect[(value "op_binary_sub")])],
            MulDiv
        }),

        (Compare = {
            [Compare, "==", _, (expect AddSub), (call collect[(value "op_compare_equal")])],
            [Compare, "!=", _, (expect AddSub), (call collect[(value "op_compare_unequal")])],
            [Compare, "<=", _, (expect AddSub), (call collect[(value "op_compare_lowerequal")])],
            [Compare, ">=", _, (expect AddSub), (call collect[(value "op_compare_greaterequal")])],
            [Compare, "<", _, (expect AddSub), (call collect[(value "op_compare_lower")])],
            [Compare, ">", _, (expect AddSub), (call collect[(value "op_compare_greater")])],
            AddSub
        }),

        (Assign = {
            [Lvalue, "=", _, Expression, (call collect[(value "assign")])] // fixme: a = b = c is possible here...
            // todo: add operators "+="", "-="", "*="", "/=" here as well
        }),

        (Expression = {
            // if
            ["if", _, Expression, Statement, "else", _, Statement,
                (call collect[(value "op_ifelse")])],
            ["if", _, Expression, Statement, (call collect[(value "op_if")])],
            ["if", _, (call error[(value "'if': Expecting condition and statement")])],

            // while
            ["while", _, Expression, (kle T_EOL), Statement, (call collect[(value "op_while")])],
            ["while", _, (call error[(value "'while': Expecting end-condition and statement")])],

            // for
            ["for", _, T_Identifier, _, "in", _, Expression, Statement, (call collect[(value "op_for_in")])],
            ["for", _, VoidStatement, ";", _, VoidStatement, ";", _, VoidStatement, VoidStatement, (call collect[(value "op_for")])],
            ["for", _, (call error[(value "'for': Expecting start; condition; iter; statement")])],

            // normal comparison
            Compare
        }),

        (VoidStatement = {
            Statement,
            (call collect[(value "value_void")])
        }),

        (Statement = {
            ["return", _, Expression, (call collect[(value "op_return")])],
            ["return", _, (call collect[(value "op_returnvoid")])],
            ["accept", _, Expression, (call collect[(value "op_accept")])],
            ["accept", _, (call collect[(value "op_acceptvoid")])],
            ["reject", _, (call collect[(value "op_reject")])],
            Assign,
            Expression
        }),

        // Parselet

        (Argument = {
            //[T_Identifier, _, ":", _, (opt Value), (call collect[(value "arg_constant")])],  // todo: later...
            [T_Identifier, _, (opt ["=", _, (opt Value)]), (call collect[(value "arg")])]
        }),

        (Arguments = {
            (pos [Argument, (opt [",", _])])
        }),

        (Parselet = {
            ["@", _, (opt Arguments), Block, (call collect[(value "value_parselet")])],
            ["@", _, Sequence, (call collect[(value "value_parselet")])]
        }),

        (Block = {
            ["{", _, Sequences, _, (expect "}"), _, (call collect[(value "block")])],
            ["{", _, (expect "}"), _, (Op::PushVoid), (call collect[(value "block")])]
        }),

        // Sequences

        (Sequences = {
            (pos Sequence)
        }),

        (Sequence = {
            ["begin", _, Statement, (call collect[(value "begin")])],
            ["end", _, Statement, (call collect[(value "end")])],
            [(pos Item), (call collect[(value "sequence")])],
            [T_EOL, (Op::Skip)]
        }),

        (Item = {
            // todo: Recognize aliases
            [T_Identifier, _, ":", _, Value, T_EOL, (call collect[(value "assign_constant")])],
            Statement
        }),

        /*
        (TokenModifier = {
            ["!", TokenModifier, (call collect[(value "mod_not")])],
            ["~", TokenModifier, (call collect[(value "mod_peek")])],
            [Token, "+", _, (call collect[(value "mod_positive")])],
            [Token, "*", _, (call collect[(value "mod_kleene")])],
            [Token, "?", _, (call collect[(value "mod_optional")])],
            [
                Token, _,
                (Op::Peek(
                    Op::Not(
                        Chars::new(ccl![
                            '='..='=',
                            '+'..='+',
                            '-'..='-',
                            '*'..='*',
                            '/'..='/'
                            // todo: More to come?
                        ]).into_box()
                    ).into_box()
                ))
            ]
        }),

        (Token = {
            [T_String, (call collect[(value "match")])],
            [T_LightString, (call collect[(value "touch")])],
            [".", _, (call collect[(value "any")])],
            Call,
            [T_Identifier, (call collect[(value "call_or_load")])],
            Parselet
        }),
        */

        (Tokay = {
            Sequences
        }),

        [_, Tokay, (call collect[(value "main")])]

        // ----------------------------------------------------------------------------
                    }))
    }

    pub(super) fn parse(&self, mut reader: Reader) -> Result<Value, Error> {
        //self.0.dump();
        let mut runtime = Runtime::new(&self.0, &mut reader);

        match self.0.run(&mut runtime) {
            Ok(Some(ast)) => {
                let ast = Value::from_ref(ast).unwrap();

                if ast.get_dict().is_some() {
                    Ok(ast)
                } else {
                    Err(Error::new(None, "Parse error".to_string()))
                }
            }
            Ok(None) => Ok(Value::Void),
            Err(error) => Err(error),
        }
    }

    pub(super) fn print(ast: &Value) {
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
}
