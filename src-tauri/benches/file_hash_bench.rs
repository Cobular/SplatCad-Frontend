use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mylib::commands::local_files::get_all_data;


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| get_all_data(black_box(PathBuf::from(r#"C:\Users\jdc10\Documents\GrabCAD"#)))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);