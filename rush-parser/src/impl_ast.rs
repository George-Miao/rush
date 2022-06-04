use std::{fmt, hash::Hash};

use itertools::Itertools;
use pest::{iterators::Pair, Span};

use crate::{
    ast::*,
    error::{Error, Result},
    Rule,
};

pub trait Node<'src> {
    const RULE: Rule;
    fn span(&self) -> &Span<'src>;
    fn src(&self) -> &'src str;
}

macro_rules! impl_node {
    ($ty:ident, $def_rule:ident $(, $extra_rule:ident )* $(,)? => $val:ident => $constructor:expr) => {
        impl<'src> TryFrom<Pair<'src, Rule>> for $ty<'src> {
            type Error = Error<'src>;

            #[allow(unused_mut)]
            fn try_from(mut $val: Pair<'src, Rule>) -> Result<Self> {
                ensure!($val, $def_rule, $( $extra_rule, )*);

                Ok($constructor)
            }
        }

        impl<'src> Node<'src> for $ty<'src> {
            const RULE: Rule = Rule::$def_rule;

            fn span(&self) -> &Span<'src> {
                &self.span
            }

            fn src(&self) -> &'src str {
                self.span().as_str()
            }
        }
    };
}

macro_rules! ensure {
    ($val:ident, $first:ident $(, $expect:ident )* $(,)?) => {
        let rule = $val.as_rule();
        if rule != Rule::$first $( && rule != Rule::$expect )* {
            return Err(Error::TreeError {
                expect: &[Rule::$first, $( Rule::$expect, )*],
                found: $val.as_rule(),
                span: $val.as_span(),
            });
        }
    };
}

impl_node! {
    Tree, main => value => {
        let span = value.as_span();
        let iter = value.into_inner();
        let items = iter.map_while(|x| {
            match x.as_rule() {
                Rule::item => Some(Item::try_from(x)),
                Rule::EOI => None,
                _ => unreachable!("Tree should only contain item and EOI"),
            }
        }).try_collect()?;
        Tree {
            span,
            items,
        }
    }
}

impl_node! {
    Item, item => value => {
        let span = value.as_span();
        let inner = value.into_inner().next().expect("Item should have content");
        let kind = match inner.as_rule() {
            Rule::fn_def => ItemKind::FnDef(FnDef::try_from(inner)?),
            Rule::stmt => ItemKind::Stmt(Stmt::try_from(inner)?),
            Rule::assign => ItemKind::Assign(Assign::try_from(inner)?),
            Rule::if_loop => ItemKind::If(If::try_from(inner)?),
            Rule::for_loop => ItemKind::For(For::try_from(inner)?),
            Rule::while_loop => ItemKind::While(While::try_from(inner)?),
            Rule::expr => ItemKind::Expr(Expr::try_from(inner)?),
            _ => unreachable!("Item should have expr, stmt or fn_def"),
        };
        Item {
            kind,
            span,
        }
    }
}

impl_node! {
    Assign, assign => value => {
        let span = value.as_span();
        let (ident, expr) = value.into_inner().next_tuple().expect("Assign should have ident and expr");
        let ident = Ident::try_from(ident)?;
        let expr = Expr::try_from(expr)?;
        Assign {
            ident,
            expr,
            span,
        }
    }
}

impl_node! {
    If, if_loop => value => {
        let span = value.as_span();
        let mut inner = value.into_inner();
        let (cond, body) = inner.next_tuple().expect("If should have cond and body");
        let else_block = match  inner.next().map(Block::try_from) {
            Some(Ok(block)) => Some(block),
            Some(Err(err)) => return Err(err),
            None => None,
        };
        let cond = Expr::try_from(cond)?;
        let then_block = Block::try_from(body)?;
        If {
            cond,
            then_block,
            else_block,
            span,
        }
    }
}

