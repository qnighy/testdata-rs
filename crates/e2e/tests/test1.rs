#![cfg(feature = "e2e")]

use testdata::Fixture;

#[testdata::testdata(rebuild = "crates/e2e/tests/test1.rs")]
#[test]
fn test_foo(#[glob = "tests/fixtures/**/*-in.txt"] input: &Fixture) {
    let text = input.raw_read();
    let text = String::from_utf8_lossy(&text).into_owned();
    assert_eq!(text, "ok\n");
}
