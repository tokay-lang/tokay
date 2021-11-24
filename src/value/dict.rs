//! Dictionary object

use super::RefValue;
use std::collections::BTreeMap;

pub type Dict = BTreeMap<String, RefValue>;
