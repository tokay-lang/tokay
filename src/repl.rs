use std::fs::File;
use std::io::{self, BufReader, Write};

use crate::compiler::Compiler;
use crate::reader::Reader;
use crate::value::RefValue;
use crate::vm::Runtime;

// A first simple REPL for Tokay
pub fn repl() {
    let mut globals: Vec<RefValue> = Vec::new();

    let mut compiler = Compiler::new();
    compiler.interactive = true;
    //compiler.debug = true;

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut code = String::new();
        if io::stdin().read_line(&mut code).is_err() {
            panic!("Error reading code")
        }

        // Stop when program is empty.
        if code.trim().is_empty() {
            return;
        }

        //println!("code = {:?}", code);

        match code.as_str() {
            "#debug\n" => {
                compiler.debug = true;
                println!("<<< Debug switched on")
            }
            "#nodebug\n" => {
                compiler.debug = false;
                println!("<<< Debug switched off")
            }
            _ => {
                if let Some(program) =
                    compiler.compile(Reader::new(Box::new(io::Cursor::new(code))))
                {
                    if std::env::args().len() == 1 {
                        let mut reader = Reader::new(Box::new(io::Cursor::new("")));
                        let mut runtime = Runtime::new(&program, &mut reader);
                        runtime.load_stack(globals);

                        let res = program.run(&mut runtime);
                        match res {
                            Ok(None) => {}
                            Ok(Some(value)) => println!("<<< {}", value.borrow().repr()),
                            Err(error) => println!("<<< {}", error),
                        }

                        globals = runtime.save_stack();
                    } else {
                        for filename in std::env::args().skip(1) {
                            let file = File::open(&filename).unwrap();

                            let mut reader = Reader::new(Box::new(BufReader::new(file)));
                            let mut runtime = Runtime::new(&program, &mut reader);
                            runtime.load_stack(globals);

                            let res = program.run(&mut runtime);

                            match res {
                                Ok(None) => {}
                                Ok(Some(value)) => {
                                    println!("{}: {}", filename, value.borrow().repr())
                                }
                                Err(error) => println!("{}: {}", filename, error),
                            }

                            globals = runtime.save_stack();
                        }
                    }
                }
            }
        }
    }
}
