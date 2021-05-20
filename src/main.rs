use ::tokay::compiler::Compiler;
use ::tokay::reader::Reader;
use ::tokay::repl::repl;
use ::tokay::vm::Runtime;

#[macro_use]
extern crate clap;
use clap::App;

use std::fs::File;
use std::io::{self, BufReader, Write};

fn main() {
    let yaml = load_yaml!("main.yaml");
    let opts = App::from(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .get_matches();
    //println!("opts = {:?}", opts);

    let debug = opts.occurrences_of("v");

    // Try getting program from argument or file
    let mut program = None;

    if let Some(src) = opts.value_of("program") {
        program = Some(Reader::new(Box::new(std::io::Cursor::new(src.to_string()))));
    } else if let Some(filename) = opts.value_of("file") {
        if let Ok(file) = File::open(&filename) {
            program = Some(Reader::new(Box::new(BufReader::new(file))));
        } else {
            println!("Program file '{}' cannot be read.", filename);
            std::process::exit(1);
        }
    }

    // Try getting files to run on program or repl
    let files: Option<Vec<&str>> = if let Some(files) = opts.values_of("input") {
        Some(files.collect())
    } else {
        None
    };

    if let Some(program) = program {
        let mut compiler = Compiler::new();

        if let Some(program) = compiler.compile(program) {
            if let Some(files) = files {
                for file in &files {
                    match program.run_from_file(file) {
                        Ok(value) => {
                            if let Some(value) = value {
                                if files.len() > 1 {
                                    println!("{}: {}", file, value.borrow());
                                } else {
                                    println!("{}", value.borrow())
                                }
                            }
                        }
                        Err(error) => {
                            if files.len() > 1 {
                                println!("{}: {}", file, error);
                            } else {
                                println!("{}", error);
                            }
                        }
                    }
                }
            } else {
                match program.run_from_str("") {
                    Ok(value) => {
                        if let Some(value) = value {
                            println!("{}", value.borrow())
                        }
                    }
                    Err(error) => {
                        println!("{}", error);
                    }
                }
            }
        }
    } else {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        repl();
    }
}
