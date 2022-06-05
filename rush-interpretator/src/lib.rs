#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::all)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod_use::mod_use![engine, error];

#[cfg(feature = "bin")]
pub fn run() -> color_eyre::Result<()> {
    use color_eyre::eyre::{Context as EyreContext, ContextCompat};
    use parser::parse;

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

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(args: Vec<Value>) -> RuntimeResult<Value> {
        if args.len() != 1 {
            return Err(RuntimeError::ArgumentError {
                ident: "typeof".to_string(),
                expected: 1,
                found: args.len(),
            });
        }
        Ok(Value::Str(args[0].type_name().to_owned().shared()))
    }

    color_eyre::install().unwrap();

    let path = std::env::args()
        .nth(1)
        .wrap_err_with(|| format!("Usage: {} <path>", std::env::args().next().unwrap()))?;

    let src = std::fs::read_to_string(&path).wrap_err("Failed to load source file")?;

    Engine::new()
        .with_fn("add", |a: i64, b: i64| Ok(Value::Int(a + b)))
        .with_fn("minus", |a: i64, b: i64| Ok((a - b).into()))
        .with_fn_raw("print", print)
        .with_fn_raw("println", |args| {
            let ret = print(args)?;
            println!();
            Ok(ret)
        })
        .with_fn_raw("type_of", type_of)
        .execute(&src)
        .unwrap();

    Ok(())
}
