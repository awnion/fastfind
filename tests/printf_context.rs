mod common;

use std::process::Command;

use tempfile::TempDir;

#[test]
fn printf_and_fprintf_preserve_each_starting_point() {
    let first = common::generate_test_dir();
    let second = common::generate_test_dir();
    let output_dir = TempDir::new().unwrap();
    let format = "%H|%P|%p\n";
    let mut results = Vec::new();

    for (index, binary) in
        [std::path::PathBuf::from(common::GNU_FIND), common::fastfind_bin()].iter().enumerate()
    {
        let destination = output_dir.path().join(format!("output-{index}"));
        let output = Command::new(binary)
            .arg(first.path())
            .arg(second.path())
            .args(["-type", "f", "-printf", format, "-fprintf"])
            .arg(&destination)
            .arg(format)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stderr.is_empty());
        let sorted_lines = |bytes: Vec<u8>| {
            let mut lines: Vec<String> =
                String::from_utf8(bytes).unwrap().lines().map(String::from).collect();
            lines.sort();
            lines
        };
        let stdout = sorted_lines(output.stdout);
        let file = sorted_lines(std::fs::read(destination).unwrap());
        assert_eq!(stdout.len(), 10);
        assert_eq!(stdout, file);
        results.push(stdout);
    }
    assert_eq!(results[0], results[1]);
}
