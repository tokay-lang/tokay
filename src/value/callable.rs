use std::any::{Any, TypeId};

use super::Dict;
use crate::vm::{Accept, Context, Reject};

// Callable
// ----------------------------------------------------------------------------

/// Describes an interface to a callable object.
pub trait Callable: std::any::Any + std::fmt::Debug {
    // Returns the callables's id.
    fn id(&self) -> usize;

    // Returns the callable's name.
    fn name(&self) -> &str;

    /// Check whether the callable accepts any arguments.
    fn is_callable(&self, with_arguments: bool) -> bool;

    /// Check whether the callable is consuming
    fn is_consuming(&self) -> bool;

    /// Check whether the callable is nullable
    fn is_nullable(&self) -> bool {
        false
    }

    /// Call a value with a given context, argument and named argument set.
    fn call(
        &self,
        _context: &mut Context,
        _args: usize,
        _nargs: Option<Dict>,
    ) -> Result<Accept, Reject>;

    fn clone_dyn(&self) -> Box<dyn Callable>;
}

// The next piece of code including the comment was kindly borrowed from
// https://gitlab.freedesktop.org/dbus/zbus/-/blob/main/zbus/src/interface.rs#L123
//
// Note: while it is possible to implement this without `unsafe`, it currently requires a helper
// trait with a blanket impl that creates `dyn Any` refs.  It's simpler (and more performant) to
// just check the type ID and do the downcast ourself.
//
// See https://github.com/rust-lang/rust/issues/65991 for a rustc feature that will make it
// possible to get a `dyn Any` ref directly from a `dyn Interface` ref; once that is stable, we can
// remove this unsafe code.
impl dyn Callable {
    /// Return Any of self
    pub(crate) fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if <dyn Callable as Any>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: If type ID matches, it means object is of type T
            Some(unsafe { &*(self as *const dyn Callable as *const T) })
        } else {
            None
        }
    }

    /// Return Any of self
    pub(crate) fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if <dyn Callable as Any>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: If type ID matches, it means object is of type T
            Some(unsafe { &mut *(self as *mut dyn Callable as *mut T) })
        } else {
            None
        }
    }
}

/*
Value could make use of Box<dyn Object> as a trait object, but this requires implementation
of several other trait on Box<dyn Object>. But this looses the possibility of doing PartialEq
and PartialOrd on the current implementation, which IS important.

Here is the link for a playground started on this:
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4d7fda9b8391506736837f93124a16f4

fixme: Need help with this!
*/

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl PartialEq for Box<dyn Callable> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialOrd for Box<dyn Callable> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

// https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for Box<dyn Callable> {
    fn eq(&self, other: &&Self) -> bool {
        self.id() == other.id()
    }
}
