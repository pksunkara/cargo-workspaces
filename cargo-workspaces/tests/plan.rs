mod utils;
use insta::assert_snapshot;

#[test]
fn test_plan_single() {
    // `err` may contain a warning about missing `http.cainfo` config value.
    let (out, _err) = utils::run("../fixtures/single", &["ws", "plan"]);
    assert_snapshot!(out);
}

#[test]
fn test_plan_normal() {
    // `err` may contain a warning about missing `http.cainfo` config value.
    let (out, _err) = utils::run("../fixtures/normal", &["ws", "plan"]);
    assert_snapshot!(out);
}

#[test]
fn test_plan_normal_long() {
    // `err` may contain a warning about missing `http.cainfo` config value.
    let (out, _err) = utils::run("../fixtures/normal", &["ws", "plan", "--long"]);
    assert_snapshot!(out);
}
