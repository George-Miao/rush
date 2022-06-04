use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Display,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::atomic::AtomicUsize,
};

use parser::ast::FnDef;

use crate::{
    Callable, ExternalFn, FnRef, IntoShared, Ref, RuntimeError, RuntimeResult, Shared, Value,
    Variable,
};

#[must_use]
pub struct Scope<'a> {
    name: String,
    depth: usize,
    fns: HashMap<FnRef, Shared<Callable<'a>>>,
    vars: Vec<Variable>,
}

impl<'a> Scope<'a> {
    pub fn new(name: impl Into<String>, depth: usize) -> Self {
        Self {
            depth,
            name: name.into(),
            fns: HashMap::new(),
            vars: vec![],
        }
    }

    pub fn new_global() -> Self {
        Self::new("global", 0)
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.depth == 0
    }

    pub fn register_script_fn(&mut self, def: FnDef<'a>) {
        let hash = {
            let mut hasher = DefaultHasher::new();
            def.hash(&mut hasher);
            hasher.finish()
        };
        let fn_ref = Self::new_ref().into();
        self.new_var(def.ident.name, Value::new(fn_ref));
        self.fns
            .insert(fn_ref, Callable::script(def, hash).shared());
    }

    pub fn register_native_fn(&mut self, name: impl Into<String>, func: impl ExternalFn) {
        let name = name.into();
        let fn_ref = FnRef::new(Self::new_ref());
        let fn_ptr = Callable::native(func, &name).shared();
        self.new_var(name, Value::new(fn_ref));
        self.fns.insert(fn_ref, fn_ptr);
    }

    pub fn new_var(&mut self, name: impl Into<String>, val: impl Into<Value>) -> Ref {
        let name = name.into();
        let ret = Self::new_ref();
        self.vars.push(Variable::new(name, ret, val));
        ret
    }

    pub fn get_fn(&self, fn_ref: FnRef) -> RuntimeResult<Shared<Callable<'a>>> {
        self.fns
            .get(&fn_ref)
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

    pub fn search_mut(&mut self, name: &str) -> RuntimeResult<&mut Variable> {
        self.vars
            .iter_mut()
            .find(|var| var.name_eq(name))
            .ok_or_else(|| RuntimeError::IdentNotFound(name.to_string()))
    }

    fn new_ref() -> Ref {
        static REF: AtomicUsize = AtomicUsize::new(0);
        REF.fetch_add(1, std::sync::atomic::Ordering::SeqCst).into()
    }
}

impl Display for Scope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scope({}, L{})", self.name, self.depth)
    }
}
