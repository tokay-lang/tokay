//! Unit tests
use crate::utils::*;
use crate::value;
use crate::value::*;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

/** Test-case evaluator.

A testcase is made of the following sections:
```tokay
print("Hello " + Int)   # code
#---
#23                     # input (optional)
#---
#Hello 23               # expected output
```

This is an entire testcase to describe a program, the input fed to it (which is optional)
and the expected output.

This function currently requires that a tokay debug executable is compiled before test cases are run.

The provided code can either be a testcase or a path to a filename that contains the testcase.

There's also a special REPL test mode: If the first line in the testcase contains `#testmode:repl`,
the code is fed to the Tokay REPL, and expected output is tested againt each line specified. In this
mode, it is important to specify multi-line definitions with the alternative `;` delimiter, otherwise
a syntax error will occur (likewise in the normal REPL).
*/
pub(crate) fn testcase(code: &str) {
    // Try to open code as file
    let (filename, code) = match File::open(code) {
        // The file is open (no error).
        Ok(mut file) => {
            let mut content = String::new();

            // Read all the file content into a variable (ignoring the result of the operation).
            file.read_to_string(&mut content).unwrap();

            // The file is automatically closed when it goes out of scope.
            (code, content)
        }
        // Error handling.
        Err(_) => ("--", code.to_owned()),
    };

    //println!("code = {:?}", code);
    let repl_mode = code.starts_with("#testmode:repl\n");

    if let Some((code, data)) = code.split_once("#---\n") {
        //let program = env::args().next().unwrap(); // Doens't work with cargo test
        let program = "target/debug/tokay";
        let mut cmd = Command::new(program);

        if repl_mode {
            cmd.arg("-q");
            cmd.stdin(Stdio::piped());
        } else {
            cmd.arg(code);
        }

        let mut result = data;
        let tmp;

        // In case there is another separator, split into input and output
        if let Some((input, output)) = data.split_once("#---\n") {
            let input: Vec<&str> = input
                .split("\n")
                .filter(|line| line.starts_with("#"))
                .map(|line| &line[1..])
                .collect();

            tmp = input.join("\n");

            cmd.arg("--").arg(&tmp);
            result = output;
        }

        // Spawn tokay interpreter process
        let mut process = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(&format!(
                "Failed to run testcase using {}; You need to run `cargo build` first!",
                program
            ));

        if repl_mode {
            process
                .stdin
                .as_mut()
                .unwrap()
                .write(code.as_bytes())
                .unwrap();
        }

        let output = process.wait_with_output().unwrap();

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

        //println!("out = {:?}", out);
        //println!("err = {:?}", err);

        for line in result.trim().split("\n").into_iter() {
            if line.is_empty() {
                continue;
            }

            assert!(
                line.starts_with("#"),
                "{}: Lines in result must start with a #-comment, got {:?}",
                filename,
                line
            );

            if line.starts_with("#ERR:") {
                let err = err.remove(0);
                let exp = &line[5..];
                assert_eq!(
                    err, exp,
                    "{} stderr expects {:?} but got {:?}",
                    filename, exp, err
                );
            } else {
                let out = out.remove(0);
                let exp = &line[1..];
                assert_eq!(
                    out, exp,
                    "{} stdout expects {:?} but got {:?}",
                    filename, exp, out
                );
            }
        }

        assert!(
            out.len() >= 1,
            "Some output {:?} not consumed in {}",
            out,
            filename
        );
        assert!(
            err.len() >= 1,
            "Some errors {:?} not consumed in {}",
            err,
            filename
        );
    } else {
        panic!(
            "Testcase invalid, require for a '#---' delimiter in {}",
            filename
        )
    }
}

#[test]
// Simple testcase for testcase
fn test_case() {
    testcase(
        r#"print("Hello " + Int)
#---
#23
#---
#Hello 23"#,
    )
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
            [1, 2, 3, 4], // fixme: this is currently correct, because a is modified afterwards;
            // anyway, there should be some kind of copy function or a copy-operator
            // to copy an object and make it self-sustaining.
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
// Test for parsing sequences, which may result in lists or dicts.
fn sequences() {
    // Sequences
    assert_eq!(run("1 2 3", ""), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("1 (2) 3", ""), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("1 (2,) 3", ""), Ok(Some(value!([1, [2], 3]))));
    assert_eq!(run("1 (2 3) 4", ""), Ok(Some(value!([1, [2, 3], 4]))));

    assert_eq!(run("(1 2 3)", ""), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("(1 (2) 3)", ""), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("(1 (2,) 3)", ""), Ok(Some(value!([1, [2], 3]))));
    assert_eq!(run("(1 (2 3) 4)", ""), Ok(Some(value!([1, [2, 3], 4]))));

    assert_eq!(run("1 'a' 2 3", "a"), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("1 ('a' 2) 3", "a"), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("1 (2 'a' 3) 4", "a"), Ok(Some(value!([1, [2, 3], 4]))));
    assert_eq!(run("1 (2 'a' 3) 4", "b"), Ok(None));

    assert_eq!(run("(1 'a' 2 3)", "a"), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(run("(1 ('a' 2) 3)", "a"), Ok(Some(value!([1, 2, 3]))));
    assert_eq!(
        run("(1 (2 'a' 3) 4)", "a"),
        Ok(Some(value!([1, [2, 3], 4])))
    );
    assert_eq!(run("(1 (2 'a' 3) 4)", "b"), Ok(None));

    // Inline operations
    assert_eq!(
        run(
            r#"
        a = 1 "x" a += 2 "y" a "z" $6 $5 $4 $3 $2 $1
        #for i = 1; i <= 6; i++ print(i + ": " + $(i))"#,
            ""
        ),
        Ok(Some(value!(["x", "y", 3, "z", "z", 3, "y", "x"])))
    );

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

    // Issue #63
    testcase("tests/test_sequence_issue_63.tok");
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
        run(include_str!("../examples/factorial.tok"), ""),
        Ok(Some(value!(24)))
    );

    // todo: Move stuff below to a separate function
    testcase("tests/test_piped_grammar.tok");
    testcase("tests/test_inline_parseable_sequence.tok");
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
            r#"
            b = true
            nb = false

            if b 1 \
            if nb 2 \
            if $1 3 else 4 \
            if !$2 5 else 6
            "#,
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

#[test]
// Run all tests in the tests/-folder
fn tests() {
    let cases = std::fs::read_dir("tests").unwrap();

    for case in cases {
        testcase(case.unwrap().path().to_str().unwrap());
    }
}
