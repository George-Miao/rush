use std::assert_matches::assert_matches;

use pest::Parser;

use crate::{ast::*, parse, Rule, RushParser};

#[test]
fn test_parse() {
    let input = r#"
        fn main(arg1) {
            main(myself);
        }

        let a = 233;
        let main = "mian";
        "#;

    let tree = parse(input).unwrap().items;
    assert!(tree.len() == 3);
    assert_matches!(
        &tree[0].kind,
        ItemKind::FnDef(FnDef {
            ident: Ident { name: "main", .. },
            params,
            ..
        }) if params.len() == 1
    );
    assert_matches!(
        &tree[1].kind,
        ItemKind::Stmt(Stmt {
            ident: Ident { name: "a", .. },
            expr: Expr {
                kind: ExprKind::Literal(Literal {
                    kind: LiteralKind::Number(233),
                    ..
                }),
                ..
            },
            ..
        })
    );
    assert_matches!(
        &tree[2].kind,
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
    let input = r#"
        fn test(a, b) {
            print(a, b);
        };
    "#;

    let tree = parse(input).unwrap().items;
    assert!(tree.len() == 1);
    assert_matches!(
        &tree[0].kind,
        ItemKind::FnDef(FnDef {
            ident: Ident { name: "main", .. },
            params,
            ..
        }) if params.len() == 1
    );
}

#[test]
fn test_literal() {
    let float = RushParser::parse(Rule::literal, "114.514")
        .unwrap()
        .next()
        .unwrap();
    let float = Literal::try_from(float);
    assert_matches!(
        float,
        Ok(Literal {
            kind: LiteralKind::Float(_),
            ..
        })
    );
}
