use super::Dict;
use crate::vm::{Accept, Context, Reject};
use std::any::Any;

// BoxedObject
// ----------------------------------------------------------------------------

pub type BoxedObject = Box<dyn Object>;

// AnyBoxedObject
// ----------------------------------------------------------------------------

pub trait AnyBoxedObject {
    fn clone_dyn(&self) -> BoxedObject;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

impl<T> AnyBoxedObject for T
where
    T: 'static + Object + Clone,
{
    fn clone_dyn(&self) -> BoxedObject {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for BoxedObject {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

// Object
// ----------------------------------------------------------------------------

/// Describes an interface to a callable object.
pub trait Object: AnyBoxedObject + std::any::Any + std::fmt::Debug {
    /// Object ID (unique memory address)
    fn id(&self) -> usize {
        self as *const Self as *const () as usize
    }

    /// Object type name.
    fn name(&self) -> &'static str;

    /// Object representation in Tokay code
    fn repr(&self) -> String {
        format!("<{} {:p}>", self.name(), self)
    }

    /// Object as bool
    fn is_true(&self) -> bool {
        true
    }

    /// Object as i64
    fn to_i64(&self) -> i64 {
        0
    }

    /// Object as f64
    fn to_f64(&self) -> f64 {
        0.0
    }

    /// Object as usize
    fn to_usize(&self) -> usize {
        self.id()
    }

    /// Object as String
    fn to_string(&self) -> String {
        self.repr()
    }

    /// Check whether the object is callable and accepts any arguments.
    fn is_callable(&self, _with_arguments: bool) -> bool {
        false
    }

    /// Check whether the object is consuming
    fn is_consuming(&self) -> bool {
        false
    }

    /// Check whether the object is nullable
    fn is_nullable(&self) -> bool {
        false
    }

    /// Call object with a given context, argument and named argument set.
    fn call(
        &self,
        _context: &mut Context,
        _args: usize,
        _nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        panic!("{} cannot be called.", self.name())
    }
}

/*
Value could make use of BoxedObject as a trait object, but this requires implementation
of several other trait on BoxedObject. But this looses the possibility of doing PartialEq
and PartialOrd on the current implementation, which IS important.

Here is the link for a playground started on this:
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4d7fda9b8391506736837f93124a16f4

fixme: Need help with this!
*/

impl PartialEq for BoxedObject {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialOrd for BoxedObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

// https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for BoxedObject {
    fn eq(&self, other: &&Self) -> bool {
        self.id() == other.id()
    }
}

/*
impl<T: Object> From<Box<T>> for RefValue {
    fn from(value: Box<T>) -> Self {
        Value::Object(value).into()
    }
}
*/
