//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use crate::value::Parselet;
use std::cell::RefCell;

#[derive(Debug)]
/// Intermediate parselet
pub struct ImlParselet {
    pub offset: Option<Offset>, // Offset of definition
    //pub consuming: Option<RefCell<Consumable>>,              // Consumable state
    pub consuming: Option<Consumable>,
    pub severity: u8,                               // Capture push severity
    pub name: Option<String>,                       // Parselet's name from source (for debugging)
    pub constants: Vec<(String, Option<ImlValue>)>, // Constant signature with default constants; generic parselet when set.
    pub signature: Vec<(String, Option<ImlValue>)>, // Argument signature with default arguments
    locals: usize, // Total number of local variables present (including arguments)
    begin: ImlOp,  // Begin-operations
    end: ImlOp,    // End-operations
    body: ImlOp,   // Operations
}

impl ImlParselet {
    /// Creates a new intermediate parselet.
    pub fn new(
        offset: Option<Offset>,
        name: Option<String>,
        constants: Vec<(String, Option<ImlValue>)>,
        signature: Vec<(String, Option<ImlValue>)>,
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
            offset,
            name,
            consuming: None,
            severity: 5,
            constants,
            signature,
            locals,
            begin,
            end,
            body,
        }
    }

    pub fn id(&self) -> usize {
        self as *const ImlParselet as usize
    }

    /// Turns an intermediate parselet in to a fixed Parselet
    pub fn into_parselet(
        &self, /* fixme: change to self without & later on... */
        module: &mut Program,
    ) -> Parselet {
        Parselet::new(
            self.name.clone(),
            if let Some(Consumable { leftrec, .. }) = self.consuming {
                Some(leftrec)
            } else {
                None
            },
            self.severity,
            self.signature
                .iter()
                .map(|var_value| {
                    (
                        var_value.0.clone(),
                        if let Some(value) = &var_value.1 {
                            Some(module.define_static(value.clone().unwrap()))
                        } else {
                            None
                        },
                    )
                })
                .collect(),
            self.locals,
            self.begin.compile(module),
            self.end.compile(module),
            self.body.compile(module),
        )
    }

    pub fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(stack)
    }
}

impl std::cmp::PartialEq for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl std::hash::Hash for ImlParselet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl std::cmp::PartialOrd for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}
