use super::Dict;
use crate::{Accept, Context, RefValue, Reject};
use num_bigint::BigInt;
use std::any::Any;

// BoxedObject
// ----------------------------------------------------------------------------

pub type BoxedObject = Box<dyn Object>;

// AnyBoxedObject
// ----------------------------------------------------------------------------

pub trait AnyBoxedObject {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

impl<T> AnyBoxedObject for T
where
    T: 'static + Object,
{
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

// CloneBoxedObject
// ----------------------------------------------------------------------------

pub trait CloneBoxedObject {
    fn dyn_clone(&self) -> BoxedObject;
}

impl<T> CloneBoxedObject for T
where
    T: 'static + Object + Clone,
{
    fn dyn_clone(&self) -> BoxedObject {
        Box::new(self.clone())
    }
}

impl Clone for BoxedObject {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

// PartialEqBoxedObject
// ----------------------------------------------------------------------------

pub trait PartialEqBoxedObject {
    fn dyn_eq(&self, other: &BoxedObject) -> bool;
}

impl<T> PartialEqBoxedObject for T
where
    T: 'static + Object + PartialEq,
{
    fn dyn_eq(&self, other: &BoxedObject) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self.eq(other)
        } else {
            false
        }
    }
}

impl PartialEq for BoxedObject {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

// fix for `move occurs because `*__arg_1_0` has type `Box<dyn Obj>`, which does not implement the `Copy` trait`
// https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for BoxedObject {
    fn eq(&self, other: &&Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for BoxedObject {}

// PartialOrdBoxedObject
// ----------------------------------------------------------------------------

pub trait PartialOrdBoxedObject {
    fn dyn_partial_cmp(&self, other: &BoxedObject) -> Option<std::cmp::Ordering>;
}

impl<T> PartialOrdBoxedObject for T
where
    T: 'static + Object + PartialEq + PartialOrd,
{
    fn dyn_partial_cmp(&self, other: &BoxedObject) -> Option<std::cmp::Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self.partial_cmp(other)
        } else {
            None
        }
    }
}

impl PartialOrd for BoxedObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dyn_partial_cmp(other)
    }
}

// Object
// ----------------------------------------------------------------------------

/// Describes an interface to a callable object.
pub trait Object:
    AnyBoxedObject
    + CloneBoxedObject
    + PartialEqBoxedObject
    + PartialOrdBoxedObject
    + std::any::Any
    + std::fmt::Debug //+ std::fmt::Display
{
    /// Object ID (unique memory address)
    fn id(&self) -> usize {
        self as *const Self as *const () as usize
    }

    /// Object severity
    fn severity(&self) -> u8 {
        0
    }

    /// Object type name.
    fn name(&self) -> &'static str;

    /// Check for value type name.
    fn is(&self, name: &str) -> bool {
        self.name() == name
    }

    /// Object representation in Tokay code
    fn repr(&self) -> String {
        format!("<{} {:p}>", self.name(), self)
    }

    /// Object as void
    fn is_void(&self) -> bool {
        false // by default, every object is never void
    }

    /// Object as bool
    fn is_true(&self) -> bool {
        true // by default, every object is boolean true
    }

    /// Object as i64
    fn to_i64(&self) -> Result<i64, String> {
        Err(format!("`{}` cannot be converted to i64", self.name()))
    }

    /// Object as f64
    fn to_f64(&self) -> Result<f64, String> {
        Err(format!("`{}` cannot be converted to f64", self.name()))
    }

    /// Object as usize
    fn to_usize(&self) -> Result<usize, String> {
        Err(format!("`{}` cannot be converted to usize", self.name()))
    }

    /// Object as String
    fn to_string(&self) -> String {
        self.repr()
    }

    /// Object as BigInt
    fn to_bigint(&self) -> Result<BigInt, String> {
        Err(format!("`{}` cannot be converted to BigInt", self.name()))
    }

    /// Check whether the object is callable.
    fn is_callable(&self, _without_arguments: bool) -> bool {
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

    /** Check whether the object is mutable in itself.

    Most objects are not mutable, they represent a single value that doesn't change.
    For example, Tokay's `str`-objects are not mutable in itself, `str`-methods do always return
    a new `str`-object. This is different for `list` or `dict`, they are mutable in theirself.

    The mutability setting changes stack behavior: Objects are (mostly) referenced when they are
    mutable, and copied when they are imutable.
    */
    fn is_mutable(&self) -> bool {
        false
    }

    /** Check whether the object is hashable.

    By default, this depends on is_mutable(), but could also be customized.
    This defines if an object can be used as key in dicts.
    */
    fn is_hashable(&self) -> bool {
        !self.is_mutable()
    }

    /// Call object with optional context, arguments and named arguments set.
    fn call(
        &self,
        _context: Option<&mut Context>,
        _args: Vec<RefValue>,
        _nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        Err(format!("'{}' is not callable", self.name()).into())
    }

    /** Directly call object with a given stack configuration.

    This leads in lesser stack operations as previously pushed stack items can be
    used directly as local variables without change, which is the case in parselets.
    */
    fn call_direct(
        &self,
        context: &mut Context,
        args: usize,
        nargs: Option<Dict>,
    ) -> Result<Accept, Reject> {
        // Default drains the context as builtins do
        let args = context.drain(args);
        self.call(Some(context), args, nargs)
    }
}
