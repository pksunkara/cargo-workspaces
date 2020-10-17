mod utils;
#[cfg(unix)]
use insta::assert_snapshot;

#[cfg(unix)]
#[test]
fn test_normal() {
    let (out, err) = utils::run("../fixtures/normal", &["ws", "exec", "cat", "Cargo.toml"]);
    assert_snapshot!(out);
    assert_snapshot!(err);
}
