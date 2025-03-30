use criterion::{Criterion, criterion_group, criterion_main};
use timeflake::Timeflake;

fn bench_generate_random_flake(c: &mut Criterion) {
    let mut rng = rand::rng();

    c.bench_function("Generate Random Timeflake", |b| {
        b.iter(|| {
            let _flake = Timeflake::new_random(&mut rng);
        })
    });
}

criterion_group!(benches, bench_generate_random_flake,);
criterion_main!(benches);
