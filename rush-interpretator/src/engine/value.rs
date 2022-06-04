use std::{
    fmt::{self, Debug},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use parser::ast::{Literal, LiteralKind};
use sealed::sealed;

use crate::{FnRef, IntoShared, Locked, RuntimeError, RuntimeResult, Shared};

#[allow(clippy::pedantic)]
pub type SharedValue = Shared<Locked<Value>>;

impl SharedValue {
    #[must_use]
    pub fn new_shared(v: Value) -> Self {
        Self::new(Locked::new(v))
    }

    pub fn get(&self) -> RwLockReadGuard<Value> {
        Locked::get(self)
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<Value> {
        Locked::get_mut(self)
    }
}

#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Shared<String>),
    Fn(FnRef),
    Unit,
}

impl Value {
    pub fn new(variant: impl Variant) -> Self {
        variant.into_value()
    }

    #[must_use]
    pub fn share(self) -> SharedValue {
        self.into()
    }

    #[must_use]
    pub fn type_name(&self) -> &str {
        map_value! {
            self,
            val => { val.type_name() },
            Unit => "unit",
        }
    }

    #[must_use]
    pub fn ty_eq(&self, other: &Self) -> bool {
        self.type_name() == other.type_name()
    }

    #[must_use]
    pub fn ty_eq_name(&self, other: &Self) -> Option<&str> {
        self.ty_eq(other).then_some(self.type_name())
    }

    pub fn cast<T: Variant>(self) -> Result<T, Self> {
        T::from_value(self)
    }

    pub fn cast_ref<T: Variant>(&self) -> Result<&T, &Self> {
        T::from_value_ref(self)
    }

    pub fn rt_cast<T: Variant>(self, ident: &str) -> RuntimeResult<T> {
        T::from_value(self).map_err(|t| RuntimeError::TypeError {
            ident: ident.to_owned(),
            expected: T::TYPE_NAME.to_owned(),
            found: t.type_name().to_owned(),
        })
    }

    pub fn rt_cast_ref<T: Variant>(&self, ident: &str) -> RuntimeResult<&T> {
        T::from_value_ref(self).map_err(|t| RuntimeError::TypeError {
            ident: ident.to_owned(),
            expected: T::TYPE_NAME.to_owned(),
            found: t.type_name().to_owned(),
        })
    }
}

impl From<Literal<'_>> for Value {
    fn from(lit: Literal) -> Self {
        match lit.kind {
            LiteralKind::Number(val) => Self::Int(val),
            LiteralKind::Bool(b) => Self::Bool(b),
            LiteralKind::String(s) => Self::Str(s.to_owned().shared()),
            LiteralKind::Float(f) => Self::Float(f),
            l => unimplemented!("Literal type not implemented yet: {l:#?}"),
        }
    }
}

impl From<&Literal<'_>> for Value {
    fn from(lit: &Literal) -> Self {
        match &lit.kind {
            LiteralKind::Number(val) => Self::Int(*val),
            LiteralKind::Bool(b) => Self::Bool(*b),
            LiteralKind::Float(f) => Self::Float(*f),
            LiteralKind::String(s) => Self::Str((*s).to_owned().shared()),
            l => unimplemented!("Literal type not implemented yet: {l:#?}"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        map_value! {
            self,
            val => { fmt::Display::fmt(val, f) },
            Unit => write!(f, "()"),
        }
    }
}

macro_rules! map_value {
    ($val:ident, $id:ident => $act:expr,Unit => $act2:expr $(,)?) => {
        match $val {
            Value::Int($id) => $act,
            Value::Float($id) => $act,
            Value::Bool($id) => $act,
            Value::Str($id) => $act,
            Value::Fn($id) => $act,
            Value::Unit => $act2,
        }
    };
}

use map_value;

#[sealed]
pub trait Variant: Sized {
    const TYPE_NAME: &'static str;

    fn type_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
    fn from_value(value: Value) -> Result<Self, Value>;
    fn from_value_ref(value: &Value) -> Result<&Self, &Value>;
    fn into_value(self) -> Value;
}

#[sealed]
impl Variant for () {
    const TYPE_NAME: &'static str = "unit";

    fn from_value(val: Value) -> Result<Self, Value> {
        match val {
            Value::Unit => Ok(()),
            other => Err(other),
        }
    }

    fn from_value_ref(value: &Value) -> Result<&Self, &Value> {
        match value {
            Value::Unit => Ok(&()),
            other => Err(other),
        }
    }

    fn into_value(self) -> Value {
        Value::Unit
    }
}

macro_rules! impl_varaint {
    ($t:ty, $variant:ident, $name:literal $(, $extra_t:ty)* $(,)?) => {
        #[sealed]
        impl Variant for $t {
            const TYPE_NAME: &'static str = $name;

            fn from_value(value: Value) -> Result<Self, Value> {
                match value {
                    Value::$variant(v) => Ok(v),
                    other => Err(other),
                }
            }

            fn from_value_ref<'b>(value: &'b Value) -> Result<&'b Self, &'b Value> {
                match value {
                    Value::$variant(v) => Ok(v),
                    other => Err(other),
                }
            }

            fn into_value(self) -> Value {
                Value::$variant(self)
            }
        }

        impl FromValue for $t {
            fn from_value(ident: &str, value: Value) -> RuntimeResult<Self> {
                value.rt_cast(ident)
            }
        }
    };
}

impl_varaint!(i64, Int, "int");
impl_varaint!(f64, Float, "float");
impl_varaint!(bool, Bool, "bool");
impl_varaint!(Shared<String>, Str, "str");
impl_varaint!(FnRef, Fn, "fn");

#[allow(clippy::module_name_repetitions)]
pub trait FromValue: Sized {
    fn from_value(ident: &str, value: Value) -> RuntimeResult<Self>;
}
