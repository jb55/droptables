use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use droptables::DropTable;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

fn gen_pairs(n: usize) -> Vec<(usize, f32)> {
    let mut rng = Pcg32::seed_from_u64(777);
    (0..n).map(|i| (i, 0.1 + rng.random::<f32>())).collect()
}

fn bench_droptable_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("droptable_build");
    for &n in &[2usize, 8, 64, 256, 1024] {
        let pairs = gen_pairs(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("from_pairs_n={n}"), |b| {
            b.iter(|| black_box(DropTable::from_pairs(black_box(pairs.clone()))).unwrap());
        });
    }
    group.finish();
}

fn bench_droptable_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("droptable_sample");
    const DRAWS_PER_ITER: usize = 1024;

    for &n in &[2usize, 8, 64, 256, 1024] {
        let dt: DropTable<usize> = DropTable::from_pairs(gen_pairs(n)).unwrap();
        group.throughput(Throughput::Elements((DRAWS_PER_ITER * n) as u64));

        group.bench_function(format!("sample_ref_n={n}"), |b| {
            b.iter_batched_ref(
                || Pcg32::seed_from_u64(999),
                |rng| {
                    let mut s = 0usize;
                    for _ in 0..DRAWS_PER_ITER {
                        s ^= *dt.sample(rng);
                    }
                    black_box(s)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(format!("sample_owned_n={n}"), |b| {
            b.iter_batched_ref(
                || Pcg32::seed_from_u64(1001),
                |rng| {
                    let mut s = 0usize;
                    for _ in 0..DRAWS_PER_ITER {
                        s ^= dt.sample_owned(rng);
                    }
                    black_box(s)
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(droptable, bench_droptable_build, bench_droptable_sample);
criterion_main!(droptable);
