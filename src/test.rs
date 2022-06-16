//! Unit tests

use crate::utils::*;
use crate::value;
use crate::value::*;

//use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

/** Test case utility function to run test-cases from files with expected output.

This function currently requires that a tokay debug executable is compiled before test cases are run.
*/
pub(crate) fn testcase(filename: &'static str) {
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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
        run(
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

    // Inline add int to itself
    assert_eq!(
        run(
            "
            i = 2 \
            i *= i \
            i
            ",
            ""
        ),
        Ok(Some(value!(4)))
    );

    // Inline add list to itself
    // fixme: Should be put into the specific modules, e.g. value/list.rs
    assert_eq!(
        run(
            "
            a = (1,2) \
            b = (3,4) \
            a + b \
            a \
            b \
            a += b \
            a \
            b
            ",
            ""
        ),
        Ok(Some(value!([
            [1, 2, 3, 4],
            [1, 2],
            [3, 4],
            [1, 2, 3, 4],
            [3, 4]
        ])))
    );
}

#[test]
// Test for variable store/hold cases
fn variables() {
    // Test store-hold-global
    assert_eq!(
        run(
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
        run(
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
        run(
            "
            10 20 $1 = $2 = 30 ++$1 $2
            ",
            ""
        ),
        Ok(Some(value![[31, 30, 31, 30]]))
    );

    // Test store-hold-aliased-capture
    assert_eq!(
        run(
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
        run(include_str!("../tests/test_scopes.tok"), ""),
        Ok(Some(value![[10, 2000, 1072]]))
    );
}

// Tests for dicts and lists ----------------------------------------------------------------------

#[test]
// Test for parsing inline-sequences, which may result in lists or dicts.
fn inline_sequences() {
    // Inline alternation
    assert_eq!(run("('a' | 'b' | 'c')", "b"), Ok(Some(value!("b"))));

    // Lists
    assert_eq!(
        run(
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
        run(
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
        run("'a' 'b'", s),
        Ok(Some(value![[["a", "b"], ["a", "b"]]]))
    );

    // Touch and match
    assert_eq!(run("'a' ''b''", s), Ok(Some(value![["b", "b"]])));

    // Match with positive modifier
    assert_eq!(
        run("'a' ''b''+", s),
        Ok(Some(value![["b", ["b", "b", "b"]]]))
    );

    // Match with kleene and positive modifiers
    assert_eq!(
        run("''a''* ''b''+", s),
        Ok(Some(value![[
            ["a", "b"],
            ["a", ["b", "b", "b"]],
            ["b", "b"]
        ]]))
    );

    // Touch with kleene and positive modifiers
    assert_eq!(
        run("'a'* ''b''+", s),
        Ok(Some(value![["b", ["b", "b", "b"], ["b", "b"]]]))
    );

    // Character classes
    assert_eq!(run("[a-z]", &s[..2]), Ok(Some(value![["a", "b"]])));

    assert_eq!(
        run("[a-z]+", s),
        Ok(Some(value![["ab", "abbb", "bb", "def"]]))
    );

    assert_eq!(
        run("[^ ']+", s),
        Ok(Some(value![[
            "ab", "abbb", "bb", "123", "ABC", "456", "def"
        ]]))
    );

    // Built-in token
    assert_eq!(run("Int", s), Ok(Some(value![[123, 456]])));

    // Parsing with sequences and modifiers

    assert_eq!(
        run("''a'' {''b'' ''c''}* ''d''", "abcbcd"),
        Ok(Some(value![["a", [["b", "c"], ["b", "c"]], "d"]]))
    );

    assert_eq!(
        run("''a'' {''b'' ''c''}+ ''d''", "abcbcd"),
        Ok(Some(value![["a", [["b", "c"], ["b", "c"]], "d"]]))
    );

    assert_eq!(
        run("''a'' {''b'' ''c''}* ''d''", "ad"),
        Ok(Some(value![["a", "d"]]))
    );

    assert_eq!(run("''a'' {''b'' ''c''}+ ''d''", "ad"), Ok(None));

    assert_eq!(
        run("{ Word { ',' _ }? }+", "Hello,   World,  Beta,  Test"),
        Ok(Some(value![["Hello", "World", "Beta", "Test"]]))
    );

    // todo: more token tests, please!
}

#[test]
// Testing examples provided in the examples folder
fn examples() {
    assert_eq!(
        run(
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
        run(
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
        run(include_str!("../examples/expr.tok"), "1+2*3+4"),
        Ok(Some(value!(11)))
    );

    // todo: Would be nice to test against stdout
    assert_eq!(
        run(include_str!("../examples/expr_with_ast.tok"), "1+2*3+4"),
        Ok(None)
    );

    assert_eq!(
        run(
            include_str!("../examples/expr_with_spaces.tok"),
            "1 +  \t 2 \n *  3 + 4"
        ),
        Ok(Some(value!(11)))
    );

    assert_eq!(
        run(include_str!("../examples/faculty.tok"), ""),
        Ok(Some(value!(24)))
    );

    // todo: Move this to a separate function
    testcase("tests/test_piped_grammar.tok");
}

// Tests for control flow -------------------------------------------------------------------------

#[test]
// Testing if...else construct
fn if_else() {
    // These expressions are optimized by the compiler
    assert_eq!(
        run(
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
        run(
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

    testcase("tests/test_if.tok");
}

#[test]
// tests for push and next
fn push_next() {
    assert_eq!(
        run(
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
    testcase("tests/test_loop.tok");
    testcase("tests/test_for.tok");
    testcase("tests/err_break_continue.tok");
}
