mod utils;
use insta::assert_snapshot;

#[test]
fn test_single() {
    let out = utils::run_out("../fixtures/single", &["ws", "ls"]);
    assert_snapshot!(out);
}

#[test]
fn test_long() {
    let out = utils::run_out("../fixtures/single", &["ws", "ll"]);
    assert_snapshot!(out);
}

#[test]
fn test_long_root() {
    let out = utils::run_out("../fixtures/root", &["ws", "ll"]);
    assert_snapshot!(out);
}

#[test]
fn test_all() {
    let out = utils::run_out("../fixtures/private", &["ws", "la"]);
    assert_snapshot!(out);
}

#[test]
fn test_long_all() {
    let out = utils::run_out("../fixtures/private", &["ws", "list", "--long", "--all"]);
    assert_snapshot!(out);
}

#[test]
fn test_json() {
    let out = utils::run_out("../fixtures/private", &["ws", "list", "--json"]);

    assert!(out.contains(r#""name": "simple""#));
    assert!(out.contains(r#""version": "0.1.0-rc.0""#));
    assert!(out.contains(r#""private": false"#));

    assert!(!out.contains(r#""name": "private""#));
    assert!(!out.contains(r#""version": "0.2.0""#));
    assert!(!out.contains(r#""private": true"#));
}

#[test]
fn test_json_all() {
    let out = utils::run_out("../fixtures/private", &["ws", "list", "--json", "--all"]);

    assert!(out.contains(r#""name": "simple""#));
    assert!(out.contains(r#""version": "0.1.0-rc.0""#));
    assert!(out.contains(r#""private": false"#));

    assert!(out.contains(r#""name": "private""#));
    assert!(out.contains(r#""version": "0.2.0""#));
    assert!(out.contains(r#""private": true"#));
}

#[test]
fn test_json_conflicts_with_long() {
    let err = utils::run_err("../fixtures/private", &["ws", "list", "--long", "--json"]);
    assert_snapshot!(err);
}
