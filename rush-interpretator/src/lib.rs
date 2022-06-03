#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::all)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod_use::mod_use![engine, value, utils, error, fn_ptr, scope, var, refs];

#[test]
fn test_run() {
    use color_eyre::eyre::Context;

    color_eyre::install().unwrap();

    let src = r#"
        let a = 1;
        let b = 2;
        print(add(1,2));
        println(1);
        println(a, b, 114.115);
        {
            let a = "Not number";
            fn test(a, b) {
                print(a);
            }
            print(a);
            test(1, 2);
        };
        print(a, b);
    "#;

    Engine::new()
        .with_fn("add", |a: i128, b: i128| Ok(Value::Int(a + b)))
        .with_fn_raw("println", |args: Vec<Shared<Locked<Value>>>| {
            for arg in args {
                let arg = &*arg.get();
                print!("{}, ", arg);
            }
            println!();
            Ok(Value::Unit)
        })
        .with_fn_raw("print", |args: Vec<Shared<Locked<Value>>>| {
            for arg in args {
                let arg = &*arg.get();
                print!("{}, ", arg);
            }
            Ok(Value::Unit)
        })
        .execute(src)
        .wrap_err("Engine error")
        .unwrap();
}
