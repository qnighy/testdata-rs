use std::path::{Path, PathBuf};

use testdata_walk::{ArgSpec, GlobSpec};

#[test]
fn test_expand() {
    let spec = GlobSpec::new()
        .arg(ArgSpec::new("data/**/*-in.txt"))
        .arg(ArgSpec::new("data/**/*-out.txt"));
    let root = Path::new("tests/fixtures/project1");
    assert_eq!(
        spec.expand(root, "bar"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/bar-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/bar-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "baz"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/baz-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/baz-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "foo"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/foo-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/foo-out.txt"),
        ])
    );
    assert_eq!(spec.expand(root, "fooo"), None);
    assert_eq!(
        spec.expand(root, "nested/bar"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/nested/bar-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/nested/bar-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "nested/baz"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/nested/baz-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/nested/baz-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "nested/foo"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/nested/foo-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/nested/foo-out.txt"),
        ])
    );
    assert_eq!(spec.expand(root, "nested/fooo"), None);
}

#[test]
fn test_expand_non_nested() {
    let spec = GlobSpec::new()
        .arg(ArgSpec::new("data/*-in.txt"))
        .arg(ArgSpec::new("data/*-out.txt"));
    let root = Path::new("tests/fixtures/project1");
    assert_eq!(
        spec.expand(root, "bar"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/bar-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/bar-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "baz"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/baz-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/baz-out.txt"),
        ])
    );
    assert_eq!(
        spec.expand(root, "foo"),
        Some(vec![
            PathBuf::from("tests/fixtures/project1/data/foo-in.txt"),
            PathBuf::from("tests/fixtures/project1/data/foo-out.txt"),
        ])
    );
    assert_eq!(spec.expand(root, "fooo"), None);
    assert_eq!(spec.expand(root, "nested/bar"), None);
    assert_eq!(spec.expand(root, "nested/baz"), None);
    assert_eq!(spec.expand(root, "nested/foo"), None);
    assert_eq!(spec.expand(root, "nested/fooo"), None);
}
