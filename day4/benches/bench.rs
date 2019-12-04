use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    c.bench_function("2019 day 4 part one", |b| {
        b.iter(|| day4::part_1(234_208, 765_869));
    });

    c.bench_function("2019 day 4 part two", |b| {
        b.iter(|| day4::part_2(234_208, 765_869));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
