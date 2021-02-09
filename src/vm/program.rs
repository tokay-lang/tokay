use std::cell::RefCell;
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

        let mut program = Self {
            statics,
            main: main.unwrap(),
        };

        program
    }

    pub fn run(&self, runtime: &mut Runtime) -> Result<Option<RefValue>, Option<String>> {
        let main = self.main.borrow();
        let res = main.run(runtime, true);

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

    pub fn run_from_str(&self, s: &'static str) -> Result<Option<RefValue>, Option<String>> {
        let mut reader = Reader::new(Box::new(std::io::Cursor::new(s)));
        let mut runtime = Runtime::new(&self, &mut reader);

        let ret = self.run(&mut runtime);

        // tmp: report unconsumed input
        if let Some(ch) = reader.peek() {
            println!("Input was not fully consumed, next character is {:?}", ch);
        }

        ret
    }
}
