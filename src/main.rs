use std::io;
use ::tokay::reader::Reader;
use ::tokay::parser::TokayParser;
use ::tokay::compiler::Compiler;


fn main() {
    let p = TokayParser::new();
    //let s = include_str!("../a.tok");
    let s = include_str!("../readme.tok");

    // todo: turn the examples below into a test suite
    //let s = "P = @{\nP? 'Hello'\nP? \"World\"}\nP";
    //let s = "P = @{\nP? \"Hello\"\nP? \"World\"}\nP";
    //let s = "A = @{ \"Hello\"+ B* (1337.+-3) (+true) { if a == b + 1 c else d } }";
    //let s = "A B C\nX Y Z";
    //let s = "x = @{return0}";
    //let s = "a = 42 a a + 1 a + 2";
    //let s = "A = 'Hello' A+ 3 + 2* ('Bernd Waldemar')";

    // Capture load
    //let s = "'Hello' 'World' $1 * 2 + $2 * 3";
    //let s = "a = 2 'Hello' 'World' $(a) * 2 + $( a + 1 ) * 3";

    // Capture store
    //let s = "a = 2 'Hello' 'World' $2 = $3 * 2 + $2 $3";
    //let s = "a = 2 'Hello' 'World' $(a) = $3 * 2 + $2 $3 * 2";

    // Comparisons
    //let s = "'Hello' 'World' $1 == $2";

    //let s = "P = @{ P 'A' 'B' $2 * 2 + $3 * 3 }\nP";
    //let s = "a:'Hello' a\na : 'Hallo' A";

    println!("src = {:?}", s);

    let res = p.parse(
        Reader::new(Box::new(io::Cursor::new(s)))
    );

    if let Ok(ast) = res {
        println!("ast = {:#?}", ast);

        let mut compiler = Compiler::new();
        compiler.traverse(&ast);

        let prg = compiler.into_program();
        //prg.dump();

        println!("prg = {:#?}", prg);

        //println!("res = {:?}", prg.run_from_str("HelloWelt"));
        println!("res = {:?}", prg.run_from_str("1+2*3+4"));
    }
}
