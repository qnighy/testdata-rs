use std::str;
use testdata::{assert_snapshot, TestFile};

#[testdata::files(rebuild = "tests/example.rs")]
#[test]
fn test_foo(
    #[glob = "tests/fixtures/project2/**/*-in.txt"] input: &TestFile,
    #[glob = "tests/fixtures/project2/**/*-out.txt"] output: &TestFile,
) {
    let s = input.raw_read();
    let s = str::from_utf8(&s).unwrap();
    let result = s.to_uppercase();
    assert_snapshot!(result, snapshot = output);
}
