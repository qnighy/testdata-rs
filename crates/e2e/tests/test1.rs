#[testdata::testdata(rebuild = "tests/test1.rs")]
#[test]
fn test_foo(#[glob = "tests/fixtures/**/*-in.txt"] input: &std::path::PathBuf) {
    //
}