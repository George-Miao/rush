use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rush_interpreter::*;

fn loop_speed(n: i64) {
    let mut scope = Scope::new_global();

    scope.new_var("test", Value::Int(n));

    while *scope
        .get(black_box("test"))
        .unwrap()
        .value_ref()
        .cast_ref::<i64>()
        .unwrap()
        != 0
    {
        let new_val = *scope
            .get(black_box("test"))
            .unwrap()
            .value_ref()
            .cast_ref::<i64>()
            .unwrap()
            - 1;

        scope
            .get_mut(black_box("test"))
            .unwrap()
            .update(new_val.into());
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.benchmark_group("update variable")
        .bench_function("var 10000", |b| b.iter(|| loop_speed(black_box(10000))))
        .bench_function("var 100000", |b| b.iter(|| loop_speed(black_box(100000))))
        .bench_function("var 1000000", |b| b.iter(|| loop_speed(black_box(1000000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
