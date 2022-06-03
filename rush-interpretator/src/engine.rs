use std::{iter::zip, process::Command};

use parser::ast::{Block, Expr, ExprKind, FnCall, Item, ItemKind};

use crate::{
    Callable, CommandError, CommandResult, Error, ExternalFn, ExtractFn, FnRef, Ref, Result,
    RuntimeError, RuntimeResult, Scope, Shared, SharedValue, Value, Variable, Variant,
};

#[must_use]
pub struct Engine<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new("global".to_string())],
        }
    }

    pub fn execute(&mut self, src: &'a str) -> Result<'a, ()> {
        let tree = parser::parse(src).map_err(Error::Parse)?;

        for item in &tree.items {
            self.handle_item(item)?;
        }
        Ok(())
    }

    pub fn with_fn<Param, FnPtr, Func>(self, name: impl Into<String>, func: Func) -> Self
    where
        Func: Into<ExtractFn<Param, FnPtr>>,
        ExtractFn<Param, FnPtr>: ExternalFn,
    {
        self.with_fn_raw(name, func.into())
    }

    pub fn with_fn_raw(mut self, name: impl Into<String>, func: impl ExternalFn) -> Self {
        self.global().register_native_fn(name.into(), func);
        self
    }

    fn handle_item(&mut self, item: &Item<'a>) -> Result<'a, SharedValue> {
        match &item.kind {
            ItemKind::FnDef(def) => {
                let def_clone = def.clone();
                self.current_mut().register_script_fn(def_clone);
                Ok(Value::Unit.into())
            }
            ItemKind::Stmt(stmt) => {
                let val = self.eval_expr(&stmt.expr)?;
                self.current_mut().new_var(stmt.ident.name, val);
                Ok(Value::Unit.into())
            }
            ItemKind::Expr(expr) => self.eval_expr(expr),
        }
    }

    fn eval_expr(&mut self, expr: &Expr<'a>) -> Result<'a, SharedValue> {
        match &expr.kind {
            ExprKind::Ident(ident) => self
                .search(ident.name)
                .map(Variable::value)
                .map_err(Into::into),
            ExprKind::Literal(lit) => Ok(Value::from(lit).into()),
            ExprKind::FnCall(fn_call) => self.run_fn(fn_call),
            ExprKind::Exec(cmd) => {
                let res = eval_exec_str(cmd.cmd)?;
                Ok(Value::Str(res).into())
            }
            ExprKind::Block(block) => self.run_block(block),
            ExprKind::Unit => Ok(Value::Unit.into()),
        }
    }

    fn run_fn(&mut self, fn_call: &FnCall<'a>) -> Result<'a, SharedValue> {
        let name = fn_call.ident.name;
        let found = self.search(name)?.value();
        let guard = found.get();
        let fn_ref = guard
            .cast_ref::<FnRef>()
            .map_err(|e| RuntimeError::TypeError {
                ident: name.to_string(),
                expected: FnRef::TYPE_NAME.to_owned(),
                found: e.type_name().to_owned(),
            })?;
        match &*self.get_fn(fn_ref)? {
            Callable::Native(native_fn) => {
                let args = fn_call
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(arg))
                    .collect::<Result<Vec<_>>>()?;
                native_fn.call(args).map(Into::into).map_err(Into::into)
            }
            Callable::Script(script_fn) => {
                let def = &script_fn.def;
                let args = &fn_call.args;
                if def.params.len() != args.len() {
                    Err(RuntimeError::ArgumentError {
                        ident: def.ident.name.to_owned(),
                        expected: def.params.len(),
                        found: args.len(),
                    })?;
                }
                self.enter_scope(fn_ref.name());
                for (param, arg) in zip(&def.params, args) {
                    let arg_val = self.eval_expr(arg)?;
                    self.current_mut().new_var(param.name, arg_val);
                }
                for item in &script_fn.def.body.items {
                    self.handle_item(item)?;
                }
                self.pop_scope();
                Ok(Value::Unit.into())
            }
        }
    }

    fn run_block(&mut self, block: &Block<'a>) -> Result<'a, SharedValue> {
        self.enter_scope("block");
        for item in &block.items {
            self.handle_item(item)?;
        }
        self.pop_scope();
        Ok(Value::Unit.into())
    }

    fn global(&mut self) -> &mut Scope<'a> {
        self.scopes.first_mut().unwrap()
    }

    #[inline]
    fn current_mut(&mut self) -> &mut Scope<'a> {
        self.scopes.last_mut().unwrap()
    }

    #[inline]
    fn enter_scope(&mut self, name: impl Into<String>) {
        let scope = Scope::new(name);
        self.scopes.push(scope);
    }

    #[inline]
    fn pop_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }

    fn get(&self, ref_: Ref) -> RuntimeResult<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Ok(val) = scope.get(&ref_) {
                return Ok(val);
            }
        }
        Err(RuntimeError::NullRefError(ref_))
    }

    fn get_fn(&self, fn_ref: &FnRef) -> RuntimeResult<Shared<Callable<'a>>> {
        for scope in self.scopes.iter().rev() {
            if let Ok(val) = scope.get_fn(fn_ref) {
                return Ok(val);
            }
        }
        Err(RuntimeError::IdentNotFound(fn_ref.name().to_string()))
    }

    fn search(&self, name: &str) -> RuntimeResult<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Ok(val) = scope.search(name) {
                return Ok(val);
            }
        }
        Err(RuntimeError::IdentNotFound(name.to_string()))
    }
}

impl<'a> Default for Engine<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn eval_exec(command: &str) -> CommandResult {
    Command::new(command)
        .output()
        .map_err(CommandError::Command)
}

pub fn eval_exec_str(command: &str) -> CommandResult<String> {
    let res = eval_exec(command)?;
    Ok(String::from_utf8_lossy(&res.stdout).to_string())
}
