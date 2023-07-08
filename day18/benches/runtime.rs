use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

static INPUT: &str = include_str!("../data/input.txt");

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("run");
    macro_rules! measure {
        ($name:ident) => {
            let input = String::from(INPUT);
            group.bench_function(stringify!($name), |b| {
                b.iter(|| {
                    day18::$name::parse(black_box(&input))
                        .unwrap()
                        .exposed_area()
                })
            });
        };
    }

    // The exposed area for the naive implementation is just a placeholder method
    // measure!(naive);
    measure!(dfs1);
    measure!(dfs2);
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
