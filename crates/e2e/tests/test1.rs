use testdata::{pending, Fixture};

#[testdata::testdata(rebuild = "tests/test1.rs")]
#[test]
fn test_foo(
    #[glob = "tests/fixtures/**/*-in.txt"] input: &Fixture,
    #[glob = "tests/fixtures/**/*-pending.txt"] pending_file: &Fixture,
) {
    if !cfg!(feature = "e2e") {
        return;
    }
    pending(pending_file, || {
        let text = input.raw_read();
        let text = String::from_utf8_lossy(&text).into_owned();
        assert_eq!(text, "ok\n");
    });
}
