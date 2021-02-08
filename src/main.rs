use std::io;

use ::tokay::value::{Value, RefValue, Dict, List};
use ::tokay::reader::Reader;
use ::tokay::parser::Parser;
use ::tokay::compiler::Compiler;
use ::tokay::value;


fn compile_and_run(src: &'static str, input: &'static str, debug: bool)
    -> Result<Option<RefValue>, Option<String>>
{
    let p = Parser::new();
    let ast = p.parse(
        Reader::new(Box::new(io::Cursor::new(src)))
    );

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
    }
    else {
        Err(ast.err())
    }
}

#[test]
fn test_parselet_simple() {
    assert_eq!(
        compile_and_run(
            "P: @{\nP \"Hello\"}\nP",
            "HelloHelloHelloHello",
            false
        ),
        Ok(Some(value!([[["Hello", "Hello"], "Hello"], "Hello"])))
    );
}

#[test]
fn test_capture_loading() {
    assert_eq!(
        compile_and_run(
            "'Hello' 'World' $1 * 2 + $2 * 3",
            "HelloWorld",
            false
        ),
        Ok(Some(value!("HelloHelloWorldWorldWorld")))
    );

    assert_eq!(
        compile_and_run(
            "a = 2 'Hello' 'World' $( a + 1 ) * 3 + $(a) * 2",
            "HelloWorld",
            false
        ),
        Ok(Some(value!("WorldWorldWorldHelloHello")))
    );
}

#[test]
fn test_readme_tok() {
    assert_eq!(
        compile_and_run(
            include_str!("../readme.tok"),
            "1+2*3+4",
            false
        ),
        Ok(Some(value!(11)))
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


fn main() {
    println!("{:#?}",
        compile_and_run(
            "P: @{\nP \"Hello\"}\nP",
            "HelloHelloHelloHello",
            true
        )
    );

    println!("{:#?}", value!([[["Hello", "Hello"], "Hello"], "Hello"]));
}
