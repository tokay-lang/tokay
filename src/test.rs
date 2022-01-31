//! Unit tests

use crate::utils::*;
use crate::value;
use crate::value::*;

//use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

// Universal function to run test-cases from files with expected output.
fn run_testcase(filename: &'static str) {
    let mut code = String::new();

    match File::open(filename) {
        // The file is open (no error).
        Ok(mut file) => {
            // Read all the file content into a variable (ignoring the result of the operation).
            file.read_to_string(&mut code).unwrap();

            // The file is automatically closed when is goes out of scope.
        }
        // Error handling.
        Err(error) => {
            panic!("Error opening file {}: {}", filename, error);
        }
    }

    //println!("code = {:?}", code);

    if let Some((_, data)) = code.split_once("#---\n") {
        let mut params = vec![filename];
        let mut result = data;
        let tmp;

        // In case there is another separator, split into input and output
        if let Some((i, o)) = data.split_once("#---\n") {
            let input: Vec<&str> = i
                .split("\n")
                .filter(|line| line.starts_with("#"))
                .map(|line| &line[1..])
                .collect();

            tmp = input.join("\n");
            params.extend(vec!["--", &tmp]);
            result = o;
        }

        //let program = env::args().next().unwrap(); // Doens't work with cargo test
        let program = "target/debug/tokay";

        let output = Command::new(program)
            .args(&params)
            //.env("LS_COLORS", "rs=0:di=38;5;27:mh=44;38;5;15")
            .output()
            .expect(&format!(
                "Failed to run '{} {}'; You have to `cargo run` first!",
                program, filename
            ));

        let mut out: Vec<String> = String::from_utf8(output.stdout)
            .expect("Not UTF-8")
            .split("\n")
            .map(|line| line.to_string())
            .collect();
        let mut err: Vec<String> = String::from_utf8(output.stderr)
            .expect("Not UTF-8")
            .split("\n")
            .map(|line| line.to_string())
            .collect();

        println!("out = {:?}", out);
        println!("err = {:?}", err);

        for line in result.trim().split("\n").into_iter() {
            assert!(
                line.starts_with("#"),
                "Lines in result must start with a #-comment"
            );

            if line.starts_with("#ERR:") {
                assert_eq!(err.remove(0), line[5..].to_string());
            } else {
                assert_eq!(out.remove(0), line[1..].to_string());
            }
        }

        assert!(out.len() == 1, "Some output not consumed: {:?}", out);
        assert!(err.len() == 1, "Some errors not consumed: {:?}", err);
    } else {
        panic!("Testcase invalid, require for a '#---' delimiter.")
    }
}

// Tests expression basics ------------------------------------------------------------------------

#[test]
// Test for literals
fn literal() {
    assert_eq!(
        compile_and_run(
            "\
            1337 \
            23.5 \
            true \
            false \
            \"Hello World\" \
            ",
            ""
        ),
        Ok(Some(value!([1337, 23.5, true, false, "Hello World"])))
    );
}

