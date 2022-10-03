use testdata::util::{diff, DiffResult};

use big_s::S;

#[test]
fn test_diff() {
    let testcases = vec![
        (
            vec![],
            vec![],
            DiffResult {
                has_diff: false,
                extra: vec![],
                missing: vec![],
            },
        ),
        (
            vec![S("foo")],
            vec![],
            DiffResult {
                has_diff: true,
                extra: vec![S("foo")],
                missing: vec![],
            },
        ),
        (
            vec![],
            vec![S("bar")],
            DiffResult {
                has_diff: true,
                extra: vec![],
                missing: vec![S("bar")],
            },
        ),
        (
            vec![S("bar"), S("foo")],
            vec![S("baz"), S("foo")],
            DiffResult {
                has_diff: true,
                extra: vec![S("bar")],
                missing: vec![S("baz")],
            },
        ),
    ];
    for testcase in &testcases {
        assert_eq!(
            diff(&testcase.0, &testcase.1),
            testcase.2,
            "diff({:?}, {:?})",
            testcase.0,
            testcase.1,
        );
    }
}
