mod utils;
use insta::assert_snapshot;

#[test]
fn test_dry_run_single() {
    // `--allow-dirty` is supplied to not break tests during development.
    let mut err = utils::run_err(
        "../fixtures/single",
        &["ws", "publish", "--dry-run", "--allow-dirty"],
    );
    utils::normalize_output(&mut err);
    assert_snapshot!(err);
}

#[test]
fn test_dry_run_normal() {
    // `--allow-dirty` is supplied to not break tests during development.
    let mut err = utils::run_err(
        "../fixtures/normal",
        &["ws", "publish", "--dry-run", "--allow-dirty"],
    );
    utils::normalize_output(&mut err);
    assert_snapshot!(err);
}

#[test]
fn test_dry_run_normal_missing_fields() {
    // `--allow-dirty` is supplied to not break tests during development.
    let mut err = utils::run_err(
        "../fixtures/normal_missing_fields",
        &["ws", "publish", "--dry-run", "--allow-dirty"],
    );
    utils::normalize_output(&mut err);
    assert_snapshot!(err);
}
