use std::fmt::Display;

use parser::ast::FnDef;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ScriptFn<'a> {
    pub def: FnDef<'a>,
    pub hash: u64,
}

impl<'a> ScriptFn<'a> {
    #[must_use]
    pub const fn new(def: FnDef<'a>, hash: u64) -> Self {
        Self { def, hash }
    }
}

impl Display for ScriptFn<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptFn({})", self.def)
    }
}
