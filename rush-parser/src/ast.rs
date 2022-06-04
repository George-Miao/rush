use std::{hash::Hash, marker::PhantomData};

use pest::Span;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Tree<'src> {
    pub span: Span<'src>,
    pub items: Vec<Item<'src>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Item<'src> {
    pub span: Span<'src>,
    pub kind: ItemKind<'src>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ItemKind<'src> {
    FnDef(FnDef<'src>),
    Stmt(Stmt<'src>),
    Assign(Assign<'src>),
    If(If<'src>),
    For(For<'src>),
    While(While<'src>),
    Expr(Expr<'src>),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Stmt<'src> {
    pub span: Span<'src>,
    pub ident: Ident<'src>,
    pub expr: Expr<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FnDef<'src> {
    pub span: Span<'src>,
    pub ident: Ident<'src>,
    pub params: Vec<Ident<'src>>,
    pub body: Block<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct If<'src> {
    pub span: Span<'src>,
    pub cond: Expr<'src>,
    pub then_block: Block<'src>,
    pub else_block: Option<Block<'src>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct For<'src> {
    pub span: Span<'src>,
    pub ident: Ident<'src>,
    pub expr: Expr<'src>,
    pub block: Block<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct While<'src> {
    pub span: Span<'src>,
    pub expr: Expr<'src>,
    pub block: Block<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: Span<'src>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ExprKind<'src> {
    Ident(Ident<'src>),
    Literal(Literal<'src>),
    FnCall(FnCall<'src>),
    Exec(Exec<'src>),
    Block(Block<'src>),
    BinOp(BinOpExpr<'src>),
    UnOp(UnOpExpr<'src>),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Ident<'src> {
    pub name: &'src str,
    pub span: Span<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FnCall<'src> {
    pub ident: Ident<'src>,
    pub args: Vec<Expr<'src>>,
    pub span: Span<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Exec<'src> {
    pub span: Span<'src>,
    pub cmd: &'src str,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Block<'src> {
    pub span: Span<'src>,
    pub items: Vec<Item<'src>>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Literal<'src> {
    pub kind: LiteralKind<'src>,
    pub span: Span<'src>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind<'src> {
    String(&'src str),
    Bool(bool),
    Number(i64),
    Float(f64),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum BinOpKind<'src> {
    __Marker(PhantomData<&'src ()>),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum UnOpKind {
    Neg,
    Not,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BinOpExpr<'src> {
    pub left: Box<Expr<'src>>,
    pub right: Box<Expr<'src>>,
    pub kind: BinOpKind<'src>,
    pub span: Span<'src>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct UnOpExpr<'src> {
    pub expr: Box<Expr<'src>>,
    pub kind: UnOpKind,
    pub span: Span<'src>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign<'src> {
    pub span: Span<'src>,
    pub ident: Ident<'src>,
    pub expr: Expr<'src>,
}
