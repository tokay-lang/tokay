use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::value::{Parselet, Value};

/** Parselet is the conceptual building block of a Tokay program.

A parselet is like a function in ordinary programming languages, with the
exception that it can either be a snippet of parsing instructions combined with
semantic code, or just an ordinary function consisting of code and returning
values. The destinction if a parselet represents just a function or a parselet is
done by the consuming-flag, which is determined by use of static tokens, parselets
and consuming builtins.

Parselets support static program constructs being left-recursive, and extend
the generated parse tree automatically until no more input can be consumed.
*/

#[derive(Debug)]
pub struct ImlParselet {
    pub(crate) consuming: Option<Consumable>, // Consumable state
    pub(crate) severity: u8,                  // Capture push severity
    pub(crate) name: Option<String>,          // Parselet's name from source (for debugging)
    signature: Vec<(String, Option<usize>)>,  // Argument signature with default arguments
    pub(crate) locals: usize,                 // Number of local variables present
    begin: ImlOp,                             // Begin-operations
    end: ImlOp,                               // End-operations
    body: ImlOp,                              // Operations
}

impl ImlParselet {
    /// Creates a new parselet.
    pub fn new(
        name: Option<String>,
        signature: Vec<(String, Option<usize>)>,
        locals: usize,
        begin: ImlOp,
        end: ImlOp,
        body: ImlOp,
    ) -> Self {
        assert!(
            signature.len() <= locals,
            "signature may not be longer than locals..."
        );

        Self {
            name,
            consuming: None,
            severity: 5,
            signature,
            locals,
            begin,
            end,
            body,
        }
    }

    /// Turns parselet into a Value
    pub fn into_value(self) -> Value {
        Value::ImlParselet(Rc::new(RefCell::new(self)))
    }

    // Turns an ImlParselet in to a parselet
    pub fn into_parselet(&self /* fixme: change to self without & later on... */) -> Parselet {
        Parselet::new(
            self.name.clone(),
            self.consuming.clone(),
            self.severity,
            self.signature.clone(),
            self.locals,
            self.begin.compile(&self),
            self.end.compile(&self),
            self.body.compile(&self),
        )
    }

    pub(in crate::compiler) fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.begin.resolve(usages);
        self.end.resolve(usages);
        self.body.resolve(usages);
    }

    pub(in crate::compiler) fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(statics, stack)
    }

    // Checks if parselet is callable with or without arguments
    pub(crate) fn is_callable(&self, with_arguments: bool) -> bool {
        // Either without arguments and signature is empty or all arguments have default values
        (!with_arguments && (self.signature.len() == 0 || self.signature.iter().all(|arg| arg.1.is_some())))
        // or with arguments and signature exists
            || (with_arguments && self.signature.len() > 0)
    }
}

impl std::cmp::PartialEq for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self as *const ImlParselet as usize == other as *const ImlParselet as usize
    }
}

impl std::hash::Hash for ImlParselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const ImlParselet as usize).hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let left = self as *const ImlParselet as usize;
        let right = other as *const ImlParselet as usize;

        left.partial_cmp(&right)
    }
}
