use std::{collections::HashMap, process::Command};

use parser::{
    ast::{BinOpExpr, Block, Expr, ExprKind, FnCall, If, Item, ItemKind, UnOpKind, While},
    parse,
};

use crate::{CommandError, CommandResult, Error, Result, RuntimeError, RuntimeResult};

mod_use::mod_use![value, utils, scope, var, refs, func, module];

const MAX_DEPTH: usize = 1 << 14;

#[must_use]
pub struct Engine {
    fns: HashMap<String, Box<dyn ExternalFn>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
        }
    }

    pub fn with_fn<Param, FnPtr, Func>(self, name: impl Into<String>, func: Func) -> Self
    where
        Func: Into<ExtractFn<Param, FnPtr>>,
        ExtractFn<Param, FnPtr>: ExternalFn,
    {
        self.with_fn_raw(name, func.into())
    }

    pub fn with_fn_raw(mut self, name: impl Into<String>, func: impl ExternalFn) -> Self {
        self.fns.insert(name.into(), Box::new(func));
        self
    }

    pub fn execute(self, src: &str) -> Result<'_, ()> {
        let tree = parse(src).map_err(Error::Parse)?;
        let mut ctx = Context::new();

        let global = ctx.global();

        for (name, func) in self.fns {
            global.register_boxed_external_fn(name, func);
        }

        // hoist
        for item in &tree.items {
            if let ItemKind::FnDef(fn_def) = &item.kind {
                ctx.current_mut().register_script_fn(fn_def.clone());
            }
        }

        for item in &tree.items {
            drop(ctx.eval_item(item)?);
        }

        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub struct Context<'src> {
    scopes: Vec<Scope<'src>>,
    depth: usize,
}

impl<'src> Context<'src> {
    pub fn new() -> Self {
        let mut scopes = Vec::with_capacity(8);
        scopes.push(Scope::new_global());
        Self { scopes, depth: 0 }
    }

