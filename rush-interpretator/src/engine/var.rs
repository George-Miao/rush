use crate::{Ref, Value};

#[derive(Debug)]
pub struct Variable {
    name: String,
    var_ref: Ref,
    value: Value,
}

impl Variable {
    pub fn new(name: impl Into<String>, var_ref: Ref, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            var_ref,
            value: value.into(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> (String, Ref, Value) {
        (self.name, self.var_ref, self.value)
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub const fn value_ref(&self) -> &Value {
        &self.value
    }

    #[must_use]
    pub fn type_name(&self) -> &str {
        self.value.type_name()
    }

    pub fn update(&mut self, val: Value) {
        self.value = val;
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
