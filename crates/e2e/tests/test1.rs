#![cfg(feature = "e2e")]

use std::fs;
use std::path::Path;

#[testdata::testdata(rebuild = "crates/e2e/tests/test1.rs")]
#[test]
fn test_foo(#[glob = "tests/fixtures/**/*-in.txt"] input: &Path) {
    let text = fs::read(input).unwrap();
    let text = String::from_utf8_lossy(&text).into_owned();
    assert_eq!(text, "ok\n");
}
