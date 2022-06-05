use crate::{Ref, Value};

#[derive(Debug)]
pub struct Variable {
    var_ref: Ref,
    value: Value,
}

impl Variable {
    pub fn new(var_ref: Ref, value: impl Into<Value>) -> Self {
        Self {
            var_ref,
            value: value.into(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> (Ref, Value) {
        (self.var_ref, self.value)
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub const fn value_ref(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
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
}
