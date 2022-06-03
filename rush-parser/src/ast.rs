use std::{fmt, hash::Hash};

use pest::Span;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Tree<'a> {
    pub span: Span<'a>,
    pub items: Vec<Item<'a>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Item<'a> {
    pub span: Span<'a>,
    pub kind: ItemKind<'a>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ItemKind<'a> {
    Expr(Expr<'a>),
    Stmt(Stmt<'a>),
    FnDef(FnDef<'a>),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Stmt<'a> {
    pub span: Span<'a>,
    pub ident: Ident<'a>,
    pub expr: Expr<'a>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FnDef<'a> {
    pub span: Span<'a>,
    pub ident: Ident<'a>,
    pub params: Vec<Ident<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ExprKind<'a> {
    Ident(Ident<'a>),
    Literal(Literal<'a>),
    FnCall(FnCall<'a>),
    Exec(Exec<'a>),
    Block(Block<'a>),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Ident<'a> {
    pub name: &'a str,
    pub span: Span<'a>,
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "Ident({})", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FnCall<'a> {
    pub ident: Ident<'a>,
    pub args: Vec<Expr<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Exec<'a> {
    pub span: Span<'a>,
    pub cmd: &'a str,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Block<'a> {
    pub span: Span<'a>,
    pub items: Vec<Item<'a>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Literal<'a> {
    pub kind: LiteralKind<'a>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind<'a> {
    String(&'a str),
    Bool(bool),
    Number(i128),
    Float(f64),
}

impl Hash for LiteralKind<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            LiteralKind::String(s) => s.hash(state),
            LiteralKind::Bool(b) => b.hash(state),
            LiteralKind::Number(n) => n.hash(state),
            LiteralKind::Float(f) => (*f as i64).hash(state),
        }
    }
}
