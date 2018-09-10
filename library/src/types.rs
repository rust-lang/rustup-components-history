//! Types that are used internally.

use std::borrow::Borrow;
use std::ops::Deref;
use std::rc::Rc;

/// Reference-counted build-target triple.
#[derive(Debug, Clone, Hash, PartialEq, Display, Eq)]
pub struct TargetTripple(Rc<str>);

impl Deref for TargetTripple {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for TargetTripple {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for TargetTripple
where
    Rc<str>: From<T>,
{
    fn from(t: T) -> Self {
        TargetTripple(t.into())
    }
}

/// Reference-counted package name.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PackageName(Rc<str>);

impl Deref for PackageName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PackageName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for PackageName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for PackageName
where
    Rc<str>: From<T>,
{
    fn from(t: T) -> Self {
        PackageName(t.into())
    }
}
