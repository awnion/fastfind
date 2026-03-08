mod common;

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use common::*;

// -- name matching --

#[test]
fn iname_case_insensitive() {
    let dir = generate_test_dir();
    fs::write(dir.path().join("FILE1.TXT"), "upper").unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-iname", "file1.txt"]);
}

#[test]
fn path_matching() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-path", "*/sub/*"]);
}

#[test]
fn wholename_same_as_path() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-wholename", "*/sub/*"]);
}

#[test]
fn ipath_case_insensitive() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-ipath", "*/SUB/*"]);
}

#[test]
fn name_with_dot_prefix() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", ".*"]);
}

// -- type --

#[test]
fn type_comma_separated() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f,d"]);
}

#[test]
fn empty_file() {
    let dir = generate_test_dir();
    fs::write(dir.path().join("empty_file"), "").unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-empty"]);
}

#[test]
fn empty_dir() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-empty", "-type", "d"]);
}

#[test]
fn type_f_with_empty() {
    let dir = generate_test_dir();
    fs::write(dir.path().join("zero_size"), "").unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-empty"]);
}

// -- size --

#[test]
fn size_bytes() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-size", "0c"]);
}

#[test]
fn size_plus() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-size", "+0c"]);
}

#[test]
fn size_minus() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-size", "-100c"]);
}

#[test]
fn size_kilobytes() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-size", "-1k"]);
}

// -- permissions --

#[cfg(unix)]
#[test]
fn perm_exact() {
    let dir = generate_test_dir();
    let file = dir.path().join("perm_test");
    fs::write(&file, "test").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-name", "perm_test", "-perm", "644"]);
}

#[cfg(unix)]
#[test]
fn perm_at_least() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-perm", "-644"]);
}

#[cfg(unix)]
#[test]
fn perm_any() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-perm", "/111"]);
}

#[test]
fn readable_test() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-readable"]);
}

#[test]
fn writable_test() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-writable"]);
}

// -- time --

#[test]
fn mmin_minus() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-mmin", "-10"]);
}

#[test]
fn mtime_plus() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-mtime", "+0"]);
}

#[test]
fn newer_than_file() {
    let dir = generate_test_dir();
    let ref_file = dir.path().join("ref_file");
    fs::write(&ref_file, "ref").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(dir.path().join("newer_file"), "new").unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-newer", ref_file.to_str().unwrap()]);
}

// -- user/group --

#[cfg(unix)]
#[test]
fn user_current() {
    let dir = generate_test_dir();
    let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-user", &user]);
}

#[cfg(unix)]
#[test]
fn uid_current() {
    let dir = generate_test_dir();
    let uid = unsafe { libc::getuid() };
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-uid", &uid.to_string()]);
}

// -- links --

#[test]
fn links_count() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "f", "-links", "1"]);
}

// -- depth constraints combined with filters --

#[test]
fn mindepth_maxdepth_with_name() {
    let dir = generate_test_dir();
    assert_same_output(
        dir.path().to_str().unwrap(),
        &["-mindepth", "1", "-maxdepth", "1", "-name", "*.txt"],
    );
}
