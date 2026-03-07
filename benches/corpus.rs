#![allow(dead_code)]

use std::io::Write;

use tempfile::TempDir;

/// Generate a realistic directory tree for benchmarking find operations.
///
/// Structure per module (repeated `num_modules` times):
///   src/module_NNNN/
///     lib.rs, mod.rs, utils.rs
///     tests/
///       test_NNNN.rs
///     sub_0..sub_depth/
///       file_NNNN.rs, data_NNNN.log, config_NNNN.toml
///
/// Total files ≈ num_modules * (3 + 1 + depth * 3)
/// Total dirs  ≈ num_modules * (1 + 1 + depth)
pub fn generate_tree(num_modules: usize, depth: usize) -> TempDir {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();

    for i in 0..num_modules {
        let module = src.join(format!("module_{i:04}"));
        std::fs::create_dir_all(&module).unwrap();

        // top-level source files
        for name in ["lib.rs", "mod.rs", "utils.rs"] {
            let mut f = std::fs::File::create(module.join(name)).unwrap();
            writeln!(f, "// {name} for module {i}").unwrap();
        }

        // tests subdirectory
        let tests = module.join("tests");
        std::fs::create_dir_all(&tests).unwrap();
        let mut f = std::fs::File::create(tests.join(format!("test_{i:04}.rs"))).unwrap();
        writeln!(f, "#[test] fn test_{i}() {{}}").unwrap();

        // nested subdirectories
        let mut current = module.clone();
        for d in 0..depth {
            current = current.join(format!("sub_{d}"));
            std::fs::create_dir_all(&current).unwrap();

            let mut f = std::fs::File::create(current.join(format!("file_{i:04}.rs"))).unwrap();
            writeln!(f, "// nested file depth {d}").unwrap();

            let mut f = std::fs::File::create(current.join(format!("data_{i:04}.log"))).unwrap();
            writeln!(f, "log entry {i} depth {d}").unwrap();

            let mut f = std::fs::File::create(current.join(format!("config_{i:04}.toml"))).unwrap();
            writeln!(f, "[module_{i}]").unwrap();
        }
    }

    dir
}
