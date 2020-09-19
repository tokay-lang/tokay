use std::cell::RefCell;

use std::io::prelude::*;
use ::tokay::reader::Reader;
use ::tokay::tokay::*;
use ::tokay::token::*;
use ::tokay::ccl;
use ::tokay::value::Value;


fn main() {
    let s = "42+3-1337/3*2  helloworldworldworldhellohelloworld 7*(2+5) world  666-600 3".to_string();
    let s = "HelloWorld".to_string();
    //let s = "a(1+2)b".to_string();
    //let s = "1+2+3";
    //let s = "23".to_string();
    println!("{}", s);

    let mut reader = Reader::new(
        Box::new(std::io::Cursor::new(s))
    );

    let program = Program::new();
    let mut runtime = Runtime::new(&program, &mut reader);

    let p = Parselet::new(
        Op::Block(vec![
            Op::Sequence(vec![(Op::Token(Match::new("Hello")), None), (Op::Token(Match::new("World")), None)]),
            //Op::Sequence(vec![(Op::Token(Match::new("Hello")), None), (Op::Token(Match::new("Trorld")), None)])
        ])
    );
    let ret = p.run(&mut runtime);

    println!("{:?}", ret);

    /*
    let mut program = Program::new();

    //trace_macros!(true);
    //sequence!(&mut program, [(positive("hello"))]);

    tokay!(
        &mut program,

        main {
            => (expr)
            => (("hello") ((kle("world")) (|runtime| {
                let hello = runtime.get_capture(1).unwrap().borrow().to_string().unwrap();
                let world = runtime.get_capture(2).unwrap().borrow().to_string().unwrap();
        
                println!("{} {} {}", runtime.get_capture(0).unwrap().borrow().to_string().unwrap(), hello, world);
                Return::Accept(None)
            })))
        }

        factor {
            => ("(") (expr) (")")
            => (int)
        }

        term {
            => (term) ("*") (factor)
            => (term) ("/") (factor)
            => (factor)
        }

        expr {
            => (expr) ("+") (term)
            => (expr) ("-") (term)
            => (term)
        }

        int {
            =>  (Token::Chars(ccl!['0'..='9']))
                (|runtime| {
                    //println!("{:?}", runtime.get_capture(0));

                    if let Some(i) = runtime.get_capture(1).unwrap().borrow().to_integer() {
                        Return::Accept(Some(Value::Integer(i).into_ref()))
                    }
                    else {
                        Return::Reject
                    }
                })
        }
    );
    //trace_macros!(false);

    program.finalize();

    /*
    for (i, p) in program.parselets.iter().enumerate() {
        println!("{} => {:?}", i, p);
    }
    */

    let mut runtime = Runtime::new(reader);

    while !runtime.is_eof() {
        if let Return::Accept(result) = program.run(&mut runtime) {
            println!("match {:?}", result);
            //runtime.stats();
        } else {
            runtime.skip();
        }
    }
    */
}
