use ::tokay::reader::Reader;
use ::tokay::tokay::{State, Program, Token, CallBy, Runtime};
use ::tokay::{ccl, tokay, token};
use ::tokay::value::Value;


fn main() {
    let s = "42+3-1337/3*2  hello 7*(2+5) world  666-600 3".to_string();
    //let s = "a(1+2)b".to_string();
    //let s = "1+2+3";
    //let s = "23".to_string();
    println!("{}", s);

    let reader = Reader::new(
        std::io::Cursor::new(s)
    );

    let mut program = Program::new();

    tokay!(
        &mut program,

        main {
            => (expr)
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
                        State::Accept(Some(Value::Integer(i).into_ref()))
                    }
                    else {
                        State::Reject
                    }
                })
        }
    );

    program.finalize();

    /*
    for (i, p) in program.parselets.iter().enumerate() {
        println!("{} => {:?}", i, p);
    }
    */

    let mut runtime = Runtime::new(reader);

    while !runtime.is_eof() {
        if let State::Accept(result) = program.run(&mut runtime, 0) {
            println!("match {:?}", result);
            //runtime.stats();
        } else {
            program.skip(&mut runtime, 0);
            runtime.clean();
        }
    }
}
