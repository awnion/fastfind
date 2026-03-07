#[path = "../benches/corpus.rs"]
mod corpus;

use std::process::Command;

use corpus::generate_tree;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

/// Baseline find binary path. Defaults to system `find`, override with BASELINE_FIND env var.
fn baseline_find() -> String {
    std::env::var("BASELINE_FIND").unwrap_or_else(|_| "find".into())
}

// --- Benchmarks ---

/// List all entries
fn bench_list_all(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let find = baseline_find();

    let mut group = c.benchmark_group("baseline_list_all");

    group.bench_function("find", |b| {
        b.iter(|| {
            let out = Command::new(&find).arg(&dir_str).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Filter by -type f
fn bench_type_f(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let find = baseline_find();

    let mut group = c.benchmark_group("baseline_type_f");

    group.bench_function("find", |b| {
        b.iter(|| {
            let out = Command::new(&find).args([&dir_str, "-type", "f"]).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Filter by -name '*.rs'
fn bench_name_glob(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let find = baseline_find();

    let mut group = c.benchmark_group("baseline_name_glob_rs");

    group.bench_function("find", |b| {
        b.iter(|| {
            let out = Command::new(&find).args([&dir_str, "-name", "*.rs"]).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Combined: -type f -name '*.rs' -maxdepth 3
fn bench_combined(c: &mut Criterion) {
    let dir = generate_tree(100, 5);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let find = baseline_find();

    let mut group = c.benchmark_group("baseline_combined_type_name_maxdepth");

    group.bench_function("find", |b| {
        b.iter(|| {
            let out = Command::new(&find)
                .args([&dir_str, "-type", "f", "-name", "*.rs", "-maxdepth", "3"])
                .output()
                .unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Scaling: increasing number of modules
fn bench_scaling(c: &mut Criterion) {
    let find = baseline_find();
    let mut group = c.benchmark_group("baseline_scaling");
    group.sample_size(10);

    for num_modules in [50, 200, 500] {
        let dir = generate_tree(num_modules, 3);
        let dir_str = dir.path().to_str().unwrap().to_string();

        group.bench_with_input(BenchmarkId::new("find", num_modules), &dir_str, |b, dir_str| {
            b.iter(|| {
                Command::new(&find).args([dir_str, "-type", "f"]).output().unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    baselines,
    bench_list_all,
    bench_type_f,
    bench_name_glob,
    bench_combined,
    bench_scaling,
);
criterion_main!(baselines);
