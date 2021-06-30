//! Unit tests

use crate::compiler::*;
use crate::token::*;
use crate::utils::*;
use crate::value;
use crate::value::*;
use crate::vm::*;
use crate::{tokay_embed, tokay_embed_item};

// Tests expression basics ------------------------------------------------------------------------

#[test]
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
            "",
            false
        ),
        Ok(Some(value!([1337, 23.5, true, false, "Hello World"])))
    );
}

#[test]
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
            "",
            false
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
            "",
            false
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
            "",
            false
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
            "",
            false
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
            "",
            false
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
            "",
            false
        ),
        Ok(Some(value!([true, false, true, true, true, false, false])))
    );
}

#[test]
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
            "",
            false
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
            "",
            false
        ),
        Ok(Some(value![[1, 1, 3, 3, 4, 12, 2, 8, 3]]))
    );
}

#[test]
fn variables() {
    // Test store-hold-global
    assert_eq!(
        compile_and_run(
            "
            a = b = 10
            a++
            a b
            ",
            "",
            false
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
            "",
            false
        ),
        Ok(Some(value![[11, 10]]))
    );

    // Test store-hold-capture
    assert_eq!(
        compile_and_run(
            "
            10 20 $1 = $2 = 30 ++$1 $2
            ",
            "",
            false
        ),
        Ok(Some(value![[31, 30, 31, 30]]))
    );

    // Test store-hold-aliased-capture
    assert_eq!(
        compile_and_run(
            "
            a => 10 b => 20 $a = $b = 30 c => ++$a d => $b
            ",
            "",
            false
        ),
        Ok(Some(value![["a" => 31, "b" => 30, "c" => 31, "d" => 30]]))
    );
}

#[test]
fn scoping() {
    // Test-case for scoping
    assert_eq!(
        compile_and_run(include_str!("tests/testcase_scopes.tok"), "", false),
        Ok(Some(value![[10, 2000, 1072]]))
    );
}

// Tests for dicts and lists ----------------------------------------------------------------------

#[test]
fn collections() {
    // Lists
    assert_eq!(
        compile_and_run(
            "
            (1 2 3) \
            (1, 2, 3) \
            (42, \"Hello\", 23.5, true, false)
            ",
            "",
            false
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
            "",
            false
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
fn token() {
    let s = "ab abbb bb 123 ABC 456 'def'";

    // Simple touch
    assert_eq!(
        compile_and_run("'a' 'b'", s, false),
        Ok(Some(value![[["a", "b"], ["a", "b"]]]))
    );

    // Touch and match
    assert_eq!(
        compile_and_run("'a' ''b''", s, false),
        Ok(Some(value![["b", "b"]]))
    );

    // Match with positive modifier
    assert_eq!(
        compile_and_run("'a' ''b''+", s, false),
        Ok(Some(value![["b", ["b", "b", "b"]]]))
    );

    // Match with kleene and positive modifiers
    assert_eq!(
        compile_and_run("''a''* ''b''+", s, false),
        Ok(Some(value![[
            ["a", "b"],
            ["a", ["b", "b", "b"]],
            ["b", "b"]
        ]]))
    );

    // Touch with kleene and positive modifiers
    assert_eq!(
        compile_and_run("'a'* ''b''+", s, false),
        Ok(Some(value![["b", ["b", "b", "b"], ["b", "b"]]]))
    );

    // Character classes
    assert_eq!(
        compile_and_run("[a-z]", &s[..2], false),
        Ok(Some(value![["a", "b"]]))
    );

    assert_eq!(
        compile_and_run("[a-z]+", s, false),
        Ok(Some(value![["ab", "abbb", "bb", "def"]]))
    );

    assert_eq!(
        compile_and_run("[^ ']+", s, false),
        Ok(Some(value![[
            "ab", "abbb", "bb", "123", "ABC", "456", "def"
        ]]))
    );

    // Built-in token
    assert_eq!(
        compile_and_run("Integer", s, false),
        Ok(Some(value![[123, 456]]))
    );

    // todo: more token tests, please!
}

#[test]
fn builtin_tokens() {
    let gliders = "Glasflügel Libelle 201b\tG102 Astir  \nVentus_2cT";

    assert_eq!(
        compile_and_run("Identifier", gliders, false),
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
        compile_and_run("Integer", gliders, false),
        Ok(Some(value!([201, 102, 2])))
    );

    assert_eq!(
        compile_and_run("Whitespaces", gliders, false),
        Ok(Some(value!([" ", " ", "\t", " ", "  \n"])))
    );

    assert_eq!(
        compile_and_run("Word", gliders, false),
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
        compile_and_run("Word _; ", abc, false),
        Ok(Some(value![["abc", "def", "abcabc", "ghi", "abcdef"]]))
    );

    assert_eq!(
        compile_and_run("Word __; ", abc, false),
        Ok(Some(value![["abc", "def", "ghi"]]))
    );
}

// Tests for parselets ----------------------------------------------------------------------------

#[test]
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
            "",
            false
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
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
            "",
            false
        ),
        Ok(Some(value!(24)))
    );
}

#[test]
fn parselet_leftrec() {
    assert_eq!(
        compile_and_run("P: @{ P? ''a'' }\nP", "aaaa", false),
        Ok(Some(value!([[["a", "a"], "a"], "a"])))
    );

    // todo: More examples here please!
}

#[test]
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

        assert_eq!(compile_and_run(&call, "", false), Err(msg.to_owned()));
    }
}

