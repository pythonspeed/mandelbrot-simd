use criterion::{criterion_group, criterion_main, Criterion};

use mandelbrot_lib::{Algorithm, Mandelbrot};

fn bench_mb(c: &mut Criterion) {
    c.bench_function("mandelbrot", |b| {
        b.iter(|| {
            let mb=Mandelbrot::generate((3200, 3200),Algorithm::CompilerSimd);
            std::hint::black_box(mb);
        });
    });
}

criterion_group!(benches, bench_mb);
criterion_main!(benches);