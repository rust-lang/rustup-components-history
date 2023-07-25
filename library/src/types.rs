//! Types that are used internally.

use std::fmt;
use std::{borrow::Borrow, ops::Deref, rc::Rc};

/// Reference-counted build-target triple.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TargetTriple(Rc<str>);

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for TargetTriple {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for TargetTriple {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for TargetTriple
where
    Rc<str>: From<T>,
{
    fn from(t: T) -> Self {
        TargetTriple(t.into())
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