#[test]
// Test for expressions
fn expression() {
    // Simple Integer expressions
    assert_eq!(
        compile_and_run(
            "\
            1 + 2 \
            100 - 99 \
            7 * 3 \
            21 / 3 \
            1 + 2 * 3 + 4 \
            ",
            ""
        ),
        Ok(Some(value!([3, 1, 21, 7, 11])))
    );

    // Simple Float expressions
    assert_eq!(
        compile_and_run(
            "\
            .2  + .3 \
            1.9 - 1.1 \
            2.1 * 6.5 \
            13.6 / 15.1 \
            1.1 + 2.2 * 3.3 + 4.4 \
            ",
            ""
        ),
        Ok(Some(value!([
            0.5,
            0.7999999999999998,
            13.65,
            0.9006622516556292,
            12.76
        ])))
    );

    // Simple String expressions
    assert_eq!(
        compile_and_run(
            "\
            \"a\" + \"b\" \
            \"a\" + 2 \
            \"a\" + 1.337 \
            \"a\" + \"b\" * 3 \
            \"a\" * 9 \
            \"a\" * 3.3 \
            ",
            ""
        ),
        Ok(Some(value!([
            "ab",
            "a2",
            "a1.337",
            "abbb",
            "aaaaaaaaa",
            "aaa"
        ])))
    );

    // Equality Comparisons
    assert_eq!(
        compile_and_run(
            "\
            x = 42 \
            42 == 42 \
            x == 42 \
            x == x \
            42 != 42 \
            x != 42 \
            x != x \
            42 != 23 \
            42 == 23 \
            1.3 != 3.7 \
            1.3 == 3.7 \
            1.12345 == 1.12345 \
            1.12345 != 1.12345 \
            \"a\" == \"a\" \
            \"a\" != \"a\" \
            \"a\" != \"a\" * 2 \
            \"a\" == \"a\" * 2 \
            ",
            ""
        ),
        Ok(Some(value!([
            true, true, true, false, false, false, true, false, true, false, true, false, true,
            false, true, false
        ])))
    );

    // Ordered Comparisons
    assert_eq!(
        compile_and_run(
            "\
            42 >= 42 \
            42 <= 42 \
            42 < 42 \
            42 > 42 \
            \
            42.23 >= 42.23 \
            42.23 <= 42.23 \
            42.23 < 42.23 \
            42.23 > 42.23 \
            \
            \"42.23\" >= 42.23 \
            \"42.23\" <= 42.23 \
            \"42.23\" < 42.23 \
            \"42.23\" > 42.23 \
            ",
            ""
        ),
        Ok(Some(value!([
            true, true, false, false, true, true, false, false, true, false, false, true
        ])))
    );

    // Logical AND and OR
    assert_eq!(
        compile_and_run(
            "\
            a = true
            b = false
            \
            a && a \
            a && b \
            a || b \
            b || a \
            \
            b || a && a \
            b || b && a \
            b && b
            ",
            ""
        ),
        Ok(Some(value!([true, false, true, true, true, false, false])))
    );

    // Unary operations
    assert_eq!(
        compile_and_run(
            "\
            -1 \
            (-(-1)) \
            !true \
            !!true
            ",
            ""
        ),
        Ok(Some(value![[(-1), 1, false, true]]))
    );
}

#[test]
// Test for operations
fn operations() {
    // Test assignment-operations
    assert_eq!(
        compile_and_run(
            "
            a = true a \
            a + 2 == 3 \
            a += 30 a \
            a -= 9 a \
            a *= 3 a \
            a /= 6 a \
            a /= 2 a \
            a *= 10 a
            ",
            ""
        ),
        Ok(Some(value![[true, true, 31, 22, 66, 11, 5.5, 55.0]]))
    );

    // Tests for pre- and post-increment and -decrements
    // These require spaces in some situations to find correct path throug meaning
    assert_eq!(
        compile_and_run(
            "
            a = 1 \
            a \
            a++ \
            ++a \
            a++ a++ \
            a++ + ++a \
            a-- - --a \
            a-- - - --a \
            a",
            ""
        ),
        Ok(Some(value![[1, 1, 3, 3, 4, 12, 2, 8, 3]]))
    );
}

#[test]
// Test for variable store/hold cases
fn variables() {
    // Test store-hold-global
    assert_eq!(
        compile_and_run(
            "
            a = b = 10
            a++
            a b
            ",
            ""
        ),
        Ok(Some(value![[11, 10]]))
    );

    // Test store-hold-local
    assert_eq!(
        compile_and_run(
            "
            f : @{
                a = b = 10
                a++
                a b
            }

            f
            ",
            ""
        ),
        Ok(Some(value![[11, 10]]))
    );

    // Test store-hold-capture
    assert_eq!(
        compile_and_run(
            "
            10 20 $1 = $2 = 30 ++$1 $2
            ",
            ""
        ),
        Ok(Some(value![[31, 30, 31, 30]]))
    );

    // Test store-hold-aliased-capture
    assert_eq!(
        compile_and_run(
            "
            a => 10 b => 20 $a = $b = 30 c => ++$a d => $b
            ",
            ""
        ),
        Ok(Some(value![["a" => 31, "b" => 30, "c" => 31, "d" => 30]]))
    );
}

