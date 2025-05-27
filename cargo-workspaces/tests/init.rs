mod utils;
use insta::assert_snapshot;
use serial_test::serial;
use std::fs::{read_to_string, rename, write};

#[test]
fn test_no_path() {
    let err = utils::run_err("../fixtures/single", &["ws", "init", "new"]);
    assert_snapshot!(err);
}

#[test]
#[serial]
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

#[test]
#[serial]
fn test_normal_with_manifest() {
    let manifest = "../fixtures/normal/Cargo.toml";
    let content = read_to_string(manifest).unwrap();

    let err = utils::run_err("../fixtures/normal", &["ws", "init"]);
    assert_snapshot!(err);

    let data = read_to_string(manifest).unwrap();
    assert_snapshot!(data);

    // Restore Cargo.toml
    write(manifest, content).unwrap();
}

#[test]
#[serial]
fn test_root() {
    let manifest = "../fixtures/root/Cargo.toml";
    let backup = "../fixtures/root/Cargo.toml.bak";

    // Rename Cargo.toml
    rename(manifest, backup).unwrap();

    let err = utils::run_err("../fixtures/root", &["ws", "init"]);
    assert_snapshot!(err);

    let data = read_to_string(manifest).unwrap();
    assert_snapshot!(data);

    // Rename Cargo.toml
    rename(backup, manifest).unwrap();
}

#[test]
#[serial]
fn test_root_with_manifest() {
    let manifest = "../fixtures/root/Cargo.toml";
    let content = read_to_string(manifest).unwrap();

    let err = utils::run_err("../fixtures/root", &["ws", "init"]);
    assert_snapshot!(err);

    let data = read_to_string(manifest).unwrap();
    assert_snapshot!(data);

    // Restore Cargo.toml
    write(manifest, content).unwrap();
}

#[test]
#[serial]
fn test_root_with_manifest_no_workspace() {
    let manifest = "../fixtures/normal/Cargo.toml";
    let backup = "../fixtures/normal/Cargo.toml.bak";
    let root_manifest = "../fixtures/root/Cargo.toml";

    // Rename Cargo.toml
    rename(manifest, backup).unwrap();
    write(manifest, read_to_string(root_manifest).unwrap()).unwrap();

    let err = utils::run_err("../fixtures/normal", &["ws", "init"]);
    assert_snapshot!(err);

    let data = read_to_string(manifest).unwrap();
    assert_snapshot!(data);

    // Rename Cargo.toml
    rename(backup, manifest).unwrap();
}
