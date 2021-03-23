use std::io;

use ::tokay::compiler::Compiler;
use ::tokay::error::Error;
use ::tokay::reader::Reader;
use ::tokay::repl::repl;
use ::tokay::value;
use ::tokay::value::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

//#[cfg(test)]
fn compile_and_run(
    src: &'static str,
    input: &'static str,
    debug: bool,
) -> Result<Option<RefValue>, Error> {
    let mut compiler = Compiler::new();
    compiler.debug = debug;

    if let Some(program) = compiler.compile(Reader::new(Box::new(io::Cursor::new(src)))) {
        program.run_from_str(input)
    } else {
        Err(Error::new(None, "Error during compilations".to_string()))
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
fn test_expression() {
    assert_eq!(
        compile_and_run("1 + 2 * 3 + 4", "", false),
        Ok(Some(value![11]))
    );
    //assert_eq!(compile_and_run("1 + 2 == 3", "", false), Ok(Some(value![true])));
    // todo: more expressions, please!
}

#[test]
fn test_token() {
    assert_eq!(
        compile_and_run("'a' 'b'", "ab", false),
        Ok(Some(value![["a", "b"]]))
    );

    assert_eq!(
        compile_and_run("'a' ''b''", "ab", false),
        Ok(Some(value!["b"]))
    );

    assert_eq!(
        compile_and_run("'a' pos ''b''", "abbb", false),
        Ok(Some(value![["b", "b", "b"]]))
    );

    assert_eq!(
        compile_and_run("kle ''a'' pos ''b''", "aabbb", false),
        Ok(Some(value![[["a", "a"], ["b", "b", "b"]]]))
    );

    assert_eq!(
        compile_and_run("kle 'a' pos ''b''", "bbb", false),
        Ok(Some(value![["b", "b", "b"]]))
    );

    /*
    assert_eq!(
        compile_and_run("opt 'a' pos ''b''", "aabbb", false),
        Ok(None)
    );

    assert_eq!(
        compile_and_run("not 'Hello' 'World' accept; reject", "HelloWorld", false),
        Ok(None)
    );
    */

    // todo: more token tests, please!
}

#[test]
fn test_parselet_static_with_args() {
    assert_eq!(
        compile_and_run(
            "
            faculty : @x {
                if !x return 1
                x * faculty(x - 1)
            }

            faculty(4)
            ",
            "",
            false
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
fn test_parselet_variable_with_args() {
    assert_eq!(
        compile_and_run(
            "
            faculty = @x {
                if !x return 1
                x * faculty(x - 1)
            }

            faculty(4)
            ",
            "",
            false
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
fn test_parselet_leftrec() {
    assert_eq!(
        compile_and_run("P: @{ P ''a''}\nP", "aaaa", false),
        Ok(Some(value!([[["a", "a"], "a"], "a"])))
    );
}

#[test]
fn test_readme_examples() {
    assert_eq!(
        compile_and_run(include_str!("../readme.tok"), "1+2*3+4", false),
        Ok(Some(value!(11)))
    );

    assert_eq!(
        compile_and_run(include_str!("../faculty.tok"), "", false),
        Ok(Some(value!(24)))
    );
}

#[test]
fn test_capture() {
    assert_eq!(
        compile_and_run("'a' 'b' $1 * 2 + $2 * 3", "ab", false),
        Ok(Some(value!("aabbb")))
    );

    assert_eq!(
        compile_and_run("a=2 'a' 'b' $(a + 1) * 3+ $(a) * 2", "ab", false),
        Ok(Some(value!("bbbaa")))
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

fn main() {
    println!("Tokay v{}", VERSION);
    repl();

    /*
    let ast = compile_and_run(
        include_str!("../readme1.tok"),
        "1+2*3+4",
        true,
    ).unwrap().unwrap();

    Compiler::print(
        &ast.borrow()
    );
    */

    /*
    println!(
        "{:?}",
        compile_and_run(
            "
                print(\"Hello World\" + 23 * 4)
                Integer print(\"Have \" + $1)
            ",
            "yay42",
            true,
        )
    );
    */

    /*
    println!("{:#?}", compile_and_run(
        "
        TheTokaySayer : @Match: result:\"TOKAY!!!\" {
            Match result + \": \" + $1
        }

        TheTokaySayer(Integer)
        TheTokaySayer(pos 'Hello')
        ",
        "123HelloHelloello456Hello", true
    ));
    */

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
