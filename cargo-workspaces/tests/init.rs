mod utils;
use insta::assert_snapshot;
use std::fs::{read_to_string, rename};

#[test]
fn test_no_path() {
    let err = utils::run_err("../fixtures/single", &["ws", "init", "new"]);
    assert_snapshot!(err);
}

#[test]
fn test_with_manifest() {
    let err = utils::run_err("../fixtures/single", &["ws", "init"]);
    assert_snapshot!(err);
}

#[test]
fn test_normal() {
    // Rename Cargo.toml
    rename(
        "../fixtures/normal/Cargo.toml",
        "../fixtures/normal/Cargo.toml.bak",
    )
    .unwrap();

    let err = utils::run_err("../fixtures/normal", &["ws", "init"]);
    assert_snapshot!(err);

    let manifest = read_to_string("../fixtures/normal/Cargo.toml").unwrap();
    assert_snapshot!(manifest);

    // Rename Cargo.toml
    rename(
        "../fixtures/normal/Cargo.toml.bak",
        "../fixtures/normal/Cargo.toml",
    )
    .unwrap();
}
