mod common;

use common::*;

#[test]
fn default_lists_all_entries() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &[]);
}

#[test]
fn type_f_files_only() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f"]);
}

#[test]
fn type_d_directories_only() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "d"]);
}

#[test]
fn name_glob_pattern() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "*.txt"]);
}

#[test]
fn name_exact_match() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "file1.txt"]);
}

#[test]
fn name_no_match() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "nonexistent"]);
}

#[test]
fn name_hidden_file() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", ".hidden"]);
}

#[test]
fn name_question_mark_glob() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "file?.txt"]);
}

#[test]
fn name_bracket_glob() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "file[12].txt"]);
}

#[test]
fn maxdepth_zero() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-maxdepth", "0"]);
}

#[test]
fn maxdepth_one() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-maxdepth", "1"]);
}

#[test]
fn maxdepth_two() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-maxdepth", "2"]);
}

#[test]
fn mindepth_one() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-mindepth", "1"]);
}

#[test]
fn mindepth_two() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-mindepth", "2"]);
}

#[test]
fn mindepth_exceeds_tree() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-mindepth", "100"]);
}

#[test]
fn combined_type_and_name() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-name", "*.txt"]);
}

#[test]
fn combined_maxdepth_and_type() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-maxdepth", "1", "-type", "f"]);
}

#[test]
fn combined_mindepth_maxdepth() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-mindepth", "1", "-maxdepth", "2"]);
}

#[test]
fn combined_all_filters() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-mindepth", "1", "-maxdepth", "2", "-type", "f", "-name", "*.txt"],
    );
}

#[test]
fn type_f_name_log() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-name", "*.log"]);
}

#[test]
fn empty_dir_type_d() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "d", "-name", "empty"]);
}

#[test]
fn exit_code_zero_on_match() {
    let dir = generate_test_dir();
    let (_, exit) = run_fast(dir.path().to_str().unwrap(), &[]);
    assert_eq!(exit, 0);
}

#[test]
fn nonexistent_directory() {
    let (_, exit) = run_fast("/tmp/nonexistent_fastfind_test_dir_12345", &[]);
    assert_ne!(exit, 0);
}
