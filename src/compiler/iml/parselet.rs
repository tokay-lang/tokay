//! Intermediate representation of a parselet
use super::*;
use crate::reader::Offset;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
/// Intermediate parselet
pub(in crate::compiler) struct ImlParselet {
    pub offset: Option<Offset>,                     // Offset of definition
    pub consuming: bool, // Flag if parselet is consuming (detected by compiler scopes)
    pub severity: u8,    // Capture push severity
    pub name: Option<String>, // Parselet's name from source (for debugging)
    pub constants: Vec<(String, Option<ImlValue>)>, // Constant signature with default constants; generic parselet when set.
    pub signature: Vec<(String, Option<ImlValue>)>, // Argument signature with default arguments
    pub locals: usize, // Total number of local variables present (including arguments)
    pub begin: ImlOp,  // Begin-operations
    pub end: ImlOp,    // End-operations
    pub body: ImlOp,   // Operations
}

/** Representation of parselet in intermediate code. */
impl ImlParselet {
    pub fn id(&self) -> usize {
        self as *const ImlParselet as usize
    }

    pub fn finalize(&self, configs: &mut HashMap<usize, Consumable>) -> bool {
        let mut changes = false;
        let id = self.id();

        for part in [&self.begin, &self.body, &self.end] {
            if let Some(result) = part.finalize(&mut HashSet::new(), configs) {
                if !configs.contains_key(&id) || configs[&id] < result {
                    configs.insert(id, result);
                    changes = true;
                }
            }
        }

        changes
    }
}

impl std::cmp::PartialEq for ImlParselet {
    // It satisfies to just compare the parselet's memory address for equality
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for ImlParselet {}

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