impl_node! {
    For, for_loop => value => {
        let span = value.as_span();
        let (ident, expr, block) = value
            .into_inner()
            .next_tuple()
            .map(|(a, b, c)| Result::Ok((
                    TryFrom::try_from(a)?,TryFrom::try_from(b)?,TryFrom::try_from(c)?,
                ))
            ).expect("For should have ident, expr and body")?;

        For {
            span,
            ident,
            expr,
            block,
        }
    }
}

impl_node! {
    While, while_loop => value => {
        let span = value.as_span();
        let ( expr, block) = value
            .into_inner()
            .next_tuple()
            .map(|(a, b)| Result::Ok((
                    TryFrom::try_from(a)?,TryFrom::try_from(b)?,
                ))
            ).expect("For should have ident, expr and body")?;

        While {
            span,
            expr,
            block,
        }
    }
}

impl_node! {
    Stmt, stmt => value => {
        let span = value.as_span();
        let mut inner = value.into_inner();
        let (ident, expr) = inner.next_tuple().expect("Stmt should have ident and expr");
        let ident = Ident::try_from(ident)?;
        let expr = Expr::try_from(expr)?;
        Stmt {
            ident,
            expr,
            span,
        }
    }
}

impl_node! {
    FnDef, fn_def => value => {
        let span = value.as_span();
        let mut inner = value.into_inner();
        let ident = TryFrom::try_from(inner.next().expect("FnDef should have ident"))?;

        let (params, body) = match inner.next().expect("FnDef should have ident or block")  {
            params if params.as_rule() == Rule::ident_list => {
                let params = params.into_inner().map(Ident::try_from).collect::<Result<Vec<_>>>()?;
                let body = Block::try_from(inner.next().expect("FnDef should have block"))?;
                (params, body)
            },
            block => (vec![], Block::try_from(block)?)
        };

        FnDef { span, ident, params, body }
    }
}

impl_node! {
    Ident, ident => value => Ident {
        name: value.as_str(),
        span: value.as_span(),
    }
}

impl_node! {
    Literal, literal => value => {
        let span = value.as_span();
        let span1 = value.as_span();
        let inner = value.into_inner().next().expect("Literal should have content");
        let kind = match inner.as_rule() {
            Rule::number => LiteralKind::Number(inner.as_str().parse().map_err(|_| {
                Error::LiteralError {
                    expect: Rule::number,
                    val: inner.as_str(),
                    span,
                }
            })?),
            Rule::float => {
                LiteralKind::Float(inner.as_str().parse().map_err(|_| {
                Error::LiteralError {
                    expect: Rule::float,
                    val: inner.as_str(),
                    span,
                }
            })?)},
            Rule::bool => LiteralKind::Bool(match inner.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!("Bool should have true or false"),
            }),
            Rule::string => LiteralKind::String(&inner.as_str()[1..inner.as_str().len() - 1]),
            _ => unreachable!("Literal should only be number, float or string"),
        };
        Literal {
        kind,
        span: span1,
    }}
}

impl_node! {
    Exec, exec => value => Exec {
        cmd: value.as_str().trim_start_matches("$`").trim_end_matches('`'),
        span: value.as_span(),
    }
}

impl_node! {
    FnCall, fn_call => value => {
        let span = value.as_span();
        let mut inner = value.into_inner();
        let ident = TryFrom::try_from(inner.next().expect("FnCall should have ident"))?;
        let args = match inner.next()  {
            Some(args) => {
                ensure!(args, expr_list);
                args.into_inner().map(Expr::try_from).collect::<Result<Vec<_>>>()?
            },
            None => vec![]
        };

        FnCall {
            ident,
            args,
            span,
        }
    }
}

