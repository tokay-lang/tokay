use tokay;
use std::fs::File;
use std::io::{self, Write};


fn main() -> io::Result<()> {
    let mut compiler = tokay::Compiler::new();

    // let s = include_str!("../../../examples/expr_from_readme.tok");
    // let s = include_str!("../../../examples/hello.tok");
    let s = "x = 42; x + 5";
    let program = compiler.compile_from_str(s).unwrap();

    /*
    let byte_program: Vec<u8> = bincode::serde::encode_to_vec(&program, bincode::config::standard()).expect("Encoding failed");
    */

    let json_program = serde_json::to_string(&program).unwrap();

    let mut file = File::create("../hello.json")?;
    file.write_all(&json_program.as_bytes())?;
    file.flush()?;

    println!("OK");

    Ok(())
}
