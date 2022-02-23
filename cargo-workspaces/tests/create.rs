mod utils;
use insta::assert_snapshot;
use std::{
    fs::{read_to_string, remove_dir, remove_dir_all, remove_file},
    path::Path,
};

/// Clean up a cargo or cargo-workspace created package directory
/// This doesn't use remove_dir_all, so it's safer, but more liable to break
/// if cargo new is updated in the future
/// package_dir is the directory containing the package
/// package_type is bin or lib
fn clean_package_dir(package_path: &Path, package_type: &str) {
    assert_ne!(package_path.as_os_str(), "");
    assert_ne!(package_path.as_os_str(), "/");

    let exists = package_path.exists();
    if !exists {
        return;
    }

    let cargo_path = package_path.join("Cargo.toml");
    let exists = cargo_path.exists();
    if exists {
        remove_file(cargo_path).unwrap();
    }

    let src_path = match package_type {
        "bin" => package_path.join("src").join("main.rs"),
        "lib" => package_path.join("src").join("lib.rs"),
        _ => {
            return;
        }
    };

    let exists = src_path.exists();
    if exists {
        remove_file(src_path).unwrap();
    }

    let src_path = package_path.join("src");
    let exists = src_path.exists();
    if exists {
        remove_dir(src_path).unwrap();
    }

    let git_path = package_path.join(".git");
    let gitignore_path = package_path.join(".gitignore");
    let exists = git_path.exists();
    if exists {
        remove_dir_all(git_path).unwrap();
        remove_file(gitignore_path).unwrap();
    }

    let exists = package_path.exists();
    if exists {
        remove_dir(package_path).unwrap();
    }
}

/// Test creating a 2015 bin package
#[test]
fn test_create_bin_2015() {
    let package_name = "mynewcrate-bin-2015";
    let dir = "../fixtures/normal";
    let package_path = Path::new(dir).join(package_name);
    let manifest_path = package_path.join("Cargo.toml");

    clean_package_dir(&package_path, "bin");

    let _err = utils::run_err(
        dir,
        &[
            "ws",
            "create",
            package_name,
            "--edition",
            "2015",
            "--bin",
            "--name",
            package_name,
        ],
    );

    let manifest = read_to_string(manifest_path).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_path, "bin");
}

/// Test creating a 2015 lib package
#[test]
fn test_create_lib_2015() {
    let package_name = "mynewcrate-lib-2015";
    let dir = "../fixtures/normal";
    let package_path = Path::new(dir).join(package_name);
    let manifest_path = package_path.join("Cargo.toml");

    clean_package_dir(&package_path, "lib");

    let _err = utils::run_err(
        dir,
        &[
            "ws",
            "create",
            package_name,
            "--edition",
            "2015",
            "--lib",
            "--name",
            package_name,
        ],
    );

    let manifest = read_to_string(manifest_path).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_path, "lib");
}

/// Test creating a 2018 bin package
#[test]
fn test_create_bin_2018() {
    let package_name = "mynewcrate-bin-2018";
    let dir = "../fixtures/normal";
    let package_path = Path::new(dir).join(package_name);
    let manifest_path = package_path.join("Cargo.toml");

    clean_package_dir(&package_path, "bin");

    let _err = utils::run_err(
        dir,
        &[
            "ws",
            "create",
            package_name,
            "--edition",
            "2018",
            "--bin",
            "--name",
            package_name,
        ],
    );

    let manifest = read_to_string(manifest_path).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_path, "bin");
}

/// Test creating a 2018 lib package
#[test]
fn test_create_lib_2018() {
    let package_name = "mynewcrate-lib-2018";
    let dir = "../fixtures/normal";
    let package_path = Path::new(dir).join(package_name);
    let manifest_path = package_path.join("Cargo.toml");

    clean_package_dir(&package_path, "lib");

    let _err = utils::run_err(
        dir,
        &[
            "ws",
            "create",
            package_name,
            "--edition",
            "2018",
            "--lib",
            "--name",
            package_name,
        ],
    );

    let manifest = read_to_string(manifest_path).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_path, "lib");
}

/// Test that you can't create a library and binary package at the same time
#[test]
fn test_create_lib_and_bin_fails() {
    let package_name = "mynewcrate-lib-and-bin-2018";
    let dir = "../fixtures/normal";
    let package_path = Path::new(dir).join(package_name);

    clean_package_dir(&package_path, "lib");
    clean_package_dir(&package_path, "bin");

    let err = utils::run_err(
        dir,
        &[
            "ws",
            "create",
            package_name,
            "--edition",
            "2018",
            "--lib",
            "--bin",
            "--name",
            package_name,
        ],
    );

    assert!(err.contains("error"));
    assert!(err.contains("--bin"));
    assert!(err.contains("cannot be used with"));
    assert!(err.contains("--lib"));

    let exists = package_path.exists();
    assert!(!exists);

    clean_package_dir(&package_path, "lib");
    clean_package_dir(&package_path, "bin");
}
