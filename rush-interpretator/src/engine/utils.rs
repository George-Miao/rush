use std::{fmt::Display, ops::Deref, rc::Rc, sync::RwLock};

#[derive(Debug, PartialEq, Eq)]
pub struct Shared<T: ?Sized>(Rc<T>);

impl<T> Shared<T> {
    pub fn new(v: T) -> Self {
        Self(Rc::new(v))
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Display> Display for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Shared<T> {
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

pub trait IntoShared {
    fn shared(self) -> Shared<Self>;
}

impl<T> IntoShared for T {
    fn shared(self) -> Shared<Self> {
        Shared(Rc::new(self))
    }
}

#[derive(Debug)]
pub struct Locked<T>(RwLock<T>);

impl<T> Locked<T> {
    pub fn new(v: T) -> Self {
        Self(RwLock::new(v))
    }

    pub fn get(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    pub fn get_mut(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }

    pub fn get_with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let lock = self.0.read().unwrap();
        f(&lock)
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut lock = self.0.write().unwrap();
        f(&mut *lock);
    }
}

impl<T> From<T> for Locked<T> {
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

impl<T> From<T> for Shared<Locked<T>> {
    fn from(v: T) -> Self {
        Locked::new(v).into()
    }
}

pub trait ToResult: Sized {
    fn err<T>(self) -> Result<T, Self>;
    fn ok<E>(self) -> Result<Self, E>;
}

impl<Ty> ToResult for Ty {
    fn err<T>(self) -> Result<T, Self> {
        Err(self)
    }

    fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }
}