#[test]
// Test-case for scoping
fn scoping() {
    assert_eq!(
        compile_and_run(include_str!("../tests/test_scopes.tok"), ""),
        Ok(Some(value![[10, 2000, 1072]]))
    );
}

// Tests for dicts and lists ----------------------------------------------------------------------

#[test]
// Test for collection (list, dict) parsing
fn collections() {
    // Lists
    assert_eq!(
        compile_and_run(
            "
            (1 2 3) \
            (1, 2, 3) \
            (42, \"Hello\", 23.5, true, false)
            ",
            ""
        ),
        Ok(Some(value!([
            [1, 2, 3],
            [1, 2, 3],
            [42, "Hello", 23.5, true, false]
        ])))
    );

    // Dicts
    assert_eq!(
        compile_and_run(
            "
            x = 10
            (a => 1 b => 2 c => 3) \
            (a => 1, b => 2, c => 3) \
            (a => 42, x * 2 => \"Hello\", c => 23.5)
            ",
            ""
        ),
        Ok(Some(value!([
            ["a" => 1, "b" => 2, "c" => 3],
            ["a" => 1, "b" => 2, "c" => 3],
            ["a" => 42, "20" => "Hello", "c" => 23.5]
        ])))
    );
}

// Tests for tokens -------------------------------------------------------------------------------

#[test]
// Test for token modifiers
fn token_modifiers() {
    let s = "ab abbb bb 123 ABC 456 'def'";

    // Simple touch
    assert_eq!(
        compile_and_run("'a' 'b'", s),
        Ok(Some(value![[["a", "b"], ["a", "b"]]]))
    );

    // Touch and match
    assert_eq!(
        compile_and_run("'a' ''b''", s),
        Ok(Some(value![["b", "b"]]))
    );

    // Match with positive modifier
    assert_eq!(
        compile_and_run("'a' ''b''+", s),
        Ok(Some(value![["b", ["b", "b", "b"]]]))
    );

    // Match with kleene and positive modifiers
    assert_eq!(
        compile_and_run("''a''* ''b''+", s),
        Ok(Some(value![[
            ["a", "b"],
            ["a", ["b", "b", "b"]],
            ["b", "b"]
        ]]))
    );

    // Touch with kleene and positive modifiers
    assert_eq!(
        compile_and_run("'a'* ''b''+", s),
        Ok(Some(value![["b", ["b", "b", "b"], ["b", "b"]]]))
    );

    // Character classes
    assert_eq!(
        compile_and_run("[a-z]", &s[..2]),
        Ok(Some(value![["a", "b"]]))
    );

    assert_eq!(
        compile_and_run("[a-z]+", s),
        Ok(Some(value![["ab", "abbb", "bb", "def"]]))
    );

    assert_eq!(
        compile_and_run("[^ ']+", s),
        Ok(Some(value![[
            "ab", "abbb", "bb", "123", "ABC", "456", "def"
        ]]))
    );

    // Built-in token
    assert_eq!(compile_and_run("Integer", s), Ok(Some(value![[123, 456]])));

    // Parsing with sequences and modifiers

    assert_eq!(
        compile_and_run("''a'' {''b'' ''c''}* ''d''", "abcbcd"),
        Ok(Some(value![["a", [["b", "c"], ["b", "c"]], "d"]]))
    );

    assert_eq!(
        compile_and_run("''a'' {''b'' ''c''}+ ''d''", "abcbcd"),
        Ok(Some(value![["a", [["b", "c"], ["b", "c"]], "d"]]))
    );

    assert_eq!(
        compile_and_run("''a'' {''b'' ''c''}* ''d''", "ad"),
        Ok(Some(value![["a", "d"]]))
    );

    assert_eq!(
        compile_and_run("''a'' {''b'' ''c''}+ ''d''", "ad"),
        Ok(None)
    );

    assert_eq!(
        compile_and_run("{ Word { ',' _ }? }+", "Hello,   World,  Beta,  Test"),
        Ok(Some(value![["Hello", "World", "Beta", "Test"]]))
    );

    // todo: more token tests, please!
}

#[test]
// Test for built-in tokens
fn builtin_tokens() {
    let gliders = "Glasflügel Libelle 201b\tG102 Astir  \nVentus_2cT";

    assert_eq!(
        compile_and_run("Identifier", gliders),
        Ok(Some(value!([
            "Glasflügel",
            "Libelle",
            "b",
            "G102",
            "Astir",
            "Ventus_2cT"
        ])))
    );

    assert_eq!(
        compile_and_run("Integer", gliders),
        Ok(Some(value!([201, 102, 2])))
    );

    assert_eq!(
        compile_and_run("Whitespaces", gliders),
        Ok(Some(value!([" ", " ", "\t", " ", "  \n"])))
    );

    assert_eq!(
        compile_and_run("Word", gliders),
        Ok(Some(value!([
            "Glasflügel",
            "Libelle",
            "b",
            "G",
            "Astir",
            "Ventus",
            "cT"
        ])))
    );

    // Builtin whitespace handling
    let abc = "abc   \tdef  abcabc= ghi abcdef";

    assert_eq!(
        compile_and_run("Word _; ", abc),
        Ok(Some(value![["abc", "def", "abcabc", "ghi", "abcdef"]]))
    );

    assert_eq!(
        compile_and_run("Word __; ", abc),
        Ok(Some(value![["abc", "def", "ghi"]]))
    );
}

// Tests for parselets ----------------------------------------------------------------------------

#[test]
// Testing static function with arguments
fn parselet_static_with_args() {
    assert_eq!(
        compile_and_run(
            "
            faculty : @x {
                if !x return 1
                x * faculty(x - 1)
            }

            faculty(4)
            ",
            ""
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
// Testing variable function with arguments
fn parselet_variable_with_args() {
    assert_eq!(
        compile_and_run(
            "
            faculty = @x {
                if !x return 1
                x * faculty(x - 1)
            }

            faculty(4)
            ",
            ""
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
// Testing left-recursive parselets
fn parselet_leftrec() {
    assert_eq!(
        compile_and_run("P: @{ P? ''a'' }\nP", "aaaa"),
        Ok(Some(value!([[["a", "a"], "a"], "a"])))
    );

    // todo: More examples here please!
}

#[test]
// Testing compile- and run-time error reporting
fn parselet_call_error_reporting() {
    // Tests for calling functions with wrong parameter counts
    for (call, msg) in [
        ("foo()", "Line 2, column 1: Call to unresolved symbol 'foo'"),
        (
            "f()",
            "Line 2, column 1: Call to 'f' doesn't accept any arguments",
        ),
        (
            "f(1, 2, 3, 4)",
            "Line 2, column 1: Too many parameters, 3 possible, 4 provided",
        ),
        ("f(c=10, d=3)", "Line 2, column 1: Parameter 'a' required"),
        (
            "f(1, c=10, d=3)",
            "Line 2, column 1: Parameter 'd' provided to call but not used",
        ),
    ] {
        let call = format!("f : @a, b=2, c {{ a b c }}\n{}", call);
        println!("calling {:?}, expecting {:?}", call, msg);

        assert_eq!(compile_and_run(&call, ""), Err(msg.to_owned()));
    }
}

#[test]
// Testing examples provided in the examples folder
fn examples() {
    assert_eq!(
        compile_and_run(
            include_str!("../examples/planets.tok"),
            "Mercury Venus Earth Mars"
        ),
        Ok(Some(value!([
            "Hello Mercury",
            "Hello Venus",
            "Hello World",
            "Hello Mars"
        ])))
    );

    /*
    assert_eq!(
        compile_and_run(
            include_str!("../examples/planets2.tok"),
            "Mercury Venus Earth Mars Jupiter Saturn Uranus Neptune"
        ),
        Ok(Some(value!([
            "Mercury",
            "Venus (neighbour)",
            "Home",
            "Mars (neighbour)",
            "Jupiter",
            "Saturn",
            "Uranus",
            "Neptune"
        ])))
    );
    */

    assert_eq!(
        compile_and_run(include_str!("../examples/expr.tok"), "1+2*3+4"),
        Ok(Some(value!(11)))
    );

    // todo: Would be nice to test against stdout
    assert_eq!(
        compile_and_run(include_str!("../examples/expr_with_ast.tok"), "1+2*3+4"),
        Ok(None)
    );

    assert_eq!(
        compile_and_run(
            include_str!("../examples/expr_with_spaces.tok"),
            "1 +  \t 2 \n *  3 + 4"
        ),
        Ok(Some(value!(11)))
    );

    assert_eq!(
        compile_and_run(include_str!("../examples/faculty.tok"), ""),
        Ok(Some(value!(24)))
    );
}

#[test]
// Testing sequence captures
fn capture() {
    assert_eq!(
        compile_and_run("'a' 'b' $1 * 2 + $2 * 3", "ab"),
        Ok(Some(value!("aabbb")))
    );

    assert_eq!(
        compile_and_run("a=2 'a' 'b' $(a + 1) * 3+ $(a) * 2", "ab"),
        Ok(Some(value!("bbbaa")))
    );

    assert_eq!(
        compile_and_run("'a' $0 = \"yes\" 'b'+", "abbb"),
        Ok(Some(value!("yes")))
    );
}

// Tests for control flow -------------------------------------------------------------------------

#[test]
// Testing parselet begin and end special patterns
fn begin_end() {
    assert_eq!(
        compile_and_run(
            "
            begin { x = 0 1337 }
            end 1338

            P: @{ 'lol' x = x + 1 x }
            P",
            "lolalolaalolol"
        ),
        Ok(Some(value!([1337, 1, 2, 3, 1338])))
    );

    assert_eq!(
        compile_and_run(
            "
            begin x = 1

            'lol' $1 * x x x = x + 1",
            "lolAlolBlol"
        ),
        Ok(Some(value!([["lol", 1], ["lollol", 2], ["lollollol", 3]])))
    );

    // begin and end without any input
    assert_eq!(
        compile_and_run(
            "
            begin 1
            2 3 4
            end 5
            ",
            ""
        ),
        Ok(Some(value!([1, [2, 3, 4], 5])))
    )
}

#[test]
// Testing parselet repeat
fn repeat() {
    assert_eq!(
        compile_and_run("P: @{ 'a' repeat $1 }\nP", "aaaa"),
        Ok(Some(value!(["a", "a", "a", "a"])))
    );

    // todo: More examples here please!
}

#[test]
// Testing if...else construct
fn if_else() {
    // These expressions are optimized by the compiler
    assert_eq!(
        compile_and_run(
            "
            if true 1 \
            if false 2 \
            if $1 3 else 4 \
            if !$2 5 else 6",
            ""
        ),
        Ok(Some(value!([1, 3, 5])))
    );

    // These expressions are evaluated at compile time
    assert_eq!(
        compile_and_run(
            "
            b = true
            nb = false

            if b 1 \
            if nb 2 \
            if $1 3 else 4 \
            if !$2 5 else 6",
            ""
        ),
        Ok(Some(value!([1, 3, 5])))
    );

    run_testcase("tests/test_if.tok");
}

#[test]
// tests for push and next
fn push_next() {
    assert_eq!(
        compile_and_run(
            "
            1 2 3 next
            4 5 6 push 7
            ",
            ""
        ),
        Ok(Some(value!(7)))
    );

    // todo: This test is a stub. Add more tests regarding next and push.
}

#[test]
// tests for push and next
fn loops() {
    run_testcase("tests/test_loop.tok");
    run_testcase("tests/test_for.tok");
    run_testcase("tests/err_break_continue.tok");
}

// Tests for compiler behavior --------------------------------------------------------------------

#[test]
// Testing several special parsing constructs and error reporting
fn compiler_error_reporting() {
    // Test for programs which consist just of one comment
    assert_eq!(compile_and_run("#tralala", ""), Ok(None));

    // Test for whitespace
    assert_eq!(
        compile_and_run("#normal comment\n#\n\t123", ""),
        Ok(Some(value!(123)))
    );

    // Test for invalid input when EOF is expected
    assert_eq!(
        compile_and_run("{}}", ""),
        Err("Line 1, column 3: Parse error, expecting end-of-file".to_string())
    );

    // Test on unclosed sequences `(1 `
    assert_eq!(
        compile_and_run("(1", ""),
        Err("Line 1, column 3: Expecting \")\"".to_string())
    );

    assert_eq!(
        compile_and_run("(a => 1, b => 2", ""),
        Err("Line 1, column 16: Expecting \")\"".to_string())
    );

    // Test empty sequence
    assert_eq!(compile_and_run("()", ""), Ok(None));

    // Tests on filled and empty blocks and empty blocks
    assert_eq!(
        compile_and_run(
            "
            a = {}
            b = {
            }
            c = {
                1
                2
                3
            }

            a b c
            ",
            ""
        ),
        Ok(Some(value!(3)))
    );
}

#[test]
// Tests for correct identifier names for various value types
fn compiler_identifier_naming() {
    run_testcase("tests/err_compiler_identifier_names.tok");
}

#[test]
// Tests for compiler string, match and ccl escaping
fn compiler_unescaping() {
    assert_eq!(
        compile_and_run(
            "\"test\\\\yes\n\\xCA\\xFE\t\\100\\x5F\\u20ac\\U0001F98E\"",
            ""
        ),
        Ok(Some(value!("test\\yes\nÊþ\t@_€🦎")))
    );

    assert_eq!(
        compile_and_run(
            "'hello\\nworld'", // double \ quotation required
            "hello\nworld"
        ),
        Ok(Some(value!("hello\nworld")))
    );

    assert_eq!(
        compile_and_run(
            "[0-9\\u20ac]+", // double \ quotation required
            "12345€ €12345"
        ),
        Ok(Some(value!(["12345€", "€12345"])))
    );

    assert_eq!(
        compile_and_run(
            "'hello\\nworld'", // double \ quotation required
            "hello\nworld"
        ),
        Ok(Some(value!("hello\nworld")))
    );

    assert_eq!(
        compile_and_run(
            "[0-9\\u20ac]+", // double \ quotation required
            "12345€ €12345"
        ),
        Ok(Some(value!(["12345€", "€12345"])))
    );
}

// Tests for builtins -----------------------------------------------------------------------------

#[test]
// Tests for builtin functions
fn builtins() {
    // ord/chr
    assert_eq!(
        compile_and_run("i = ord(\"€\"); i chr(i)", ""),
        Ok(Some(value![[(8364 as usize), "€"]]))
    );

    assert_eq!(
        compile_and_run("ord(\"12\")", ""),
        Err(
            "Line 1, column 1: ord() expected single character, but received string of length 2"
                .to_string()
        )
    );

    assert_eq!(
        compile_and_run("ord(\"\")", ""),
        Err(
            "Line 1, column 1: ord() expected single character, but received string of length 0"
                .to_string()
        )
    );
}

#[test]
// Tests for builtin string functions
fn builtins_str() {
    assert_eq!(
        compile_and_run(
            "
            \"abcäöü\".upper() \
            \"ABCÄÖÜ\".lower() \
            \"hello world\".replace(\"l\") \
            \"hello world\".replace(\"l\", n=2) \
            \"hello world\".replace(\"l\", \"x\") \
            \"hello world\".replace(\"l\", \"x\", 2) \
            \"hello world\".replace(\"l\").upper() \
            #\"Tokay\".upper()[1]  # index is not implemented for now \
            ",
            ""
        ),
        Ok(Some(value![[
            "ABCÄÖÜ",
            "abcäöü",
            "heo word",
            "heo world",
            "hexxo worxd",
            "hexxo world",
            "HEO WORD" //"O"
        ]]))
    );
}
