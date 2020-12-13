use std::io;
use ::tokay::reader::Reader;
use ::tokay::parser::TokayParser;
use ::tokay::compiler::Compiler;


fn main() {
    let p = TokayParser::new();
    //let s = include_str!("../readme.tok");
    let s = "P = @{}\n@{}";
    //let s = "A\nB\nC\n";
    //let s = "A = @{ \"Hello\"+ B* (1337.+-3) (+true) { if a == b + 1 c else d } }";
    //let s = "A B C\nX Y Z";
    //let s = "x = @{return0}";

    println!("src = {:?}", s);

    let res = p.parse(
        Reader::new(Box::new(io::Cursor::new(s)))
    );

    if let Ok(ast) = res {
        println!("ast = {:?}", ast);

        let mut compiler = Compiler::new();
        compiler.traverse(&ast);

        let prog = compiler.into_program();
        println!("{:?}", prog);
    }
}
