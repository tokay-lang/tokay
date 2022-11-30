//! Unit tests
use crate::utils::*;
use crate::value;
use tokay_macros;

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

tokay_macros::tokay_tests!("tests/*.tok");
