use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::token::Token;
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

            $compiler.set_constant(&name, value);

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

            let parselet = $compiler.create_parselet(Vec::new(), body, true, false).into_value().into_refvalue();

            $compiler.set_constant("_", parselet);

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

            let parselet = $compiler.create_parselet(Vec::new(), body, false, false).into_value().into_refvalue();

            $compiler.define_static(parselet.clone());
            $compiler.set_constant(&name, parselet);

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
        Some(Expect::new(compile_item!($compiler, $item).unwrap(), Some($msg.to_string())))
    };

    // Value
    ( $compiler:expr, (value $value:tt) ) => {
        Some(Op::LoadStatic($compiler.define_static(value!($value))))
    };

    // Token
    ( $compiler:expr, (token $token:tt) ) => {
        {
            Some(Op::CallStatic($compiler.define_static($token.into_value().into_refvalue())))
        }
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

            let item = Usage::LoadOrCall{
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
            let item = Usage::LoadOrCall{
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
            let token = Token::Touch($literal.to_string()).into_value();
            Some(Op::CallStatic($compiler.define_static(token.into_refvalue())))
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
                let parselet = compiler.create_parselet(
                    Vec::new(),
                    main,
                    false,
                    true
                ).into_value().into_refvalue();
                compiler.define_static(parselet);
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
    pub fn new() -> Self {
        Self(compile!({
        // ----------------------------------------------------------------------------

        // Whitespace & EOL

        (_ = {
            [" "],
            ["#", (token (Token::chars_until('\n')))],
            ["\\", "\n"]
        }),

        (T_EOL = {
            ["\n", _, (Op::Skip)],
            [";", _, (Op::Skip)]
        }),

        // Prime Tokens (might probably be replaced by something native, pluggable one)

        (T_Identifier = {
            [
                (token (Token::Char(ccl!['A'..='Z', 'a'..='z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call collect[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_Consumable = {
            [
                (token (Token::Char(ccl!['A'..='Z', '_'..='_']))),
                (opt (token (Token::Chars(ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])))),
                (call collect[(value "identifier"), (Op::LoadFastCapture(0))])
            ]
        }),

        (T_String = {
            [
                "\"",
                (token (Token::chars_until('\"'))),     //fixme: Escape sequences (using Until built-in parselet)
                "\""
            ]
        }),

        (T_Match = {
            [
                "\'",
                (token (Token::chars_until('\''))),    //fixme: Escape sequences (using Until built-in parselet)
                "\'"
            ]
        }),

        (T_Integer = {
            // todo: implement as built-in Parselet
            [(token (Token::Chars(ccl!['0'..='9']))), (call collect[(value "value_integer")])]
        }),

        (T_Float = {
            // todo: implement as built-in Parselet
            [(token (Token::Chars(ccl!['0'..='9']))), ".", (opt (token (Token::Chars(ccl!['0'..='9'])))),
                (call collect[(value "value_float"), (Op::LoadFastCapture(0))])],
            [(opt (token (Token::Chars(ccl!['0'..='9'])))), ".", (token (Token::Chars(ccl!['0'..='9']))),
                (call collect[(value "value_float"), (Op::LoadFastCapture(0))])]
        }),

        // Character classes

        (CclChar = {
            [EOF, (call error[(value "Unclosed character-class, expecting ']'")])],
            ["\\", (token (Token::Any))],
            (token (Token::char_except(']')))
        }),

        (CclRange = {
            [CclChar, "-", CclChar,
                (call collect[(value "range"), [(Op::LoadFastCapture(1)), (Op::LoadFastCapture(3)), (Op::Add)]])],
            [CclChar, (call collect[(value "char")])]
        }),

        (Ccl = {
            ['^', (kle CclRange), (call collect[(value "ccl_neg")])],
            [(kle CclRange), (call collect[(value "ccl")])]
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
            ["--", Lvalue, (call collect[(value "inplace_pre_dec")])],
            Variable
        }),

        (Rvalue = {
            [Inplace, _, (kle Tail), (call collect[(value "rvalue")])]
        }),

        (CallParameter = {
            [T_Identifier, _, "=", _, Expression, (call collect[(value "param_named")])],
            [Expression, (call collect[(value "param")])]
        }),

        (CallParameters = {
            (pos [CallParameter, (opt [",", _])])
        }),

        (Call = {
            [T_Identifier, "(", _, (opt CallParameters), (expect ")"), _,
                (call collect[(value "call_identifier")])]
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

        // Tokens

        (TokenLiteral = {
            ["'", T_Match, "'", (call collect[(value "value_token_match")])],
            [T_Match, (call collect[(value "value_token_touch")])],
            [".", (call collect[(value "value_token_any")])],
            ['[', Ccl, ']', (call collect[(value "value_token_ccl")])]
        }),

        (TokenCall = {
            TokenLiteral,
            [T_Consumable, "(", _, (opt CallParameters), (expect ")"),
                (call collect[(value "call_identifier")])],
            [T_Consumable, (call collect[(value "rvalue")])]
        }),

        (Token = {
            // Token call modifiers
            [TokenCall, "+", _, (call collect[(value "op_mod_pos")])],
            [TokenCall, "*", _, (call collect[(value "op_mod_kle")])],
            [TokenCall, "?", _, (call collect[(value "op_mod_opt")])],
            // todo: {min}, {min, max} maybe with expression?
            [TokenCall, _],
            ["peek", _, (expect Token, "Token"), (call collect[(value "op_mod_peek")])],
            ["not", _, (expect Token, "Token"), (call collect[(value "op_mod_not")])],
            ["expect", _, (expect Token, "Token"), (call collect[(value "op_mod_expect")])]
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
            ["-", _, Unary, (call collect[(value "op_unary_neg")])],
            ["!", _, Unary, (call collect[(value "op_unary_not")])],
            Atomic
        }),

        // todo: & and |

        (MulDiv = {
            [MulDiv, "*", _, (expect Unary), (call collect[(value "op_binary_mul")])],
            [MulDiv, "/", _, (expect Unary), (call collect[(value "op_binary_div")])],
            // todo: ^ (pow)
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

        // todo: && and ||

        (Assign = {
            [Lvalue, "=", _, Expression, (call collect[(value "assign")])] // fixme: a = b = c is possible here...
            // todo: add operators "+="", "-="", "*="", "/=" here as well
        }),

        (Expression = {
            // if
            ["if", _, Expression, Statement, "else", _, Statement, (call collect[(value "op_ifelse")])],
            ["if", _, Expression, Statement, (call collect[(value "op_if")])],
            ["if", _, (call error[(value "'if': Expecting condition and statement")])],

            // while
            ["while", _, Expression, (kle T_EOL), Statement, (call collect[(value "op_while")])],
            ["while", _, (call error[(value "'while': Expecting end-condition and statement")])],

            // for
            ["for", _, T_Identifier, _, "in", _, Expression, Statement, (call collect[(value "op_for_in")])],
            ["for", _, StatementOrVoid, ";", _, StatementOrVoid, ";", _, StatementOrVoid, StatementOrVoid, (call collect[(value "op_for")])],
            ["for", _, (call error[(value "'for': Expecting start; condition; iter; statement")])],

            // normal comparison
            Compare
        }),

        (StatementOrVoid = {
            Statement,
            (call collect[(value "value_void")])
        }),

        (Statement = {
            ["return", _, Expression, (call collect[(value "op_return")])],
            ["return", _, (call collect[(value "op_returnvoid")])],
            ["accept", _, Expression, (call collect[(value "op_accept")])],
            ["accept", _, (call collect[(value "op_acceptvoid")])],
            ["reject", _, (call collect[(value "op_reject")])],
            // todo: report, escape, repeat
            Assign,
            Expression
        }),

        // Parselet

        (Argument = {
            [T_Identifier, _, (opt ["=", _, (opt Expression)]), (call collect[(value "arg")])]
        }),

        (Arguments = {
            (pos [Argument, (opt [",", _])])
        }),

        (Parselet = {
            ["@", _, (opt Arguments), Block, (call collect[(value "value_parselet")])],
            ["@", _, (opt Arguments), Token, (call collect[(value "value_parselet")])]
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

            [T_Identifier, _, ":", _, (expect Expression), (expect T_EOL),
                (call collect[(value "assign_constant")])],
            [(pos Item), (call collect[(value "sequence")])],
            [T_EOL, (Op::Skip)]
        }),

        (Item = {
            // todo: Recognize aliases
            Statement
        }),

        (Tokay = {
            Sequences
        }),

        [_, Tokay, (call collect[(value "main")])]

        // ----------------------------------------------------------------------------
                    }))
    }

    pub fn parse(&self, mut reader: Reader) -> Result<Value, Error> {
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
}
