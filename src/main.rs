use std::fs::File;
use std::io::{self, Write};

use ::tokay::compiler::Compiler;
use ::tokay::parser::Parser;
use ::tokay::reader::Reader;
use ::tokay::value::*;

#[cfg(test)]
use ::tokay::value; //for the value!-macro

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
fn compile_and_run(
    src: &'static str,
    input: &'static str,
    debug: bool,
) -> Result<Option<RefValue>, Option<String>> {
    let p = Parser::new();
    let ast = p.parse(Reader::new(Box::new(io::Cursor::new(src))));

    if let Ok(ast) = ast {
        if debug {
            println!("ast = {:#?}", ast);
        }

        let mut compiler = Compiler::new();
        compiler.traverse(&ast);

        let prg = compiler.into_program();
        if debug {
            println!("prg = {:#?}", prg);
        }

        prg.run_from_str(input)
    } else {
        Err(ast.err())
    }
}

#[test]
fn test_literal() {
    assert_eq!(
        compile_and_run(
            "\
            1337 \
            23.5 \
            true \
            false \
            \"Hello World\" \
            ",
            "",
            false
        ),
        Ok(Some(value!([1337, 23.5, true, false, "Hello World"])))
    );
}

#[test]
fn test_expression() {}

#[test]
fn test_parselet_leftrec() {
    assert_eq!(
        compile_and_run("P: @{ P ''Hello''}\nP", "HelloHelloHelloHello", false),
        Ok(Some(value!([[["Hello", "Hello"], "Hello"], "Hello"])))
    );

    assert_eq!(
        compile_and_run(include_str!("../readme.tok"), "1+2*3+4", false),
        Ok(Some(value!(11)))
    );
}

#[test]
fn test_capture() {
    assert_eq!(
        compile_and_run("'Hello' 'World' $1 * 2 + $2 * 3", "HelloWorld", false),
        Ok(Some(value!("HelloHelloWorldWorldWorld")))
    );

    assert_eq!(
        compile_and_run(
            "a=2 'Hello' 'World' $(a + 1) * 3+ $(a) * 2",
            "HelloWorld",
            false
        ),
        Ok(Some(value!("WorldWorldWorldHelloHello")))
    );
}

#[test]
fn test_begin_end() {
    assert_eq!(
        compile_and_run(
            "
            begin { x = 0 1337 }
            end 1338

            P: @{ 'lol' x = x + 1 x }
            P",
            "lolalolaalolol",
            false
        ),
        Ok(Some(value!([1337, 1, 2, 3, 1338])))
    );

    assert_eq!(
        compile_and_run(
            "
            begin x = 1

            'lol' $1 * x x x = x + 1",
            "lolAlolBlol",
            false
        ),
        Ok(Some(value!([["lol", 1], ["lollol", 2], ["lollollol", 3]])))
    );
}

// todo: turn the examples below into a test suite
//let s = "P = @{\nP? 'Hello'\nP? \"World\"}\nP";
//let s = "P = @{\nP? \"Hello\"\nP? \"World\"}\nP";
//let s = "A = @{ \"Hello\"+ B* (1337.+-3) (+true) { if a == b + 1 c else d } }";
//let s = "A B C\nX Y Z";
//let s = "x = @{return0}";
//let s = "a = 42 a a + 1 a + 2";
//let s = "A = 'Hello' A+ 3 + 2* ('Bernd Waldemar')";

// Capture modification
//let s = "a = 2 'Hello' 'World' $2 = $3 * 2 + $2 $3";
//let s = "a = 2 'Hello' 'World' $(a) = $3 * 2 + $2 $3 * 2";

// Comparisons
//let s = "'Hello' 'World' $1 == $2";

//let s = "P = @{ P 'A' 'B' $2 * 2 + $3 * 3 }\nP";
//let s = "a:'Hello' a\na : 'Hallo' A";

// A first simple REPL for Tokay
fn repl(debug: bool) {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut code = String::new();
        if io::stdin().read_line(&mut code).is_err() {
            panic!("Error reading code")
        }

        // Stop when program is empty.
        if code.trim().is_empty() {
            return;
        }

        //println!("code = {:?}", code);

        let parser = Parser::new();
        let ast = parser.parse(Reader::new(Box::new(io::Cursor::new(code))));

        if debug {
            println!("ast = {:#?}", ast);
        }

        match ast {
            Ok(Value::Void) => {}

            Ok(ast) => {
                let mut compiler = Compiler::new();

                compiler.traverse(&ast);
                let prg = compiler.into_program();

                if debug {
                    println!("prg = {:#?}", prg);
                }

                if std::env::args().len() == 1 {
                    println!("<<< {:?}", prg.run_from_str(""));
                } else {
                    for filename in std::env::args().skip(1) {
                        let file = File::open(&filename).unwrap();
                        println!("{}: {:?}", filename, prg.run_from_reader(file));
                    }
                }
            }

            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

fn main() {
    println!("Tokay v{}", VERSION);
    repl(false);

    /*
    println!(
        "{:#?}",
        /*
        compile_and_run(
        "
            >> x=1
            'Hallo' $1 x x = x + 1
        ",
            "HalloHallololHallo",
            true
        )
        */
        compile_and_run(
            "
            hw : @{'hello' 'world'}
            hw
            Integer
            ",
            " 123 helloworldworldworld 456",
            true
        )
    );
    */
}
