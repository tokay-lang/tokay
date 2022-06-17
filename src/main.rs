//! Tokay main executable
use clap::Parser;
use std::cell::RefCell;
use std::fs::{self, File};
use tokay::compiler::Compiler;
use tokay::repl::{repl, Stream};
use tokay::Object;

fn print_version() {
    println!("Tokay {}", env!("CARGO_PKG_VERSION"));
}

#[derive(Parser)]
#[clap(
    name = "Tokay",
    author,
    version,
    about,
    help_template = r#"{bin} {version}
© 2022 by {author}
{about}

{all-args}

PROGRAM and INPUT are directly used as input strings in case no file with the
given name exists. Use '-f' to disable this behavior. Specify '-' to use stdin
as input file.

When PROGRAM was not specified, {bin} turns into an interactive REPL.

Visit https://tokay.dev/ for help and further information.
{bin} is free software released under the MIT license."#
)]
struct Opts {
    /// Program to compile and run.
    #[clap(value_parser)]
    program: Option<String>,

    /// Input for program to operate on.
    #[clap(value_parser, last = true)]
    input: Vec<String>,

    /// Accept only files as parameters, no string fallbacks.
    #[clap(short, long, action)]
    files: bool,

    /// Sets the debug level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Show license agreement and exit.
    #[clap(short, long, action)]
    license: bool,
}

fn main() {
    let opts = Opts::parse();

    if opts.debug > 0 {
        std::env::set_var("TOKAY_DEBUG", format!("{}", opts.debug));
    }

    if opts.license {
        print_version();
        println!("{}", include_str!("../LICENSE"));
        std::process::exit(0);
    }

    let mut program: Option<Stream> = None;
    let mut streams: Vec<(&str, RefCell<Stream>)> = Vec::new();

    if let Some(prog) = &opts.program {
        if prog == "-" && !opts.files {
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
                if !opts.files {
                    program = Some(Stream::String(prog.to_string()))
                } else {
                    println!("Can't open program '{}'", prog);
                    std::process::exit(1);
                }
            }
        }
    }

    // Try getting files to run on program or repl
    for filename in &opts.input {
        if filename == "-" && !opts.files {
            streams.push((filename, RefCell::new(Stream::Stdin)))
        } else if let Ok(file) = File::open(filename) {
            streams.push((filename, RefCell::new(Stream::File(file))))
        } else if !opts.files {
            streams.push((filename, RefCell::new(Stream::String(filename.to_string()))))
        } else {
            println!("Can't open file '{}'", filename);
            std::process::exit(1);
        }
    }

    if let Some(mut program) = program {
        let mut compiler = Compiler::new(true);

        if compiler.compile(program.get_reader()).is_ok() {
            if let Ok(program) = compiler.finalize() {
                // In case no stream but a program is specified, use stdin as input stream.
                if streams.len() == 0 {
                    streams.push((
                        "",
                        // When program's main is consuming, read from stdin
                        if program.main().is_consuming() {
                            RefCell::new(Stream::Stdin)
                        }
                        // otherwise just work on an empty input
                        else {
                            RefCell::new(Stream::String("".to_string()))
                        },
                    ));
                }

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
                        Ok(Some(value)) => println!("{}", value.to_string()),
                        Err(error) => eprintln!("{}", error),
                    }
                }
            }
        }
    } else {
        print_version();

        // In case no stream was specified and REPL fires up, use empty string as input stream.
        if streams.len() == 0 {
            streams.push(("", RefCell::new(Stream::String("".to_string()))));
        }

        repl(streams);
    }
}
