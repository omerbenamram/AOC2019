use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

fn benchmark(c: &mut Criterion) {
    let s = fs::read_to_string(r#"input/opcodes"#).unwrap();

    c.bench_function("2019 day 9 part one", |b| {
        b.iter(|| day9::part_1(&s));
    });

    c.bench_function("2019 day 9 part two", |b| {
        b.iter(|| day9::part_2(&s));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
