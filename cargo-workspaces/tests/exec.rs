mod utils;
#[cfg(not(windows))]
use insta::assert_snapshot;

// #[cfg(windows)]
// static PRINT: &str = "cd";

#[cfg(not(windows))]
static PRINT: &str = "cat";

// TODO: Get exec test working on windows
#[cfg(not(windows))]
#[test]
fn test_normal() {
    let (out, err) = utils::run("../fixtures/normal", &["ws", "exec", PRINT, "Cargo.toml"]);
    assert_snapshot!(err);
    assert_snapshot!(out);
}
