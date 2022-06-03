use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    ops::Deref,
    sync::atomic::AtomicUsize,
};

use parser::ast::FnDef;

use crate::{
    Callable, ExternalFn, FnRef, IntoShared, Ref, RuntimeError, RuntimeResult, Shared, SharedValue,
    Value, Variable,
};

pub struct Scope<'a> {
    name: String,
    fns: HashMap<FnRef, Shared<Callable<'a>>>,
    vars: Vec<Variable>,
}

impl<'a> Scope<'a> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fns: HashMap::new(),
            vars: Vec::with_capacity(8),
        }
    }

    #[must_use]
    pub fn is_global(&self) -> bool {
        self.name == "global"
    }

    pub fn register_script_fn(&mut self, def: FnDef<'a>) {
        let hash = {
            let mut hasher = DefaultHasher::new();
            def.hash(&mut hasher);
            hasher.finish()
        };
        let fn_ref = FnRef::new(Self::new_ref(), def.ident.name);
        self.new_var(def.ident.name, Value::Fn(fn_ref.clone()));
        self.fns
            .insert(fn_ref, Callable::script(def, hash).shared());
    }

    pub fn register_native_fn(&mut self, name: impl Into<String>, func: impl ExternalFn) {
        let name = name.into();
        let fn_ref = FnRef::new(Self::new_ref(), &name);
        let fn_ptr = Callable::native(func, &name).shared();
        self.new_var(name, Value::Fn(fn_ref.clone()));
        self.fns.insert(fn_ref, fn_ptr);
    }

    pub fn new_var(&mut self, name: impl Into<String>, val: impl Into<SharedValue>) -> Ref {
        let name = name.into();
        let ret = Self::new_ref();
        self.vars.push(Variable::new(name, ret, val));
        ret
    }

    pub fn get_fn(&self, fn_ref: &FnRef) -> RuntimeResult<Shared<Callable<'a>>> {
        self.fns
            .get(fn_ref)
            .cloned()
            .ok_or_else(|| RuntimeError::NullRefError(*fn_ref.deref()))
    }

    pub fn get(&self, val_ref: &Ref) -> RuntimeResult<&Variable> {
        self.vars
            .iter()
            .find(|var| var.ref_eq(val_ref))
            .ok_or(RuntimeError::NullRefError(*val_ref))
    }

    pub fn search(&self, name: &str) -> RuntimeResult<&Variable> {
        self.vars
            .iter()
            .find(|var| var.name_eq(name))
            .ok_or_else(|| RuntimeError::IdentNotFound(name.to_string()))
    }

    fn new_ref() -> Ref {
        static REF: AtomicUsize = AtomicUsize::new(0);
        REF.fetch_add(1, std::sync::atomic::Ordering::SeqCst).into()
    }
}
