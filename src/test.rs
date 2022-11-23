//! Unit tests
use crate::utils::*;
use crate::value;
use crate::value::*;
use glob::glob;
use std::fs::File;
use std::io::{Read, Write}; // BufRead, BufReader,
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
and the expected output. The advantage of this specification is, that the testcase can directly
be executed with Tokay to reproduce its expected result, as the input and output sections are
specified as comments.

This function currently requires that a tokay debug executable is compiled before test cases are run.

The provided code can either be a testcase or a path to a filename that contains the testcase.

There's also a special REPL test mode: If the first line in the testcase contains `#testmode:repl`,
the code is fed to the Tokay REPL, and expected output is tested againt each line specified. In this
mode, it is important to specify multi-line definitions with the alternative `;` delimiter, otherwise
a syntax error will occur (likewise in the normal REPL).
*/
pub(crate) fn testcase(code: &str) {
    //println!("---");

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
        let mut lines = code.matches("\n").count() + 1;

        //let program = env::args().next().unwrap(); // Doens't work with cargo test
        let program = "target/debug/tokay";
        let mut cmd = Command::new(program);

        //println!("code = {:?}", code);

        if repl_mode {
            cmd.arg("-q");
            cmd.env("TOKAY_HISTORY_SAVE", "0");
            cmd.stdin(Stdio::piped());
        } else {
            cmd.arg(code);
        }

        let mut result = data;
        let tmp;

        // In case there is another separator, split into input and output
        if let Some((input, output)) = data.split_once("#---\n") {
            //println!("input = {:?}", input);
            //println!("output = {:?}", output);

            let input: Vec<&str> = input
                .split("\n")
                .map(|line| line.trim_start())
                .filter(|line| line.starts_with("#"))
                .map(|line| &line[1..])
                .collect();

            tmp = input.join("\n");
            lines += tmp.matches("\n").count() + 1 + 1;

            cmd.arg("--").arg(&tmp);
            result = output;
        }

        // Spawn tokay interpreter process
        let mut process = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(&format!(
                "{} failed to run using {}; You need to run `cargo build` first!",
                filename, program
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

        let out = String::from_utf8(output.stdout).expect("stdout is not UTF-8");
        let err = String::from_utf8(output.stderr).expect("stderr is not UTF-8");

        //println!("out = {:?}", out);
        //println!("err = {:?}", err);

        // Convert ouput and err to Vec<String>, without the last (empty) line if available
        let mut out: Vec<String> = if out.is_empty() {
            Vec::new()
        } else {
            out.trim_end()
                .split("\n")
                .map(|line| line.to_string())
                .collect()
        };

        let mut err: Vec<String> = if err.is_empty() {
            Vec::new()
        } else {
            err.trim_end()
                .split("\n")
                .map(|line| line.to_string())
                .collect()
        };

        //println!("out = {:?}", out);
        //println!("err = {:?}", err);

        //let mut stdout = BufReader::new(process.stdout.as_mut().unwrap());
        //let mut stderr = BufReader::new(process.stderr.as_mut().unwrap());

        for (row, line) in result.split("\n").into_iter().enumerate() {
            let line = line.trim_start();

            if line.is_empty() {
                continue;
            }

            assert!(
                line.starts_with("#"),
                "{}:{} Result must start with a #-comment, got {:?}",
                filename,
                lines + row + 1,
                line
            );

            if line.starts_with("#ERR:") {
                /*
                let mut err = String::new();
                stderr.read_line(&mut err).unwrap();
                if err.ends_with('\n') {
                    err.pop();
                }
                */
                let err = err.remove(0);
                let exp = &line[5..];
                assert_eq!(
                    err,
                    exp,
                    "{}:{} stderr expects {:?} but got {:?}",
                    filename,
                    lines + row + 1,
                    exp,
                    err
                );
            } else {
                /*
                let mut out = String::new();
                stdout.read_line(&mut out).unwrap();
                if out.ends_with('\n') {
                    out.pop();
                }
                */
                let out = out.remove(0);
                let exp = &line[1..];
                assert_eq!(
                    out,
                    exp,
                    "{}:{} stdout expects {:?} but got {:?}",
                    filename,
                    lines + row + 1,
                    exp,
                    out
                );
            }
        }

        assert!(
            out.is_empty(),
            "{} some output not consumed: {:#?}",
            filename,
            out
        );
        assert!(
            err.is_empty(),
            "{} Some errors not consumed: {:#?}",
            filename,
            err
        );

        //process.wait().unwrap();
    } else {
        panic!(
            "{} invalid testcase, at least one '#---' delimiter required",
            filename
        )
    }
}

#[test]
// Simple testcase for testcase
fn test_case() {
    testcase(
        r#"
        print("Hello " + Int)
        #---
        #23
        #---
        #Hello 23
    "#,
    )
}

// Tests expression basics ------------------------------------------------------------------------

#[test]
// Test for operations
fn operations() {
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

// Tests for dicts and lists ----------------------------------------------------------------------

#[test]
// Test for parsing sequences, which may result in lists or dicts.
fn sequences() {
    // Inline operations
    // fixme: IMHO this is wrong!
    assert_eq!(
        run(
            r#"
        a = 1 "x" a += 2 "y" a "z" $6 $5 $4 $3 $2 $1
        #for i = 1; i <= 6; i++ print(i + ": " + $(i))"#,
            ""
        ),
        Ok(Some(value!(["x", "y", 3, "z", "z", 3, "y", "x"])))
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
        run(include_str!("../examples/factorial.tok"), ""),
        Ok(Some(value!(24)))
    );
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
// Run all tests in the tests/-folder
fn tests() {
    for case in glob("tests/*.tok").expect("Failed to read tests/") {
        let case = case.unwrap();
        let filename = case.as_path().to_str().unwrap();
        println!("::: {} :::", filename);
        testcase(filename);
    }
}
