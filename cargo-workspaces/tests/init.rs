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
    let manifest = "../fixtures/normal/Cargo.toml";
    let backup = "../fixtures/normal/Cargo.toml.bak";

    // Rename Cargo.toml
    rename(manifest, backup).unwrap();

    let err = utils::run_err("../fixtures/normal", &["ws", "init"]);
    assert_snapshot!(err);

    let data = read_to_string(manifest).unwrap();
    assert_snapshot!(data);

    // Rename Cargo.toml
    rename(backup, manifest).unwrap();
}
