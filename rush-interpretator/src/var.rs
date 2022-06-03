use crate::{Locked, Ref, Shared, SharedValue, Value};

#[derive(Debug)]
pub struct Variable {
    name: String,
    var_ref: Ref,
    value: SharedValue,
}

impl Variable {
    pub fn new(name: impl Into<String>, var_ref: Ref, value: impl Into<SharedValue>) -> Self {
        Self {
            name: name.into(),
            var_ref,
            value: value.into(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> (String, Ref, Shared<Locked<Value>>) {
        (self.name, self.var_ref, self.value)
    }

    #[must_use]
    pub fn value(&self) -> SharedValue {
        self.value.clone()
    }

    #[must_use]
    pub fn ref_eq(&self, var_ref: &Ref) -> bool {
        &self.var_ref == var_ref
    }

    #[must_use]
    pub fn name_eq(&self, name: &str) -> bool {
        self.name == name
    }
}
