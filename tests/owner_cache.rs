mod common;

#[cfg(unix)]
#[test]
fn repeated_user_and_group_match_gnu_find() {
    use std::process::Command;

    use common::*;

    let dir = generate_test_dir();
    let root = dir.path().to_str().unwrap();
    let output = Command::new(GNU_FIND)
        .args([root, "-maxdepth", "0", "-printf", "%u\n%g\n"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let names = String::from_utf8(output.stdout).unwrap();
    let mut names = names.lines();
    let user = names.next().unwrap();
    let group = names.next().unwrap();

    for prefix in [vec![], vec!["-depth"]] {
        let mut args = prefix;
        args.extend(["-type", "f", "-user", user, "-group", group, "-user", user, "-group", group]);
        assert_same_output(root, &args);
    }
}
