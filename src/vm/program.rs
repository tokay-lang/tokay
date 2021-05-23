use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufReader};
use std::rc::Rc;

use super::*;
use crate::error::Error;
use crate::reader::Reader;
use crate::value::{RefValue, Value};

#[derive(Debug)]
pub struct Program {
    pub(super) statics: Vec<RefValue>, // Static values referenced by this program
    main: Option<Rc<RefCell<Parselet>>>, // The main parselet to run
}

impl Program {
    pub fn new(statics: Vec<RefValue>) -> Self {
        let mut main = None;

        // Find main parselet by selecting the last parselet defined.
        // todo: allow to specify main parselet.
        for i in (0..statics.len()).rev() {
            if let Value::Parselet(p) = &*statics[i].borrow() {
                main = Some(p.clone());
                break;
            }
        }

        Self { statics, main }
    }

    pub fn dump(&self) {
        for i in 0..self.statics.len() {
            println!("{} => {:#?}", i, self.statics[i]);
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Option<RefValue>, Error> {
        if let Some(main) = &self.main {
            let main = main.borrow();
            let res = main.run(runtime, runtime.stack.len(), None, true);

            let res = match res {
                Ok(Accept::Push(capture)) => Ok(Some(capture.as_value(runtime))),
                Ok(_) => Ok(None),
                Err(Reject::Error(error)) => Err(*error),
                Err(other) => Err(Error::new(None, format!("Runtime error {:?}", other))),
            };

            res
        } else {
            Ok(None)
        }
    }

    pub fn run_from_reader(&self, mut reader: Reader) -> Result<Option<RefValue>, Error> {
        let mut runtime = Runtime::new(&self, &mut reader);
        self.run(&mut runtime)
    }

    pub fn run_from_str(&self, src: &'static str) -> Result<Option<RefValue>, Error> {
        self.run_from_reader(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    pub fn run_from_string(&self, src: String) -> Result<Option<RefValue>, Error> {
        self.run_from_reader(Reader::new(Box::new(BufReader::new(std::io::Cursor::new(
            src,
        )))))
    }

    pub fn run_from_file(&self, filename: &str) -> Result<Option<RefValue>, Error> {
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