#[test]
fn examples() {
    assert_eq!(
        compile_and_run(
            include_str!("../examples/planets.tok"),
            "Mercury Venus Earth Mars",
            false
        ),
        Ok(Some(value!([
            "Hello Mercury",
            "Hello Venus",
            "Hello World",
            "Hello Mars"
        ])))
    );

    assert_eq!(
        compile_and_run(
            include_str!("../examples/planets2.tok"),
            "Mercury Venus Earth Mars Jupiter Saturn Uranus Neptune",
            false
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

    assert_eq!(
        compile_and_run(include_str!("../examples/expr.tok"), "1+2*3+4", false),
        Ok(Some(value!(11)))
    );

    // todo: Would be nice to test against stdout
    assert_eq!(
        compile_and_run(
            include_str!("../examples/expr_with_ast.tok"),
            "1+2*3+4",
            false
        ),
        Ok(None)
    );

    assert_eq!(
        compile_and_run(
            include_str!("../examples/expr_with_spaces.tok"),
            "1 +  \t 2 \n *  3 + 4",
            false
        ),
        Ok(Some(value!(11)))
    );

    assert_eq!(
        compile_and_run(include_str!("../examples/faculty.tok"), "", false),
        Ok(Some(value!(24)))
    );
}

#[test]
fn capture() {
    assert_eq!(
        compile_and_run("'a' 'b' $1 * 2 + $2 * 3", "ab", false),
        Ok(Some(value!("aabbb")))
    );

    assert_eq!(
        compile_and_run("a=2 'a' 'b' $(a + 1) * 3+ $(a) * 2", "ab", false),
        Ok(Some(value!("bbbaa")))
    );
}

// Tests for control flow -------------------------------------------------------------------------

#[test]
fn begin_end() {
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

    // begin and end without any input
    assert_eq!(
        compile_and_run(
            "
            begin 1
            2 3 4
            end 5
            ",
            "",
            false
        ),
        Ok(Some(value!([1, [2, 3, 4], 5])))
    )
}

#[test]
fn repeat() {
    assert_eq!(
        compile_and_run("P: @{ 'a' repeat $1 }\nP", "aaaa", false),
        Ok(Some(value!(["a", "a", "a", "a"])))
    );

    // todo: More examples here please!
}

#[test]
fn if_else() {
    // These expressions are optimized by the compiler
    assert_eq!(
        compile_and_run(
            "
            if true 1 \
            if false 2 \
            if $1 3 else 4 \
            if !$2 5 else 6",
            "",
            false
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
            "",
            false
        ),
        Ok(Some(value!([1, 3, 5])))
    );
}

#[test]
fn push_next() {
    assert_eq!(
        compile_and_run(
            "
            1 2 3 next
            4 5 6 push 7
            ",
            "",
            false
        ),
        Ok(Some(value!(7)))
    );

    // todo: This test is a stub. Add more tests regarding next and push.
}

// Tests for compiler behavior --------------------------------------------------------------------

// Universal function to run test-cases with expected errors inside the code.
fn run_testcase(code: &'static str) {
    if let Some((code, result)) = code.split_once("#---\n") {
        let expect = result
            .trim()
            .split("\n")
            .into_iter()
            .map(|line| {
                assert!(
                    line.starts_with("#"),
                    "Lines in result must start with a comment-#"
                );
                line[1..].to_string()
            })
            .collect::<Vec<String>>()
            .join("\n");

        assert_eq!(compile_and_run(code, "", false), Err(expect));
    } else {
        panic!("Testcase invalid, require for a '#---' delimiter.")
    }
}

#[test]
fn compiler_structure() {
    // Testing several parsing constructs

    // Tests for blocks and empty blocks
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
            "",
            false
        ),
        Ok(Some(value!(3)))
    );

    // Test for invalid input when EOF is expected
    assert_eq!(
        compile_and_run("{}}", "", false),
        Err("Line 1, column 3: Parse error, expecting end-of-file".to_string())
    );
}

