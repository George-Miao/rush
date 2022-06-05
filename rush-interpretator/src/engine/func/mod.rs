#![allow(clippy::module_name_repetitions)]

mod_use::mod_use![fn_ref, external, script];

use parser::ast::{Expr, FnCall, FnDef};

use crate::{Context, Result, RuntimeError, ToResult, Value};

pub type FnCallArg = Vec<Value>;
pub type FnCallParam<'r, 'a> = &'r [Expr<'a>];

#[must_use]
pub enum Callable<'a> {
    Native(NativeFn),
    Script(ScriptFn<'a>),
}

impl<'a> Callable<'a> {
    pub fn native(ptr: impl ExternalFn, name: impl Into<String>) -> Self {
        Self::Native(NativeFn::new(ptr, name.into()))
    }

    pub fn native_boxed(ptr: Box<dyn ExternalFn>, name: impl Into<String>) -> Self {
        Self::Native(NativeFn::new_boxed(ptr, name.into()))
    }

    pub fn call(&self, ctx: &mut Context<'a>, fn_call: &FnCall<'a>) -> Result<'a, Value> {
        let name = fn_call.ident.name;

        match self {
            Callable::Native(native_fn) => {
                let args = fn_call
                    .args
                    .iter()
                    .map(|arg| ctx.eval_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                native_fn.call(args).map(Into::into).map_err(Into::into)
            }
            Callable::Script(script_fn) => {
                let def = &script_fn.def;
                let args = &fn_call.args;
                if def.params.len() != args.len() {
                    RuntimeError::ArgumentError {
                        ident: def.ident.name.to_owned(),
                        expected: def.params.len(),
                        found: args.len(),
                    }
                    .err()?;
                }
                ctx.enter_scope(name)?;
                for (param, arg) in std::iter::zip(&def.params, args) {
                    let arg_val = ctx.eval_expr(arg)?;
                    ctx.current_mut().new_var(param.name, arg_val);
                }
                for item in &script_fn.def.body.items {
                    drop(ctx.eval_item(item)?);
                }
                ctx.pop_scope();
                Ok(Value::Unit)
            }
        }
    }

    pub const fn script(def: FnDef<'a>, hash: u64) -> Self {
        Self::Script(ScriptFn::new(def, hash))
    }
}

impl From<NativeFn> for Callable<'_> {
    fn from(native_fn: NativeFn) -> Self {
        Self::Native(native_fn)
    }
}

impl<'a> From<ScriptFn<'a>> for Callable<'a> {
    fn from(script_fn: ScriptFn<'a>) -> Self {
        Self::Script(script_fn)
    }
}
