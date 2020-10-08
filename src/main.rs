use ::tokay::reader::Reader;
use ::tokay::tokay::*;
use ::tokay::token::*;
use ::tokay::value::Value;
use ::tokay::compiler::Compiler;
use ::tokay::{tokay, tokay_item, ccl};


fn main() {
    let s = "1 + 2 * 3 + 4 ".to_string();
    //let s = "HelloWorldblablabla".to_string();
    println!("{}", s);

    /*
    let program = tokay!({
        (A = "1"),
        [A]
    });
    */

    let program = tokay!({
        (_ = {
            ' '
        }),

        (Factor = {
            ['(', _, Expr, ')', _],
            [Int]
        }),

        (Term = {
            [Term, "*", _, Factor],
            [Term, "/", _, Factor],
            [Factor]
        }),

        (Expr = {
            [Expr, "+", _, Term],
            [Expr, "-", _, Term],
            [Term]
        }),

        (Int = {
            [
                (Repeat::new(
                    Item::Token(Char::new(ccl!['0'..='9'])).into_box(), 1, 0)
                        .into_box()
                ),
                _
            ]
                /*
                (|runtime| {
                    //println!("{:?}", runtime.get_capture(0));

                    if let Some(i) = runtime.get_capture(1).unwrap().borrow().to_integer() {
                        Ok(Accept::Return(Some(Value::Integer(i).into_ref())))
                    }
                    else {
                        Err(Reject::Return)
                    }
                })
                */
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

    //program.dump();
    println!("program = {:#?}", program);
    
    //let s = "42+3-1337/3*2  helloworldworldworldhellohelloworld 7*(2+5) world  666-600 3".to_string();
    let mut reader = Reader::new(
        Box::new(std::io::Cursor::new(s))
    );

    let mut runtime = Runtime::new(&program, &mut reader);
    let ret = program.run(&mut runtime);

    println!("{:?}", ret);
    runtime.dump();
}
