use assert_cmd::Command;
use std::str::from_utf8;

fn run(dir: &str, args: &[&str]) -> (String, String) {
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
