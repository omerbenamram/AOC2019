use criterion::{criterion_group, criterion_main, Criterion};
use day1::{part_1, part_2};
use std::fs;

fn benchmark(c: &mut Criterion) {
    let s = fs::read_to_string(r#"input\modules"#).unwrap();

    c.bench_function("2019 day 1 part one", |b| {
        b.iter(|| day1::part_1(&s));
    });

    c.bench_function("2019 day 1 part two", |b| {
        b.iter(|| day1::part_2(&s));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);