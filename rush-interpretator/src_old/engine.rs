use std::{
    io::{self, Write},
    ops::Deref,
    rc::Rc,
};

use parser::{
    ast::{Expr, FnDef, Ident, Item, Stmt},
    parse,
};

use crate::{eval_exec, CommandError, FnPtr, Result, RuntimeError, Shared, Value};

#[derive(Debug)]
pub struct Engine<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Engine {
            scopes: vec![Scope::new("global")],
        }
    }

    pub fn current_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("Should exist at least one scope")
    }

    pub fn current(&self) -> &Scope {
        self.scopes.last().expect("Should exist at least one scope")
    }

    pub fn insert(&mut self, ident: Ident, value: impl Into<Shared<Value<'a>>>) {
        self.current_mut().push(ident, value.into());
    }

    pub fn search(&self, ident: &Ident) -> Option<Shared<Value>> {
        self.scopes.iter().rev().find_map(|scope| scope.get(ident))
    }

    pub fn get(&self, ident: &Ident) -> Option<Shared<Value>> {
        self.current().get(ident)
    }

    pub fn enter(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub fn depth(&self) -> Depth {
        match self.scopes.len() {
            0 => Depth::Global,
            n => Depth::Local(n - 1),
        }
    }

    pub fn run_code(&mut self, snippet: &str) -> Result<()> {
        let tree = parse(snippet)?;
        self.run(tree)
    }

    pub fn run(&mut self, tree: impl IntoIterator<Item = Item<'a>>) -> Result<()> {
        for item in tree {
            self.handle_item(item).unwrap()
        }
        Ok(())
    }

    pub fn handle_item(&mut self, item: Item) -> Result<()> {
        match item {
            Item::Stmt(stmt) => {
                self.eval_stmt(stmt)?;
            }
            Item::Expr(expr) => {
                self.eval_expr(expr)?;
            }
            Item::Comment(_) => {}
            Item::FnDef(fn_decl) => self.insert(fn_decl.ident.clone(), Value::Fn(fn_decl)),
            Item::Noop => {}
        }
        Ok(())
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Result<()> {
        let Stmt { ident, expr, .. } = stmt;

        let value = self.eval_expr(expr)?;
        self.insert(ident, value);

        Ok(())
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Shared<Value>> {
        let val = match expr {
            Expr::Exec { command } => {
                let out = eval_exec(&command)?;
                Value::new_shared(String::from_utf8(out.stdout).map_err(CommandError::CodingError)?)
            }
            Expr::Call { ident, args } => {
                if ident.deref() == "println" {
                    let mut stdout = io::stdout().lock();
                    for arg in args {
                        let arg_val = self.eval_expr(arg)?;
                        let _ = write!(stdout, "{:?}", arg_val.deref());
                    }
                    writeln!(stdout).unwrap();
                    return Ok(Value::new_shared(()));
                } else {
                    todo!()
                }
                let found = self.search(&ident);
                let decl = found
                    .as_deref()
                    .ok_or_else(|| RuntimeError::IdentNotFound(ident.to_owned()))?;

                match decl.cast_ref::<FnDef>() {
                    Some(x) => {}
                    None => match decl.cast_ref::<Rc<FnPtr>>() {
                        Some(x) => {}
                        None => Err(RuntimeError::TypeError {
                            ident: ident.clone(),
                            expected: "fn".to_owned(),
                            found: decl.type_name().to_owned(),
                        })?,
                    },
                }
                // if decl.params.len() != args.len() {
                //     return Err(RuntimeError::FnCallArgCount {
                //         ident: ident.clone(),
                //         expected: decl.params.len().try_into().expect("Too many param"),
                //         found: args.len().try_into().expect("Too many args"),
                //     }
                //     .into());
                // }
                // let mut scope = Scope::new(ident);

                // for (i, arg) in args.into_iter().enumerate() {
                //     let value = self.eval_expr(arg)?;
                //     scope.push(decl.params[i].to_owned(), value);
                // }

                todo!()
            }
            Expr::Ident(ident) => self
                .search(&ident)
                .ok_or_else(|| RuntimeError::IdentNotFound(ident.clone()))?,
            Expr::Unit => Value::new_shared(()),
            Expr::Assign { ident: _, expr: _ } => {
                todo!()
            }
            Expr::Return { expr: _ } => todo!(),
            Expr::Literal(lit) => Value::from(lit).share(),
            _ => todo!(),
        };

        Ok(val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Depth {
    Global,
    Local(usize),
}

impl Default for Engine<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    pub ident: Ident<'a>,
    names: Vec<Ident<'a>>,
    values: Vec<Shared<Value<'a>>>,
    last_idx: usize,
}

impl<'a> Scope<'a> {
    pub fn new(name: impl Into<Ident<'a>>) -> Self {
        Scope {
            ident: name.into(),
            names: vec![],
            values: vec![],
            last_idx: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn push(&mut self, ident: Ident<'a>, value: impl Into<Shared<Value<'a>>>) {
        self.names.push(ident);
        self.values.push(value.into());
    }

    pub fn get(&self, ident: &Ident<'a>) -> Option<Shared<Value<'a>>> {
        let len = self.len();

        self.names
            .iter()
            .rev()
            .enumerate()
            .find(|(.., key)| &ident == key)
            .map(|(index, ..)| self.values[len - 1 - index].to_owned())
    }
}
