//! Unit tests

use crate::compiler::*;
use crate::token::*;
use crate::utils::*;
use crate::value;
use crate::value::*;
use crate::vm::*;
use crate::{tokay_embed, tokay_embed_item};

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
fn test_builtin_tokens() {
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
        compile_and_run("Whitespace", gliders, false),
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

    // Builtin Whitespace handling
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

#[test]
fn test_variables() {
    assert_eq!(
        compile_and_run(
            "
            a = 1 \
            a \
            a++ \
            ++a \
            a++ a++ \
            a+++++a
            ",
            "",
            false
        ),
        Ok(Some(value![[1, 1, 3, 3, 4, 12]]))
    );
}

#[test]
fn test_collections() {
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

#[test]
fn test_token() {
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
        compile_and_run("P: @{ P? ''a'' }\nP", "aaaa", false),
        Ok(Some(value!([[["a", "a"], "a"], "a"])))
    );

    // todo: More examples here please!
}

#[test]
fn test_parselet_loop() {
    assert_eq!(
        compile_and_run("P: @{ 'a' repeat $1 }\nP", "aaaa", false),
        Ok(Some(value!(["a", "a", "a", "a"])))
    );

    // todo: More examples here please!
}

#[test]
fn test_examples() {
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

/*
println!("{:?}",
    compile_and_run(
        "
        A : {
            [a-zA-Z]+   $1
            Integer     $1 * 2
        }

        A",
        "73.2 Integer6        Benno",
        false
    )
);
*/

/*
Tests for the parser

let parser = Parser::new();
for expr in &[
    "A : [a-c]\nA+\n",
    "A : [a-c]+\nA+\n",
    "B : 'Hello'\nB+\n",
    "B : 'Hello'+\nB+\n",
] {
    println!("-------------------------------------\n{}", expr);
    let ast = parser.parse(Reader::new(Box::new(io::Cursor::new(expr))));

    match ast {
        Ok(ast) => Parser::print(&ast),
        Err(err) => println!("{}", err),
    }
}
*/

//println!("{:?}", compile_and_run("[loA-Z]+ print", "lol", true));

/*
println!("{:?}",
    compile_and_run(
        "
        begin { x = 0 1337 }
        end 1338

        P: @{ 'lol' x = x + 1 x }
        P",
        "lolalolaalolol",
        false
    )
);
*/

/*
let ast = compile_and_run(
    include_str!("repl.tok"),
    "#debug\n",
    true,
).unwrap().unwrap();

Parser::print(
    &ast.borrow()
);
*/

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
fn testindirectleftrec() {
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
fn testleftrec() {
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
        (Z = {
            (call print[(value "hello world")])
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
