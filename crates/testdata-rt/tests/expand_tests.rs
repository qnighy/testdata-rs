use std::path::{Path, PathBuf};

use testdata_rt::{ArgSpec, Fixture, GlobSpec};

#[test]
fn test_expand() {
    let spec = GlobSpec::new()
        .root(Path::new("tests/fixtures/project1"))
        .arg(ArgSpec::new("data/**/*-in.txt"))
        .arg(ArgSpec::new("data/**/*-out.txt"));
    assert_eq!(
        spec.expand("bar"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/bar-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/bar-out.txt")],
            },
        ])
    );
    assert_eq!(
        spec.expand("baz"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/baz-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/baz-out.txt")],
            },
        ])
    );
    assert_eq!(
        spec.expand("foo"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/foo-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/foo-out.txt")],
            },
        ])
    );
    assert_eq!(spec.expand("fooo"), None);
    assert_eq!(
        spec.expand("nested/bar"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/bar-in.txt"
                )],
            },
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/bar-out.txt"
                )],
            },
        ])
    );
    assert_eq!(
        spec.expand("nested/baz"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/baz-in.txt"
                )],
            },
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/baz-out.txt"
                )],
            },
        ])
    );
    assert_eq!(
        spec.expand("nested/foo"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/foo-in.txt"
                )],
            },
            Fixture {
                paths: vec![PathBuf::from(
                    "tests/fixtures/project1/data/nested/foo-out.txt"
                )],
            },
        ])
    );
    if cfg!(windows) {
        assert_eq!(
            spec.expand("nested/foo").unwrap()[0].paths[0]
                .to_str()
                .unwrap(),
            "tests/fixtures/project1\\data\\nested\\foo-in.txt",
        );
    } else {
        assert_eq!(
            spec.expand("nested/foo").unwrap()[0].paths[0]
                .to_str()
                .unwrap(),
            "tests/fixtures/project1/data/nested/foo-in.txt",
        );
    }
    assert_eq!(spec.expand("nested/fooo"), None);
}

#[test]
fn test_expand_non_nested() {
    let spec = GlobSpec::new()
        .root(Path::new("tests/fixtures/project1"))
        .arg(ArgSpec::new("data/*-in.txt"))
        .arg(ArgSpec::new("data/*-out.txt"));
    assert_eq!(
        spec.expand("bar"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/bar-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/bar-out.txt")],
            },
        ])
    );
    assert_eq!(
        spec.expand("baz"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/baz-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/baz-out.txt")],
            },
        ])
    );
    assert_eq!(
        spec.expand("foo"),
        Some(vec![
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/foo-in.txt")],
            },
            Fixture {
                paths: vec![PathBuf::from("tests/fixtures/project1/data/foo-out.txt")],
            },
        ])
    );
    assert_eq!(spec.expand("fooo"), None);
    assert_eq!(spec.expand("nested/bar"), None);
    assert_eq!(spec.expand("nested/baz"), None);
    assert_eq!(spec.expand("nested/foo"), None);
    assert_eq!(spec.expand("nested/fooo"), None);
}
