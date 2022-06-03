mod_use::mod_use![engine, error, value, utils];

#[test]
fn test() {
    use parser::Parser;

    let code = r#"
            let a = 1;
            let b = 2;
            let c = "testtest";
            println(a, b, c);
    "#;

    let tree = parse(code).expect("Failed to parse");
    // println!("{:#?}", tree);
    Engine::new().run(tree).unwrap();
}

// fn hoist(context: &mut Context, item: Item) {
//     match item {
//         Item::Stmt(Stmt { ident, expr }) => {}
//         _ => {}
//     }
// }
