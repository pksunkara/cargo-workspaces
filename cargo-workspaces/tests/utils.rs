#![allow(dead_code)]
use assert_cmd::Command;
use std::str::from_utf8;

pub fn run(dir: &str, args: &[&str]) -> (String, String) {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();

    let out = from_utf8(&output.stdout).unwrap();
    let err = from_utf8(&output.stderr).unwrap();

    (out.to_string(), err.to_string())
}

pub fn run_out(dir: &str, args: &[&str]) -> String {
    let (out, err) = run(dir, args);

    assert!(err.is_empty());
    out
}

pub fn run_err(dir: &str, args: &[&str]) -> String {
    let (out, err) = run(dir, args);

    assert!(out.is_empty());
    err
}

/// Makes the output of commands suitable for snapshot testing:
/// - Removes the error for missing `http.cainfo` config value.
/// - Removes `cargo build` output.
pub fn normalize_output(input: &mut String) {
    *input = input
        .lines()
        .filter(|line| {
            // `cargo build` output starts with 3 spaces.
            // Depending on configuration, there may be also ANSI escape codes,
            // so we're performing the simplest check possible.
            if strip_ansi_escapes::strip_str(line).starts_with("   ") {
                return false;
            }
            // `cargo` may warn about missing `http.cainfo` config value, and it
            // depends on the user configuration.
            if line.contains("http.cainfo") {
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n");
}
