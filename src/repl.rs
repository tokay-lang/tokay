use std::fs::File;
use std::io::{self, BufReader, Write};

use crate::compiler::Compiler;
use crate::error::Error;
use crate::reader::Reader;
use crate::value::RefValue;
use crate::vm::Runtime;

// A first simple REPL for Tokay
pub fn repl(files: Option<Vec<&str>>) {
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
                    if let Some(files) = &files {
                        for filename in files {
                            let mut reader;

                            if *filename == "-" {
                                reader = Reader::new(Box::new(BufReader::new(io::stdin())));
                            } else if let Ok(file) = File::open(filename) {
                                reader = Reader::new(Box::new(BufReader::new(file)));
                            } else {
                                println!(
                                    "{}",
                                    Error::new(
                                        None,
                                        format!("Unable to read from filename '{}'", filename),
                                    )
                                );
                                continue;
                            }

                            let mut runtime = Runtime::new(&program, &mut reader);
                            runtime.debug = compiler.debug;
                            runtime.load_stack(globals);

                            let ret = program.run(&mut runtime);

                            if files.len() > 1 {
                                print!("{}: ", filename);
                            }

                            match ret {
                                Ok(None) => print!("\n"),
                                Ok(Some(value)) => println!("{}", value.borrow()),
                                Err(error) => println!("{}", error),
                            }

                            globals = runtime.save_stack();
                        }
                    } else {
                        let mut reader = Reader::new(Box::new(io::Cursor::new("")));
                        let mut runtime = Runtime::new(&program, &mut reader);
                        runtime.debug = compiler.debug;
                        runtime.load_stack(globals);

                        let res = program.run(&mut runtime);
                        match res {
                            Ok(None) => {}
                            Ok(Some(value)) => println!("<<< {}", value.borrow().repr()),
                            Err(error) => println!("<<< {}", error),
                        }

                        globals = runtime.save_stack();
                    }
                }
            }
        }
    }
}
