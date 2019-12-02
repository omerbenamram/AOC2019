use criterion::{criterion_group, criterion_main, Criterion};
use day2::{part_1, part_2};
use std::fs;

fn benchmark(c: &mut Criterion) {
    let s = fs::read_to_string(r#"input\opcodes"#).unwrap();

    c.bench_function("2019 day 2 part one", |b| {
        b.iter(|| day2::part_1(&s));
    });

    c.bench_function("2019 day 2 part two", |b| {
        b.iter(|| day2::part_2(&s, 19_690_720));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
