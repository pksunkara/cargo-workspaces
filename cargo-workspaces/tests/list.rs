use assert_cmd::Command;
use std::str::from_utf8;

#[test]
fn test_single() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/single")
        .arg("ws")
        .arg("ls")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert_eq!(out, "simple\n");
}

#[test]
fn test_long() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/single")
        .arg("ws")
        .arg("ll")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert_eq!(out, "simple v0.1.0 simple\n");
}

#[test]
fn test_all() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/private")
        .arg("ws")
        .arg("la")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert_eq!(out, "private (PRIVATE)\nsimple\n");
}

#[test]
fn test_long_all() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/private")
        .arg("ws")
        .arg("list")
        .arg("--long")
        .arg("--all")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert_eq!(
        out,
        "private v0.2.0      private (PRIVATE)\nsimple  v0.1.0-rc.0 simple\n"
    );
}

#[test]
fn test_json() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/private")
        .arg("ws")
        .arg("list")
        .arg("--json")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert!(out.contains(r#""name": "simple""#));
    assert!(out.contains(r#""version": "0.1.0-rc.0""#));
    assert!(out.contains(r#""private": false"#));

    assert!(!out.contains(r#""name": "private""#));
    assert!(!out.contains(r#""version": "0.2.0""#));
    assert!(!out.contains(r#""private": true"#));
}

#[test]
fn test_json_all() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/private")
        .arg("ws")
        .arg("list")
        .arg("--json")
        .arg("--all")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();

    assert!(out.contains(r#""name": "simple""#));
    assert!(out.contains(r#""version": "0.1.0-rc.0""#));
    assert!(out.contains(r#""private": false"#));

    assert!(out.contains(r#""name": "private""#));
    assert!(out.contains(r#""version": "0.2.0""#));
    assert!(out.contains(r#""private": true"#));
}

#[test]
fn test_json_conflicts_with_long() {
    let output = Command::cargo_bin("cargo-ws")
        .unwrap()
        .current_dir("../fixtures/private")
        .arg("ws")
        .arg("list")
        .arg("--long")
        .arg("--json")
        .output()
        .unwrap();
    let out = from_utf8(&output.stdout).unwrap();
    let err = from_utf8(&output.stderr).unwrap();

    assert_eq!(
        err,
        "private v0.2.0      private (PRIVATE)\nsimple  v0.1.0-rc.0 simple\n"
    );
}
