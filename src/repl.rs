use std::io::{self, Write};

use crate::compiler::Compiler;
use crate::reader::Reader;
use crate::value::RefValue;
use crate::vm::Runtime;

// A first simple REPL for Tokay
pub fn repl() {
    let mut globals: Vec<RefValue> = Vec::new();

    let mut compiler = Compiler::new();
    compiler.interactive = true;

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
                    let mut reader = Reader::new(Box::new(io::Cursor::new("")));
                    let mut runtime = Runtime::new(&program, &mut reader);
                    runtime.load_stack(globals);

                    let res = program.run(&mut runtime);
                    match res {
                        Ok(None) => {}
                        Ok(Some(value)) => println!("<<< {}", value.borrow().to_string()),
                        _ => println!("<<< {:?}", res),
                    }

                    globals = runtime.into_stack();
                    //println!("globals = {:?}", globals);

                    /*
                    if std::env::args().len() == 1 {
                        let res = program.run_from_str("");
                        match res {
                            Ok(None) => {}
                            Ok(Some(value)) => println!("<<< {}", value.borrow().to_string()),
                            _ => println!("<<< {:?}", res),
                        }
                    } else {
                        for filename in std::env::args().skip(1) {
                            let file = File::open(&filename).unwrap();
                            let res = program.run_from_reader(file);

                            match res {
                                Ok(None) => {}
                                Ok(Some(value)) => {
                                    println!("{}: {:?}", filename, value.borrow().to_string())
                                }
                                _ => println!("{}: {:?}", filename, res),
                            }
                        }
                    }
                    */
                }
            }
        }
    }
}
