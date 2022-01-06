use std::fs::File;
use std::io::{self, BufReader};

use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::value::{RefValue, Value}; // todo: temporary! // todo: temporary!

/** Programs are containers holding statics and a pointer to the main parselet.

A program is the result of a successful compiler run. */
#[derive(Debug)]
pub struct Program {
    pub(crate) statics: Vec<RefValue>, // Static values referenced by this program
    main: Option<usize>,               // The main parselet to run
}

impl Program {
    pub fn new(statics: Vec<Value>) -> Self {
        let mut main = None;

        // Find main parselet by selecting the last parselet defined.
        // todo: allow to specify main parselet.
        for i in (0..statics.len()).rev() {
            if matches!(statics[i], Value::Parselet(_)) {
                main = Some(i);
                break;
            }
        }

        Self {
            statics: statics.into_iter().map(|value| value.into()).collect(),
            main,
        }
    }

    pub fn dump(&self) {
        for i in 0..self.statics.len() {
            println!("{} => {:#?}", i, self.statics[i]);
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Option<Value>, Error> {
        if let Some(main) = self.main {
            match match &*self.statics[main].borrow() {
                Value::Parselet(main) => {
                    main.borrow()
                        .run(runtime, runtime.stack.len(), None, true, 0)
                }
                _ => panic!(),
            } {
                Ok(Accept::Push(Capture::Value(value, ..))) => {
                    let value: Value = value.into();
                    match value {
                        Value::Void => Ok(None),
                        other => Ok(Some(other)),
                    }
                }
                Ok(_) => Ok(None),
                Err(Reject::Error(error)) => Err(*error),
                Err(other) => Err(Error::new(None, format!("Runtime error {:?}", other))),
            }
        } else {
            Ok(None)
        }
    }

    pub fn run_from_reader(&self, mut reader: Reader) -> Result<Option<Value>, Error> {
        let mut runtime = Runtime::new(&self, &mut reader);
        self.run(&mut runtime)
    }

    pub fn run_from_str(&self, src: &'static str) -> Result<Option<Value>, Error> {
        self.run_from_reader(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    pub fn run_from_string(&self, src: String) -> Result<Option<Value>, Error> {
        self.run_from_reader(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    pub fn run_from_file(&self, filename: &str) -> Result<Option<Value>, Error> {
        if filename == "-" {
            self.run_from_reader(Reader::new(Box::new(BufReader::new(io::stdin()))))
        } else if let Ok(file) = File::open(filename) {
            self.run_from_reader(Reader::new(Box::new(BufReader::new(file))))
        } else {
            Err(Error::new(
                None,
                format!("Unable to read from filename '{}'", filename),
            ))
        }
    }
}
