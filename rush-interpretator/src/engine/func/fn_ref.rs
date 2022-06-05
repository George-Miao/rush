use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::Ref;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct FnRef {
    fn_ref: Ref,
}

impl FnRef {
    pub const fn new(fn_ref: Ref) -> Self {
        Self { fn_ref }
    }

    #[must_use]
    pub const fn inner(&self) -> Ref {
        self.fn_ref
    }
}

impl From<Ref> for FnRef {
    fn from(ref_: Ref) -> Self {
        Self { fn_ref: ref_ }
    }
}

impl Display for FnRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "FnRef({}, {})", self.fn_ref, self)
        write!(f, "FnRef({})", self.fn_ref)
    }
}

impl Deref for FnRef {
    type Target = Ref;

    fn deref(&self) -> &Ref {
        &self.fn_ref
    }
}

impl DerefMut for FnRef {
    fn deref_mut(&mut self) -> &mut Ref {
        &mut self.fn_ref
    }
}