impl_node! {
    Expr, expr, bin_op_expr, range, trivial_expr => value => {
        let span = value.as_span();
        let kind = match value.as_rule() {
            Rule::bin_op_expr => ExprKind::BinOp(BinOpExpr::try_from(value)?),
            Rule::range => todo!(),
            Rule::expr => {
                value = value.into_inner().next().expect("Expr should have content");
                return Expr::try_from(value)
            }
            Rule::trivial_expr  => {
                value = value.into_inner().next().expect("Expr should have content");
                ensure!(value, literal, un_op_expr, fn_call, exec, block, unit, ident);
                match value.as_rule() {
                    Rule::literal => ExprKind::Literal(Literal::try_from(value)?),
                    Rule::un_op_expr => ExprKind::UnOp(UnOpExpr::try_from(value)?),
                    Rule::fn_call => ExprKind::FnCall(FnCall::try_from(value)?),
                    Rule::exec => ExprKind::Exec(Exec::try_from(value)?),
                    Rule::block => ExprKind::Block(Block::try_from(value)?),
                    Rule::unit => ExprKind::Unit,
                    Rule::ident => ExprKind::Ident(Ident::try_from(value)?),
                    _ => unreachable!("Expr should only be bin_op_expr, range, trivial_expr"),
                }
            },
            rule => unreachable!("Expr should only be expr, trivial_expr, bin_op_expr or range, found {rule:?}"),
        };

        Expr {
            kind,
            span,
        }
    }
}

impl_node! {
    BinOpExpr, bin_op_expr => value => {
        let span = value.as_span();
        let (left, op, right) = value.into_inner().next_tuple().expect("BinOpExpr should have (expr, op, expr)");
        ensure!(op, bin_op);
        let kind = {
            match op.into_inner().next().expect("Bin op should have one child").as_rule() {
                Rule::add => BinOpKind::Add,
                Rule::sub => BinOpKind::Sub,
                Rule::mul => BinOpKind::Mul,
                Rule::div => BinOpKind::Div,
                Rule::eq => BinOpKind::Eq,
                Rule::neq => BinOpKind::Neq,
                Rule::lt => BinOpKind::Lt,
                Rule::le => BinOpKind::Le,
                Rule::gt => BinOpKind::Gt,
                Rule::ge => BinOpKind::Ge,
                Rule::and => BinOpKind::And,
                Rule::or => BinOpKind::Or,
                _ => unreachable!(),
            }
        };
        let left = Box::new(Expr::try_from(left)?);
        let right = Box::new(Expr::try_from(right)?);
        BinOpExpr {
            left,
            kind,
            right,
            span,
        }
    }
}

impl_node! {
    UnOpExpr, un_op_expr => value => {
        let span = value.as_span();
        let (op, expr) = value.into_inner().next_tuple().expect("UnOpExpr should have op and expr");
        let expr = Box::new(Expr::try_from(expr)?);
        ensure!(op, un_op);
        let kind = {
            match op.into_inner().next().expect("Bin op should have one child").as_rule() {
                Rule::not => UnOpKind::Not,
                Rule::neg => UnOpKind::Neg,
                _ => unreachable!(),
            }
        };
        UnOpExpr { expr, kind, span }
    }
}
impl_node! {
    Block, block => value => {
        let span = value.as_span();
        let items = value.into_inner().map(Item::try_from).try_collect()?;
        Block {
            span,
            items,
        }
    }
}
impl fmt::Display for FnDef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.ident)?;
        for (i, ident) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", ident)?;
        }
        write!(f, ")")
    }
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

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "Ident({})", self.name)
    }
}

impl BinOpKind<'_> {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOpKind::Add => "+",
            BinOpKind::Sub => "-",
            BinOpKind::Mul => "*",
            BinOpKind::Div => "/",
            BinOpKind::Eq => "==",
            BinOpKind::Neq => "!=",
            BinOpKind::Lt => "<",
            BinOpKind::Le => "<=",
            BinOpKind::Gt => ">",
            BinOpKind::Ge => ">=",
            BinOpKind::And => "&&",
            BinOpKind::Or => "||",
            BinOpKind::__Marker(_) => unreachable!(),
        }
    }
}
