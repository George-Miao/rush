use std::collections::HashMap;

use crate::{Callable, FnRef, Shared, Variable};

pub struct Module<'a> {
    pub name: String,
    pub fns: HashMap<FnRef, Shared<Callable<'a>>>,
    pub vars: Vec<Variable>,
}
