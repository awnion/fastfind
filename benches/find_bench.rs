mod corpus;

use std::process::Command;

use corpus::generate_tree;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

fn fastfind_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path.push("find");
    if !path.exists() {
        let output =
            Command::new("cargo").args(["metadata", "--format-version", "1"]).output().unwrap();
        let meta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let target_dir = meta["target_directory"].as_str().unwrap();
        path = std::path::PathBuf::from(target_dir).join("release").join("find");
        if !path.exists() {
            path = std::path::PathBuf::from(target_dir).join("debug").join("find");
        }
    }
    path
}

// --- Benchmarks ---

/// List all entries (default: no filters)
fn bench_list_all(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let ff = fastfind_bin();

    let mut group = c.benchmark_group("list_all");

    group.bench_function("fastfind", |b| {
        b.iter(|| {
            let out = Command::new(&ff).arg(&dir_str).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Filter by -type f
fn bench_type_f(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let ff = fastfind_bin();

    let mut group = c.benchmark_group("type_f");

    group.bench_function("fastfind", |b| {
        b.iter(|| {
            let out = Command::new(&ff).args([&dir_str, "-type", "f"]).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Filter by -name '*.rs'
fn bench_name_glob(c: &mut Criterion) {
    let dir = generate_tree(100, 3);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let ff = fastfind_bin();

    let mut group = c.benchmark_group("name_glob_rs");

    group.bench_function("fastfind", |b| {
        b.iter(|| {
            let out = Command::new(&ff).args([&dir_str, "-name", "*.rs"]).output().unwrap();
            assert!(out.status.success());
        });
    });

    group.finish();
}

/// Combined: -type f -name '*.rs' -maxdepth 3
fn bench_combined(c: &mut Criterion) {
    let dir = generate_tree(100, 5);
    let dir_str = dir.path().to_str().unwrap().to_string();
    let ff = fastfind_bin();

    let mut group = c.benchmark_group("combined_type_name_maxdepth");

    group.bench_function("fastfind", |b| {
        b.iter(|| {
            let out = Command::new(&ff)
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
    let ff = fastfind_bin();
    let mut group = c.benchmark_group("scaling");
    group.sample_size(10);

    for num_modules in [50, 200, 500] {
        let dir = generate_tree(num_modules, 3);
        let dir_str = dir.path().to_str().unwrap().to_string();

        group.bench_with_input(
            BenchmarkId::new("fastfind", num_modules),
            &dir_str,
            |b, dir_str| {
                b.iter(|| {
                    Command::new(&ff).args([dir_str, "-type", "f"]).output().unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_list_all,
    bench_type_f,
    bench_name_glob,
    bench_combined,
    bench_scaling,
);
criterion_main!(benches);
