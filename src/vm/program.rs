use std::cell::RefCell;
use std::io::prelude::*;
use std::io::BufReader;
use std::rc::Rc;

use super::*;
use crate::reader::Reader;
use crate::value::{RefValue, Value};

#[derive(Debug)]
pub struct Program {
    pub(super) statics: Vec<RefValue>,
    main: Rc<RefCell<Parselet>>,
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

        if main.is_none() {
            panic!("No main parselet found!");
        }

        Self {
            statics,
            main: main.unwrap(),
        }
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Option<RefValue>, Option<String>> {
        let main = self.main.borrow();
        let res = main.run(runtime, 0, true);

        match res {
            Ok(Accept::Push(capture)) => {
                //println!("capture = {:?}", capture.as_value(runtime));
                Ok(Some(capture.as_value(runtime)))
            }
            Ok(_) => Ok(None),
            Err(Reject::Error(msg)) => Err(Some(msg)),
            Err(_) => Err(None),
        }
    }

    pub fn run_from_reader<R: 'static + Read>(
        &self,
        read: R,
    ) -> Result<Option<RefValue>, Option<String>> {
        let mut reader = Reader::new(Box::new(BufReader::new(read)));
        let mut runtime = Runtime::new(&self, &mut reader);

        let ret = self.run(&mut runtime);

        // tmp: report unconsumed input
        if let Some(ch) = reader.peek() {
            println!("Input was not fully consumed, next character is {:?}", ch);
        }

        ret
    }

    pub fn run_from_str(&self, s: &'static str) -> Result<Option<RefValue>, Option<String>> {
        self.run_from_reader(std::io::Cursor::new(s))
    }
}
