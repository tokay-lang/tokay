//! Tokay main executable

use ::tokay::compiler::Compiler;
use ::tokay::repl::{repl, Stream};

#[macro_use]
extern crate clap;
use clap::App;

use std::cell::RefCell;
use std::fs::{self, File};

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

    let debug = opts.occurrences_of("debug");
    if debug > 0 {
        std::env::set_var("TOKAY_DEBUG", format!("{}", debug));
    }

    if opts.is_present("license") {
        print_version();
        println!("{}", include_str!("../LICENSE"));
        std::process::exit(0);
    }

    let files_only = opts.is_present("files");

    let mut program: Option<Stream> = None;
    let mut streams: Vec<(&str, RefCell<Stream>)> = Vec::new();

    if let Some(prog) = opts.value_of("program") {
        if prog == "-" && !files_only {
            program = Some(Stream::Stdin)
        } else {
            if let Some(meta) = fs::metadata(prog).ok() {
                if !meta.is_dir() {
                    if let Ok(file) = File::open(prog) {
                        program = Some(Stream::File(file));
                    }
                }
            }

            if program.is_none() {
                if !files_only {
                    program = Some(Stream::String(prog.to_string()))
                } else {
                    println!("Can't open program '{}'", prog);
                    std::process::exit(1);
                }
            }
        }
    }

    // Try getting files to run on program or repl
    if let Some(files) = opts.values_of("input") {
        for filename in files {
            if filename == "-" && !files_only {
                streams.push((filename, RefCell::new(Stream::Stdin)))
            } else if let Ok(file) = File::open(filename) {
                streams.push((filename, RefCell::new(Stream::File(file))))
            } else if !files_only {
                streams.push((filename, RefCell::new(Stream::String(filename.to_string()))))
            } else {
                println!("Can't open file '{}'", filename);
                std::process::exit(1);
            }
        }
    }

    // In case no stream is specified, use empty string as default input stream.
    if streams.len() == 0 {
        streams.push(("", RefCell::new(Stream::String("".to_string()))));
    }

    if let Some(mut program) = program {
        let mut compiler = Compiler::new();

        if let Ok(program) = compiler.compile(program.get_reader()) {
            for (name, stream) in &streams {
                let ret = program.run_from_reader(stream.borrow_mut().get_reader());

                if streams.len() > 1 {
                    print!("{}: ", name);
                }

                match ret {
                    Ok(None) => {
                        if streams.len() > 1 {
                            print!("\n")
                        }
                    }
                    Ok(Some(value)) => println!("{}", value),
                    Err(error) => eprintln!("{}", error),
                }
            }
        }
    } else {
        print_version();
        repl(streams);
    }
}
