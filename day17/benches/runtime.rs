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
                    day17::$name::parse(black_box(&input))
                        .unwrap()
                        .height_of_tower(2022)
                })
            });
        };
    }
    measure!(chamber);
    measure!(naive);
    measure!(relative);

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
