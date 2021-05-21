use ::tokay::compiler::Compiler;
use ::tokay::reader::Reader;
use ::tokay::repl::repl;

#[macro_use]
extern crate clap;
use clap::App;

use std::fs::File;
use std::io::BufReader;

fn print_version() {
    println!("Tokay {}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    let yaml = load_yaml!("main.yaml");
    let opts = App::from(yaml)
        .bin_name(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .get_matches();
    //println!("opts = {:?}", opts);

    let debug = opts.occurrences_of("debug"); // todo: Not used yet.

    if opts.is_present("license") {
        print_version();
        println!("{}", include_str!("../LICENSE"));
        std::process::exit(0);
    }

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
                for filename in &files {
                    let ret = program.run_from_file(filename);

                    if files.len() > 1 {
                        print!("{}: ", filename);
                    }

                    match ret {
                        Ok(None) => {
                            if files.len() > 0 {
                                print!("\n")
                            }
                        }
                        Ok(Some(value)) => println!("{}", value.borrow()),
                        Err(error) => println!("{}", error),
                    }
                }
            } else {
                match program.run_from_str("") {
                    Ok(None) => {}
                    Ok(Some(value)) => println!("{}", value.borrow()),
                    Err(error) => println!("{}", error),
                }
            }
        }
    } else {
        print_version();
        repl(files);
    }
}
