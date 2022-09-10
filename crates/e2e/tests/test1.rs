#[testdata::testdata]
#[test]
fn test_foo(#[glob = "tests/fixtures/**/*-in.txt"] input: &std::path::PathBuf) {
    //
}
