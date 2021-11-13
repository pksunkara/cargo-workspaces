mod utils;
use insta::assert_snapshot;
use std::fs::{read_to_string, remove_dir, remove_file};
use std::path::Path;

/// Clean up a cargo or cargo-workspace created package directory
/// This doesn't use remove_dir_all, so it's safer, but more liable to break
/// if cargo new is updated in the future
/// package_dir is the directory containing the package
/// package_type is bin or lib
fn clean_package_dir(package_dir: &String, package_type: &str) {
    assert_ne!(package_dir, "");
    assert_ne!(package_dir, "/");

    let package_path = Path::new(package_dir);
    let exists = package_path.exists();
    if !exists {
        return;
    }

    let cargo_fn = String::from(format!("{}/Cargo.toml", package_dir));
    let cargo_path = Path::new(&cargo_fn);
    let exists = cargo_path.exists();
    if exists {
        remove_file(cargo_fn).unwrap();
    }

    let src_fn =
        match package_type {
            "bin" => {
                String::from(format!("{}/src/main.rs", package_dir))
            },
            "lib" => {
                String::from(format!("{}/src/lib.rs", package_dir))
            },
            _ => {
                return;
            },
        };

    let src_path = Path::new(&src_fn);
    let exists = src_path.exists();
    if exists {
        remove_file(src_fn).unwrap();
    }

    let src_fn = String::from(format!("{}/src", package_dir));
    let src_path = Path::new(&src_fn);
    let exists = src_path.exists();
    if exists {
        remove_dir(src_fn).unwrap();
    }

    let package_path = Path::new(package_dir);
    let exists = package_path.exists();
    if exists {
        remove_dir(package_dir).unwrap();
    }
}

/// Test creating a 2015 bin package
#[test]
fn test_create_bin_2015() {
    let package_name = "mynewcrate-bin-2015";
    let dir = "../fixtures/normal";
    let package_dir = format!("{}/{}", dir, package_name);

    clean_package_dir(&package_dir, "bin");

    let _err = utils::run_err(dir,
                              &["ws", "create", package_name,
                                "--edition", "2015", "--bin",
                                "--name", package_name]);

    let manifest = read_to_string(format!("{}/Cargo.toml", package_dir)).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_dir, "bin");
}

/// Test creating a 2015 lib package
#[test]
fn test_create_lib_2015() {
    let package_name = "mynewcrate-lib-2015";
    let dir = "../fixtures/normal";
    let package_dir = format!("{}/{}", dir, package_name);

    clean_package_dir(&package_dir, "lib");

    let _err = utils::run_err(dir,
                              &["ws", "create", package_name,
                                "--edition", "2015", "--lib",
                                "--name", package_name]);

    let manifest = read_to_string(format!("{}/Cargo.toml", package_dir)).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_dir, "lib");
}

/// Test creating a 2018 bin package
#[test]
fn test_create_bin_2018() {
    let package_name = "mynewcrate-bin-2018";
    let dir = "../fixtures/normal";
    let package_dir = format!("{}/{}", dir, package_name);

    clean_package_dir(&package_dir, "bin");

    let _err = utils::run_err(dir,
                              &["ws", "create", package_name,
                                "--edition", "2018", "--bin",
                                "--name", package_name]);

    let manifest = read_to_string(format!("{}/Cargo.toml", package_dir)).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_dir, "bin");
}

/// Test creating a 2018 lib package
#[test]
fn test_create_lib_2018() {
    let package_name = "mynewcrate-lib-2018";
    let dir = "../fixtures/normal";
    let package_dir = format!("{}/{}", dir, package_name);

    clean_package_dir(&package_dir, "lib");

    let _err = utils::run_err(dir,
                              &["ws", "create", package_name,
                                "--edition", "2018", "--lib",
                                "--name", package_name]);

    let manifest = read_to_string(format!("{}/Cargo.toml", package_dir)).unwrap();

    assert_snapshot!(&manifest);

    clean_package_dir(&package_dir, "lib");
}

/// Test that you can't create a library and binary package at the same time
#[test]
fn test_create_lib_and_bin_fails() {
    let package_name = "mynewcrate-lib-and-bin-2018";
    let dir = "../fixtures/normal";
    let package_dir = format!("{}/{}", dir, package_name);

    clean_package_dir(&package_dir, "lib");
    clean_package_dir(&package_dir, "bin");

    let err = utils::run_err(dir,
                             &["ws", "create", package_name,
                               "--edition", "2018", "--lib", "--bin",
                               "--name", package_name]);

    assert!(err.contains("error"));
    assert!(err.contains("--bin"));
    assert!(err.contains("cannot be used with"));
    assert!(err.contains("--lib"));

    let package_path = Path::new(&package_dir);
    let exists = package_path.exists();
    assert!(!exists);

    clean_package_dir(&package_dir, "lib");
    clean_package_dir(&package_dir, "bin");
}