    fn eval_item(&mut self, item: &Item<'src>) -> Result<'src, Value> {
        match &item.kind {
            ItemKind::FnDef(_) => Ok(Value::Unit),
            ItemKind::Stmt(stmt) => {
                let val = self.eval_expr(&stmt.expr)?;
                self.current_mut().new_var(stmt.ident.name, val);
                Ok(Value::Unit)
            }
            ItemKind::Expr(expr) => self.eval_expr(expr),
            ItemKind::Assign(assign) => {
                let new_val = self.eval_expr(&assign.expr)?;
                let var = self.search_mut(assign.ident.name)?;
                var.update(new_val);
                Ok(Value::Unit)
            }
            ItemKind::If(If {
                cond,
                then_block,
                else_block,
                ..
            }) => {
                if self.eval_expr(cond)?.rt_cast::<bool>("<if_cond>")? {
                    self.eval_block(then_block)
                } else if let Some(else_block) = else_block {
                    self.eval_block(else_block)
                } else {
                    Ok(Value::Unit)
                }
            }
            ItemKind::While(While { expr, block, .. }) => {
                while self.eval_expr(expr)?.rt_cast::<bool>("<while_cond>")? {
                    drop(self.eval_block(block)?);
                }
                Ok(Value::Unit)
            }
            ItemKind::For(_) => {
                unimplemented!("for loop")
            }
            _ => unreachable!("Break by new variant"),
        }
    }

    fn eval_expr(&mut self, expr: &Expr<'src>) -> Result<'src, Value> {
        match &expr.kind {
            ExprKind::Unit => Ok(Value::Unit),
            ExprKind::Literal(lit) => Value::from(lit).ok(),
            ExprKind::FnCall(fn_call) => self.eval_fn(fn_call),
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::BinOp(op) => self.eval_bin_op(op),
            ExprKind::Ident(ident) => self
                .search(ident.name)
                .map(Variable::value)
                .map_err(Into::into),
            ExprKind::Exec(cmd) => {
                println!("Executing {cmd:?}");
                let res = eval_exec_str(cmd.cmd)?;
                Value::Str(res.into()).ok()
            }
            ExprKind::UnOp(op) => {
                let val = self.eval_expr(&op.expr)?;
                match op.kind {
                    UnOpKind::Neg => Ok(Value::Int(-val.rt_cast::<i64>("<neg>")?)),
                    UnOpKind::Not => Ok(Value::Bool(!val.rt_cast::<bool>("<not>")?)),
                    _ => unreachable!("Break by new variant"),
                }
            }
            _ => unreachable!("Break by new variant"),
        }
    }

    fn eval_bin_op(&mut self, bin_op: &BinOpExpr<'src>) -> Result<'src, Value> {
        #[allow(clippy::enum_glob_use)]
        use parser::ast::BinOpKind::*;

        let BinOpExpr {
            left, right, kind, ..
        } = bin_op;

        let (left, right) = (self.eval_expr(left)?, self.eval_expr(right)?);
        match kind {
            numerical_op @ (Add | Sub | Mul | Div | Lt | Le | Gt | Ge) => {
                let ty_name = left
                    .ty_eq_name(&right)
                    .ok_or_else(|| RuntimeError::TypeError {
                        ident: format!("<right of ({})>", numerical_op.as_str()),
                        expected: left.type_name().to_owned(),
                        found: right.type_name().to_owned(),
                    })?;

                if ty_name == i64::TYPE_NAME {
                    let (left, right) = (left.cast::<i64>().unwrap(), right.cast::<i64>().unwrap());

                    let res = match numerical_op {
                        Add => Value::Int(left + right),
                        Sub => Value::Int(left - right),
                        Mul => Value::Int(left * right),
                        Div => Value::Int(left / right),
                        Lt => Value::Bool(left < right),
                        Le => Value::Bool(left <= right),
                        Gt => Value::Bool(left > right),
                        Ge => Value::Bool(left >= right),
                        _ => unreachable!("Break by new variant"),
                    };
                    Ok(res)
                } else if ty_name == f64::TYPE_NAME {
                    let (left, right) = (left.cast::<f64>().unwrap(), right.cast::<f64>().unwrap());

                    let res = match numerical_op {
                        Add => Value::Float(left + right),
                        Sub => Value::Float(left - right),
                        Mul => Value::Float(left * right),
                        Div => Value::Float(left / right),
                        Lt => Value::Bool(left < right),
                        Le => Value::Bool(left <= right),
                        Gt => Value::Bool(left > right),
                        Ge => Value::Bool(left >= right),
                        _ => unreachable!("Break by new variant"),
                    };
                    Ok(res)
                } else {
                    Err(RuntimeError::TypeError {
                        ident: format!("<right of ({})>", numerical_op.as_str()),
                        expected: left.type_name().to_owned(),
                        found: right.type_name().to_owned(),
                    })?
                }
            }
            op @ (Eq | Neq) => {
                if left.ty_eq(&right) {
                    match op {
                        Eq => Value::Bool(left == right),
                        Neq => Value::Bool(left != right),
                        _ => unreachable!("Break by new variant"),
                    }
                    .ok()
                } else {
                    RuntimeError::TypeError {
                        ident: format!("<right of ({})>", op.as_str()),
                        expected: left.type_name().to_owned(),
                        found: right.type_name().to_owned(),
                    }
                    .err()?
                }
            }
            And => {
                let left = left.rt_cast::<bool>("<left of (&&)>")?;
                let right = right.rt_cast::<bool>("<right of (&&)>")?;
                Ok(Value::Bool(left && right))
            }
            Or => Ok(Value::Bool(
                left.rt_cast::<bool>("<left of (||)>")?
                    || right.rt_cast::<bool>("<right of (||)>")?,
            )),
            _ => unreachable!("Break by new variant"),
        }
    }

    fn eval_fn(&mut self, fn_call: &FnCall<'src>) -> Result<'src, Value> {
        let name = fn_call.ident.name;
        let found = self.search_mut(name)?.value_ref();
        let fn_ref = *found
            .cast_ref::<FnRef>()
            .map_err(|e| RuntimeError::TypeError {
                ident: name.to_string(),
                expected: FnRef::TYPE_NAME.to_owned(),
                found: e.type_name().to_owned(),
            })?;
        self.get_fn(fn_ref)?.call(self, fn_call)
    }

    fn eval_block(&mut self, block: &Block<'src>) -> Result<'src, Value> {
        self.enter_scope("block")?;
        for item in &block.items {
            drop(self.eval_item(item)?);
        }
        self.pop_scope();
        Ok(Value::Unit)
    }

    #[inline]
    fn global(&mut self) -> &mut Scope<'src> {
        &mut self.scopes[0]
    }

    #[inline]
    fn current(&self) -> &Scope<'src> {
        &self.scopes[self.depth]
    }

    #[inline]
    fn current_mut(&mut self) -> &mut Scope<'src> {
        &mut self.scopes[self.depth]
    }

    #[inline]
    fn enter_scope(&mut self, name: impl Into<String>) -> RuntimeResult<()> {
        if self.depth == MAX_DEPTH {
            return Err(RuntimeError::MaxRecursionExceeded);
        }

        let new = self.depth + 1;

        if self.scopes.len() <= new {
            let scope = Scope::new(name);
            self.scopes.push(scope);
        } else {
            self.scopes[new].clear(name);
        }

        self.depth += 1;

        Ok(())
    }

    #[inline]
    fn pop_scope(&mut self) {
        self.depth -= 1;
    }

    fn _get(&self, ref_: Ref) -> RuntimeResult<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Ok(val) = scope.search(&ref_) {
                return Ok(val);
            }
        }
        Err(RuntimeError::NullRefError(ref_))
    }

    fn get_fn(&self, fn_ref: FnRef) -> RuntimeResult<Shared<Callable<'src>>> {
        for scope in self.scopes.iter().rev() {
            if let Ok(val) = scope.get_fn(fn_ref) {
                return Ok(val);
            }
        }
        Err(RuntimeError::NullRefError(fn_ref.inner()))
    }

    fn search(&self, name: &str) -> RuntimeResult<&Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|x| x.get(name).ok())
            .ok_or_else(|| RuntimeError::IdentNotFound(name.to_owned()))
    }

    fn search_mut(&mut self, name: &str) -> RuntimeResult<&mut Variable> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|x| x.get_mut(name).ok())
            .ok_or_else(|| RuntimeError::IdentNotFound(name.to_owned()))
    }
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn eval_exec(command: &str) -> CommandResult {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(CommandError::Command)
}

pub fn eval_exec_str(command: &str) -> CommandResult<String> {
    let res = eval_exec(command)?;
    Ok(String::from_utf8_lossy(&res.stdout).to_string())
}
