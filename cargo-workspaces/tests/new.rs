mod utils;
use serial_test::serial;
use std::{
    fs::{read_to_string, remove_dir_all},
    path::Path,
};

/// Clean up a test created directory
/// This removes the entire directory and its contents
/// dir is the directory to remove
fn clean_test_dir(dir: &Path) {
    assert_ne!(dir.as_os_str(), "");
    // On Windows, the root drive directory would have a length of 3 (e.g., "C:\\")
    if dir.as_os_str().len() <= 3 {
        return;
    }

    if dir.exists() {
        let _ = remove_dir_all(dir); // Ignore errors when removing
    }
}

#[test]
#[serial]
fn test_new_basic() {
    let dir = "../fixtures/test_new_basic";
    let test_path = Path::new(dir);

    clean_test_dir(test_path);

    // Run the new command
    let err = utils::run_err(".", &["ws", "new", dir]);

    // Verify the command output
    assert!(err.contains("info initialized"));

    // Verify that directory was created with proper content
    let gitignore_path = test_path.join(".gitignore");
    let cargo_toml_path = test_path.join("Cargo.toml");

    assert!(gitignore_path.exists());
    assert!(cargo_toml_path.exists());

    // Check .gitignore content
    let gitignore_content = read_to_string(gitignore_path).unwrap();
    assert_eq!(gitignore_content.trim(), "/target");

    // Check Cargo.toml content
    let cargo_toml_content = read_to_string(cargo_toml_path).unwrap();
    assert!(cargo_toml_content.contains("[workspace]"));

    clean_test_dir(test_path);
}

#[test]
#[serial]
fn test_new_with_resolver() {
    let dir = "../fixtures/test_new_resolver";
    let test_path = Path::new(dir);

    clean_test_dir(test_path);

    // Run the new command with resolver option
    let err = utils::run_err(".", &["ws", "new", "--resolver", "2", dir]);

    // Verify the command output
    assert!(err.contains("info initialized"));

    // Verify that directory was created with proper content
    let gitignore_path = test_path.join(".gitignore");
    let cargo_toml_path = test_path.join("Cargo.toml");

    assert!(gitignore_path.exists());
    assert!(cargo_toml_path.exists());

    // Check .gitignore content
    let gitignore_content = read_to_string(gitignore_path).unwrap();
    assert_eq!(gitignore_content.trim(), "/target");

    // Check Cargo.toml content includes resolver
    let cargo_toml_content = read_to_string(cargo_toml_path).unwrap();
    assert!(cargo_toml_content.contains("[workspace]"));
    assert!(cargo_toml_content.contains("resolver = \"2\""));

    clean_test_dir(test_path);
}

#[test]
#[serial]
fn test_new_path_already_exists() {
    let dir = "../fixtures/test_exists";
    let test_path = Path::new(dir);

    clean_test_dir(test_path);
    std::fs::create_dir_all(test_path).unwrap();

    // Run the new command on existing path - this should succeed since init handles existing directories
    let err = utils::run_err(".", &["ws", "new", dir]);

    // The command should succeed if the workspace is already initialized
    if err.contains("info already initialized") || err.contains("info initialized") {
        // Both are acceptable outcomes
    } else {
        panic!("Unexpected error: {}", err);
    }

    clean_test_dir(test_path);
}
