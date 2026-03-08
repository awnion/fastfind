mod common;

use std::fs;

use common::*;

// -- print --

#[test]
fn print_action_explicit() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-print"]);
}

#[test]
fn print0_action() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-print0"]);
}

// -- exec --

#[test]
fn exec_echo() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-type", "f", "-name", "*.txt", "-exec", "echo", "{}", ";"],
    );
}

#[test]
fn exec_batch() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-type", "f", "-name", "*.txt", "-exec", "echo", "{}", "+"],
    );
}

// -- delete --

#[test]
fn delete_action() {
    let dir = generate_test_dir();
    let del_dir = dir.path().join("to_delete");
    fs::create_dir(&del_dir).unwrap();
    fs::write(del_dir.join("a.tmp"), "a").unwrap();
    fs::write(del_dir.join("b.tmp"), "b").unwrap();

    let output = std::process::Command::new(fastfind_bin())
        .args([del_dir.to_str().unwrap(), "-name", "*.tmp", "-delete"])
        .output()
        .unwrap();
    assert!(output.status.success());

    assert!(!del_dir.join("a.tmp").exists());
    assert!(!del_dir.join("b.tmp").exists());
}

// -- prune --

#[test]
fn prune_action() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-name", "sub", "-prune", "-o", "-type", "f", "-print"],
    );
}

// -- depth / -d --

#[test]
fn depth_option() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-depth"]);
}

#[test]
fn depth_d_option() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-d"]);
}

// -- boolean literals / global options --

#[test]
fn true_test() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-true"]);
}

#[test]
fn false_test() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-false"]);
}

#[test]
fn noleaf_option() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-noleaf"]);
}
