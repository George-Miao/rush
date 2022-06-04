use color_eyre::eyre::Context;
use rush_interpreter::*;

#[allow(clippy::unnecessary_wraps)]
fn print(args: Vec<Value>) -> RuntimeResult<Value> {
    let mut iter = args.into_iter().peekable();
    loop {
        let arg = iter.next().unwrap();
        if iter.peek().is_none() {
            print!("{}", arg);
            break;
        }
        print!("{} ", arg);
    }
    Ok(Value::Unit)
}

fn main() {
    color_eyre::install().unwrap();

    let src = include_str!("../../data/example.rush");

    Engine::new()
        .with_fn("add", |a: i64, b: i64| Ok(Value::Int(a + b)))
        .with_fn("minus", |a: i64, b: i64| Ok(Value::Int(a - b)))
        .with_fn_raw("print", print)
        .with_fn_raw("println", |args| {
            let ret = print(args)?;
            println!();
            Ok(ret)
        })
        .execute(src)
        .wrap_err("Engine error")
        .unwrap();
}
