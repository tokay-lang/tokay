use super::iml::*;
use crate::vm::Op;
use crate::{RefValue, Value};
use indexmap::IndexMap;
use num::ToPrimitive;

/// The linker glues compiled intermediate program and finalized VM program together.
#[derive(Debug)]
pub(super) struct Linker {
    pub statics: IndexMap<ImlValue, Option<RefValue>>, // static values with optional replacement
}

impl Linker {
    pub fn new() -> Self {
        Linker {
            statics: IndexMap::new(),
        }
    }

    pub fn register_static(&mut self, value: &ImlValue) -> usize {
        match self.statics.get_index_of(value) {
            None => self.statics.insert_full(value.clone(), None).0,
            Some(idx) => idx,
        }
    }

    pub fn push_static(&mut self, value: &ImlValue) -> Op {
        // Primary value pushes can directly be made by specific VM commands
        if let ImlValue::Value(value) = value {
            match &*value.borrow() {
                Value::Void => return Op::PushVoid,
                Value::Null => return Op::PushNull,
                Value::True => return Op::PushTrue,
                Value::False => return Op::PushFalse,
                Value::Int(i) => match i.to_i64() {
                    Some(0) => return Op::Push0,
                    Some(1) => return Op::Push1,
                    _ => {}
                },
                _ => {}
            }
        }

        Op::LoadStatic(self.register_static(value))
    }
}
