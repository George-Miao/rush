use std::fmt::Display;

use itertools::Itertools;
use pest::{iterators::Pair, Span};

use crate::{ast::*, error::Error, Rule};

pub trait Node<'a> {
    const RULE: Rule;
    fn span(&self) -> &Span<'a>;
    fn src<'b>(&self) -> &'b str
    where
        'a: 'b;
}

macro_rules! impl_single {
    ($ty:ident, $rule:ident, $val:ident => $constructor:expr) => {
        impl<'a> TryFrom<Pair<'a, Rule>> for $ty<'a> {
            type Error = Error<'a>;

            fn try_from($val: Pair<'a, Rule>) -> Result<Self, Self::Error> {
                ensure!($val, $rule);

                Ok($constructor)
            }
        }

        impl<'a> Node<'a> for $ty<'a> {
            const RULE: Rule = Rule::$rule;

            fn span(&self) -> &Span<'a> {
                &self.span
            }

            fn src<'b>(&self) -> &'b str
            where
                'a: 'b,
            {
                self.span().as_str()
            }
        }
    };
}

macro_rules! ensure {
    ($val:ident, $expect:ident) => {
        if $val.as_rule() != Rule::$expect {
            return Err(Error::TreeError {
                expect: Rule::$expect,
                found: $val.as_rule(),
                span: $val.as_span(),
            });
        }
    };
}

impl_single! {
    Ident, ident, value => Ident {
        name: value.as_str(),
        span: value.as_span(),
    }
}

impl_single! {
    Literal, literal, value => {
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
            Rule::string => LiteralKind::String(&inner.as_str()[1..inner.as_str().len() - 1]),
            _ => unreachable!("Literal should only be number, float or string"),
        };
        Literal {
        kind,
        span: span1,
    }}
}

impl_single!(Exec, exec, value => Exec {
    span: value.as_span(),
    cmd: value.as_str(),
});

impl_single!(Expr, expr, value => {
    let span = value.as_span();
    let inner = value.into_inner().next().expect("Expr should have content");
    let kind = match inner.as_rule() {
        Rule::ident => ExprKind::Ident(Ident::try_from(inner)?),
        Rule::literal => ExprKind::Literal(Literal::try_from(inner)?),
        Rule::fn_call => ExprKind::FnCall(FnCall::try_from(inner)?),
        Rule::exec => ExprKind::Exec(Exec::try_from(inner)?),
        Rule::block => ExprKind::Block(Block::try_from(inner)?),
        _ => unreachable!("Expr should only be ident, literal, fn_call, exec or block"),
    };

    Expr {
        kind,
        span,
    }
});

impl_single!(Stmt, stmt, value => {
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
});

impl_single!(FnCall, fn_call, value => {
    let span = value.as_span();
    let mut inner = value.into_inner();
    let ident = TryFrom::try_from(inner.next().expect("FnCall should have ident"))?;
    let args = match inner.next()  {
        Some(args) => {
            ensure!(args, expr_list);
            args.into_inner().map(Expr::try_from).collect::<Result<Vec<_>, _>>().unwrap()
        },
        None => vec![]
    };

    FnCall {
        ident,
        args,
        span,
    }
});

impl_single!(FnDef, fn_def, value => {
    let span = value.as_span();
    let mut inner = value.into_inner();
    let ident = TryFrom::try_from(inner.next().expect("FnDef should have ident"))?;
    let params = match inner.next()  {
        Some(args) => {
            ensure!(args, ident_list);
            args.into_inner().map(Ident::try_from).collect::<Result<Vec<_>, _>>().unwrap()
        },
        None => vec![]
    };

    let body = Block::try_from(inner.next().expect("FnDef should have block"))?;

    FnDef { span, ident, params, body }
});

impl_single!(Block, block, value => {
    let span = value.as_span();
    let items = value.into_inner().map(Item::try_from).try_collect()?;
    Block {
        span,
        items,
    }
});

impl_single!(Item, item, value => {
    let span = value.as_span();
    let inner = value.into_inner().next().expect("Item should have content");
    let kind = match inner.as_rule() {
        Rule::expr => ItemKind::Expr(Expr::try_from(inner)?),
        Rule::stmt => ItemKind::Stmt(Stmt::try_from(inner)?),
        Rule::fn_def => ItemKind::FnDef(FnDef::try_from(inner)?),
        _ => unreachable!("Item should have expr, stmt or fn_def"),
    };
    Item {
        kind,
        span,
    }
});

impl_single!(Tree, main, value => {
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
});

impl Display for FnDef<'_> {
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
