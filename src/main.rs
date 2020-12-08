use std::io::{self, Write, stdout, stdin};

use ::tokay::reader::Reader;
use ::tokay::tokay::*;
use ::tokay::parser::TokayParser;
use ::tokay::value::Value;
use ::tokay::compiler::{Compiler, TokayCompiler};
use ::tokay::{tokay, tokay_item, ccl};


fn traverse(value: &Value) -> Option<Op> {
    let value = value.get_complex().unwrap();
    let emit = value.get_by_key("emit").unwrap().borrow();
    let key = emit.get_string().unwrap();

    println!("{}", key);
    /*

    if emit == "tokay1" {
        /*
        let nodes = payload.get_complex().unwrap();

        for i in 0..nodes.len() {
            traverse(&nodes[i].borrow());
        }
        */
    }
    else if emit == "assign_constant" {
        /*
        let payload = payload.get_complex().unwrap();

        traverse(&payload[1].borrow());
        let name = payload[0].borrow().get_complex().unwrap()[1].borrow();
        println!("LET CONSTANT {:?}", name);
        */
    }
    else {
        println!("emit = {:?}", emit);
        //println!("emit = {:?} payload = {:?}", emit, payload);
    }
    */

    None
}


fn main() {
    /*
    let s = "123 + 456 * 789 + (4)  (99 - 3)*99 + 4".to_string();
    //let s = "HelloWorldblablabla".to_string();
    println!("{}", s);

    let calculator = tokay!({
        (_ = {
            " "
        }),

        (Factor = {
            ["(", _, Expr, ")", _],
            [Int]
        }),

        (Term = {
            [Term, "*", _, Factor, (Op::Create("mul"))],
            [Term, "/", _, Factor, (Op::Create("div"))],
            [Factor]
        }),

        (Expr = {
            [Expr, "+", _, Term, (Op::Create("add"))],
            [Expr, "-", _, Term, (Op::Create("sub"))],
            [Term]
        }),

        (Int = {
            [
                (Char::span(ccl!['0'..='9'])),
                _,
                (Rust(|context: &mut Context| {
                    if let Some(i) = context.get_capture(1).unwrap().borrow().to_integer() {
                        Ok(Accept::Return(Some(Value::Integer(i).into_ref())))
                    }
                    else {
                        Err(Reject::Return)
                    }
                }).into_op())
            ]
        }),


        [Expr]
        /*
        => (("hello") ((kle("world")) (|runtime| {
            let hello = runtime.get_capture(1).unwrap().borrow().to_string().unwrap();
            let world = runtime.get_capture(2).unwrap().borrow().to_string().unwrap();

            println!("{} {} {}", runtime.get_capture(0).unwrap().borrow().to_string().unwrap(), hello, world);
            Ok(Accept::Next)
        })))
        */
    });
    */

    let test = tokay!({
        (a = "hallo"),
        (b = 31),
        (x = {
            ["Helloween", a, b, (Op::Print)]
        }),
        [x, x, a, b, (Op::Print)]
    });

    println!("{:?}", test);

    let mut reader = Reader::new(
        Box::new(io::Cursor::new("HelloweenHelloween".to_string()))
    );
    let mut runtime = Runtime::new(&test, &mut reader);
    let ret = test.run(&mut runtime);
    println!("{:?}", ret);

    //trace_macros!(true);

    /*
    let mut program = tokay!({
        (Main = {
            (A = {
                ["Hello"],
                [B]
            }),
            (B = {
                ["World"],
                [A]
            })
        }),
        (A = {
            ["Trollo"]
        }),

        [Main, A]
    });
    */

    //trace_macros!(false);

    //let s = "42+3-1337/3*2  helloworldworldworldhellohelloworld 7*(2+5) world  666-600 3".to_string();
    /*
    let mut reader = Reader::new(
        Box::new(io::Cursor::new(s))
    );

    //program.dump();
    for program in &[calculator] {
        //println!("program = {:#?}", program);

        reader.reset(0);

        let mut runtime = Runtime::new(&program, &mut reader);
        let ret = program.run(&mut runtime);

        println!("{:#?}", ret);
        runtime.dump();
    }
    */

    let p = TokayParser::new();
    let s = include_str!("../readme.tok");
    let s = "1+2*3+4";
    //let s = "A\nB\nC\n";
    //let s = "A = @{ \"Hello\"+ B* (1337.+-3) (+true) { if a == b + 1 c else d } }";
    //let s = "A B C\nX Y Z";
    //let s = "x = @{return0}";

    println!("{}", s);

    let res = p.parse(
        Reader::new(Box::new(io::Cursor::new(s)))
    );

    if let Ok(ast) = res {
        println!("{:?}", ast);

        let compiler = TokayCompiler::new();
        compiler.traverse(&ast.get_complex().unwrap());
    }

    /*
    print!("tokay>");
    stdout().flush();

    let res = p.parse(
        Reader::new(Box::new(io::BufReader::new(stdin())))
    );
    */
}
