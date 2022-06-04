use std::assert_matches::assert_matches;

use itertools::Itertools;
use pest::Parser;

use crate::{ast::*, parse, Rule, RushParser};

macro_rules! assert_parse {
    ($src:expr, $rule:ident, $( $pattern:pat_param )+ $(|)? $( if $guard:expr )?) => {
        let res = RushParser::parse(Rule::$rule, $src)
            .unwrap()
            .next()
            .unwrap();
        let res = TryFrom::try_from(res).unwrap();
        assert_matches!(res, $($pattern)+ $( if $guard )? );
    };
}

#[test]
fn test_parse() {
    let input = r#"
        fn main(arg1) {
            main(myself);
        }

        let a = 233 + c;
        let main = "mian";
        "#;

    let tree = parse(input).unwrap().items;
    assert!(tree.len() == 3);
    let (fn_def, stmt1, stmt2) = tree.into_iter().next_tuple().unwrap();

    assert_matches!(
        fn_def.kind,
        ItemKind::FnDef(FnDef {
            ident: Ident { name: "main", .. },
            params,
            ..
        }) if params.len() == 1
    );

    assert_matches!(
        stmt1.kind,
        ItemKind::Stmt(Stmt {
            ident: Ident { name: "a", .. },
            expr: Expr {
                kind: ExprKind::BinOp(BinOpExpr {
                    left,
                    right,
                    kind: BinOpKind::Add,
                    ..
                }),
                ..
            },
            ..
        }) if matches!(&*left, Expr {
            kind: ExprKind::Literal(Literal {
                kind: LiteralKind::Number(233),
                ..
            }),
            ..
        }) && matches!(&*right, Expr {
            kind: ExprKind::Ident(Ident { name: "c", .. }),
            ..
        })
    );

    assert_matches!(
        stmt2.kind,
        ItemKind::Stmt(Stmt {
            ident: Ident { name: "main", .. },
            expr: Expr {
                kind: ExprKind::Literal(Literal {
                    kind: LiteralKind::String("mian"),
                    ..
                }),
                ..
            },
            ..
        })
    );
}

#[test]
fn test_fn_def() {
    let input = r#"fn test(a, b) { print(a, b); }"#;
    assert_parse!(
        input,
        item,
        Item {
            kind: ItemKind::FnDef(FnDef {
                ident: Ident { name: "test", .. },
                params,
                ..
            }),
            ..
        } | if params.len() == 2
    );
}

#[test]
fn test_literal() {
    assert_parse!(
        "114.514",
        literal,
        Literal {
            kind: LiteralKind::Float(_),
            ..
        }
    );

    assert_parse!(
        "114.514",
        expr,
        Expr {
            kind: ExprKind::Literal(Literal {
                kind: LiteralKind::Float(_),
                ..
            }),
            ..
        }
    );
}

#[test]
fn test_exec() {
    assert_parse!("$`ls -al`", exec, Exec { cmd: "ls -al", .. });
}

#[test]
fn test_un_op() {
    assert_parse!(
        "!a",
        un_op_expr,
        UnOpExpr {
            kind: UnOpKind::Not,
            expr,
            ..
        } | if matches!(&*expr, Expr {
            kind: ExprKind::Ident(Ident { name: "a", .. }),
            ..
        }
    ));
}

#[test]
fn test_bin_op() {
    assert_parse! {
        "a + b",
        bin_op_expr,
        BinOpExpr {
            kind: BinOpKind::Add,
            left,
            right,
            ..
        } | if matches!(&*left, Expr {
            kind: ExprKind::Ident(Ident { name: "a", .. }),
            ..
        }) && matches!(&*right, Expr {
            kind: ExprKind::Ident(Ident { name: "b", .. }),
            ..
        })
    };
}

#[test]
fn test_loop() {
    assert_parse!(
        "while true { print(a); }",
        while_loop,
        While {
            expr: Expr {
                kind: ExprKind::Literal(Literal {
                    kind: LiteralKind::Bool(true),
                    ..
                }),
                ..
            },
            block: Block { items, .. },
            ..
        } | if matches!(*items, [
            Item {
                kind: ItemKind::Expr(
                    Expr {
                        kind: ExprKind::FnCall(
                            FnCall {
                                ident: Ident {
                                    name: "print",
                                    ..
                                },
                                ..
                            }
                        ),
                        ..
                    },
                ..
            ),
            ..
        }])
    );
}

#[test]
fn test_some() {
    let res = RushParser::parse(
        Rule::item,
        "if a == b {
        while true {
            print(a);
        }
    } else {
        exit(b);
    }",
    )
    .unwrap()
    .next()
    .unwrap();

    let res = Item::try_from(res).unwrap();
    println!("{:#?}", res);
}