#[test]
fn compiler_identifier_naming() {
    // Tests for correct identifier names for various value types
    run_testcase(include_str!("tests/testcase_compiler_identifier_names.tok"));
}

// Tests for parsing and packrat features ---------------------------------------------------------

/*
    Below are some tests that provide indirect left-recursion.

    They currently don't work properly due to the following reason:
    For indirect left-recursion in packrat parsing, one rule in the
    grammar's graph must be declared as "leading", so that subsequent,
    even left-recursive parselets are considered as not left-recursive.


    An implementation of an solution for this issue can be found in
    the pegen parser generator from Python:

    https://github.com/python/cpython/blob/main/Tools/peg_generator/pegen/parser_generator.py

    Tokay won't take care of this right now as it is an edge-case
    and also more complex, as Tokay does not directly implements a
    grammar.
*/

#[test]
fn parser_indirectleftrec() {
    let program = tokay_embed!({
        (X = {
            [Y, (MATCH "c")]
        }),
        (Y = {
            [Z, (MATCH "b")]
            //Void
        }),
        (Z = {
            X,
            Y,
            (MATCH "a")
        }),
        Z
    });

    println!("{:#?}", program.run_from_str("aaabc"));
}

#[test]
fn parser_leftrec() {
    /*
    let program = tokay_embed!({
        (X = {
            [X, (MATCH "b")],
            (MATCH "a")
        }),

        X
    });
    */

    let program = tokay_embed!({
        (Y = {
            X,
            (MATCH "a")
        }),
        (X = {
            [Y, (MATCH "b")]
        }),
        X
    });

    /*
    let program = tokay_embed!({
        (Factor = {
            ["(", (pos [Expression]), ")"],
            (token (Token::Chars(ccl!['0'..='9'])))
        }),
        (Expression = {
            [Expression, "+", Expression],
            Factor
        }),
        Expression
    });
    */

    println!("{:#?}", program.run_from_str("abb"));
}

// Tests for utils --------------------------------------------------------------------------------

#[test]
fn unescape() {
    // First try to unescape by utils function
    assert_eq!(
        crate::utils::unescape("test\\\\yes\n\\xCA\\xFE\t\\100\\x5F\\u20ac\\U0001F929"),
        "test\\yes\nÊþ\t@_€🤩"
    );
}
