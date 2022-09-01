//! Tokay main executable
use clap::Parser;
use rustyline;
use std::cell::RefCell;
use std::fs::{self, File};
use tokay::compiler::Compiler;
use tokay::repl::{repl, Stream};
use tokay::Object;
use tokay::Reader;

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

    /// Start the given PROGRAM in its own REPL.
    #[clap(short, long, action)]
    repl: bool,

    /// Sets the debug level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Show license agreement and exit.
    #[clap(short, long, action)]
    license: bool,
}

fn main() {
    // Handle command-line arguments from Opts.
    let opts = Opts::parse();

    // Set TOKAY_DEBUG when debug flag was set.
    if opts.debug > 0 {
        std::env::set_var("TOKAY_DEBUG", format!("{}", opts.debug));
    }

    // Show license and exit?
    if opts.license {
        print_version();
        print!("{}", include_str!("../LICENSE"));
        std::process::exit(0);
    }

    // Read program, either from stdin, file or direct string.
    let mut program: Option<Stream> = None;

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
                    eprintln!("Can't open PROGRAM file '{}'", prog);
                    std::process::exit(1);
                }
            }
        }
    }

    // Try getting files to run on program or repl
    let mut streams: Vec<(&str, RefCell<Stream>)> = Vec::new();

    for filename in &opts.input {
        if filename == "-" && !opts.files {
            streams.push((filename, RefCell::new(Stream::Stdin)))
        } else if let Ok(file) = File::open(filename) {
            streams.push((filename, RefCell::new(Stream::File(file))))
        } else if !opts.files {
            streams.push((filename, RefCell::new(Stream::String(filename.to_string()))))
        } else {
            eprintln!("Can't open INPUT file '{}'", filename);
            std::process::exit(1);
        }
    }

    if let Some(mut program) = program {
        let mut compiler = Compiler::new(true);

        if let Ok(Some(program)) = compiler.compile(program.get_reader()) {
            // In case no stream but a program is specified, use stdin as input stream.
            if streams.len() == 0 {
                // Run program in its own REPL?
                if opts.repl {
                    let mut readline = rustyline::Editor::<()>::new();
                    readline.load_history(".tokayrepl").ok();

                    loop {
                        let code = match readline.readline("<<< ") {
                            Err(rustyline::error::ReadlineError::Interrupted)
                            | Err(rustyline::error::ReadlineError::Eof) => break,
                            Err(err) => {
                                eprintln!("Error {:?}", err);
                                break;
                            }

                            Ok(code) => code,
                        };

                        // Stop when program is empty.
                        if code.trim().is_empty() {
                            continue;
                        }

                        readline.add_history_entry(code.as_str());

                        match program
                            .run_from_reader(Reader::new(Box::new(std::io::Cursor::new(code))))
                        {
                            Ok(None) => {
                                if streams.len() > 1 {
                                    print!("\n")
                                }
                            }
                            Ok(Some(value)) => println!("{}", value.to_string()),
                            Err(error) => eprintln!("{}", error),
                        }
                    }

                    readline.save_history(".tokayrepl").unwrap();
                    std::process::exit(0);
                }

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

            if opts.repl {
                eprintln!("REPL-mode not allowed in combination with provided INPUT");
                std::process::exit(1);
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
    } else {
        if opts.repl {
            eprintln!("No PROGRAM was specified, therefore can't turn into a REPL for PROGRAM");
            std::process::exit(1);
        }

        print_version();

        // In case no stream was specified and REPL fires up, use empty string as input stream.
        if streams.len() == 0 {
            streams.push(("", RefCell::new(Stream::String("".to_string()))));
        }

        repl(streams);
    }
}
