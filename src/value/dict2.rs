//! Dictionary object
use super::{BoxedObject, Object, RefValue};
use indexmap::IndexMap;

// Alias for the inner dict
type InnerDict = IndexMap<RefValue, RefValue>;

// Dict object type
#[derive(Debug, Clone, PartialEq)]
pub struct Dict {
    dict: InnerDict,
}
