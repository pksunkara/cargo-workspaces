#![allow(dead_code)]
use assert_cmd::Command;
use regex::Regex;
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
    // This warning is emitted by cargo when trying to fetch value; can be either present
    // or not depending on the user configuration.
    *input = input.replace("error: config value `http.cainfo` is not set\n", "");

    // `cargo build` output starts with 4 spaces.
    // The actual output cannot be reliably asserted (e.g. paths can differ, package may compile or not, etc)
    // so we fully strip any lines that start with 4 spaces.
    let re = Regex::new(r"^\s{4}.*$").unwrap();
    *input = input
        .lines()
        .filter_map(|line| if re.is_match(line) { None } else { Some(line) })
        .collect::<Vec<_>>()
        .join("\n");
}
