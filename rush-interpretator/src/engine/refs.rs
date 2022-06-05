use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ref(usize);

impl From<usize> for Ref {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl Ref {
    #[must_use]
    pub const fn new(ptr: usize) -> Self {
        Self(ptr)
    }
}

impl Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ref({})", self.0)
    }
}

impl Add<usize> for Ref {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<Self> for Ref {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<usize> for Ref {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl AddAssign<Self> for Ref {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
