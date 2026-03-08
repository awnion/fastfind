mod common;

use common::*;

#[test]
fn not_operator() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["!", "-name", "*.txt"]);
}

#[test]
fn not_word() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-not", "-name", "*.txt"]);
}

#[test]
fn or_operator() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "*.txt", "-o", "-name", "*.rs"]);
}

#[test]
fn or_word() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "*.txt", "-or", "-name", "*.rs"]);
}

#[test]
fn and_explicit() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-a", "-name", "*.txt"]);
}

#[test]
fn and_word() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-and", "-name", "*.txt"]);
}

#[test]
fn grouping() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["(", "-name", "*.txt", "-o", "-name", "*.rs", ")"],
    );
}

#[test]
fn complex_expression() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["(", "-name", "*.txt", "-o", "-name", "*.rs", ")", "-type", "f"],
    );
}

#[test]
fn not_with_or() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["!", "(", "-name", "*.txt", "-o", "-name", "*.rs", ")"],
    );
}

#[test]
fn not_type() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["!", "-type", "d"]);
}

#[test]
fn nested_groups() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["(", "-type", "f", "-name", "*.txt", ")", "-o", "(", "-type", "d", "-name", "sub", ")"],
    );
}

#[test]
fn maxdepth_with_or() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-maxdepth", "1", "(", "-type", "f", "-o", "-type", "d", ")"],
    );
}
