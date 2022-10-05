use std::path::Path;

use testdata_rt::{ArgSpec, GlobError, GlobSpec};

#[test]
fn test_walk_dir() {
    let spec = GlobSpec::new()
        .root(Path::new("tests/fixtures/project1"))
        .arg(ArgSpec::new("data/**/*-in.txt"))
        .arg(ArgSpec::new("data/**/*-out.txt"));
    let stems = spec.glob().unwrap();
    assert_eq!(
        stems,
        vec![
            "bar".to_owned(),
            "baz".to_owned(),
            "foo".to_owned(),
            "nested/bar".to_owned(),
            "nested/baz".to_owned(),
            "nested/foo".to_owned()
        ]
    );
}

#[test]
fn test_walk_dir2() {
    let spec = GlobSpec::new()
        .arg(ArgSpec::new("tests/fixtures/project1/data/**/*-in.txt"))
        .arg(ArgSpec::new("tests/fixtures/project1/data/**/*-out.txt"));
    let stems = spec.glob().unwrap();
    assert_eq!(
        stems,
        vec![
            "bar".to_owned(),
            "baz".to_owned(),
            "foo".to_owned(),
            "nested/bar".to_owned(),
            "nested/baz".to_owned(),
            "nested/foo".to_owned()
        ]
    );
}

#[test]
fn test_walk_dir_non_nested() {
    let spec = GlobSpec::new()
        .root(Path::new("tests/fixtures/project1"))
        .arg(ArgSpec::new("data/*-in.txt"))
        .arg(ArgSpec::new("data/*-out.txt"));
    let stems = spec.glob().unwrap();
    assert_eq!(
        stems,
        vec!["bar".to_owned(), "baz".to_owned(), "foo".to_owned(),]
    );
}

#[test]
fn test_walk_dir_no_args() {
    let spec = GlobSpec::new().root(Path::new("tests/fixtures/project1"));
    let stems = spec.glob().unwrap();
    assert_eq!(stems, vec![] as Vec<String>);
}

#[test]
fn test_error_source() {
    use std::error::Error as StdError;
    let _ = GlobError::InvalidPath("".into()).source();
}

#[test]
fn test_error_debug() {
    let _ = format!("{:?}", GlobError::InvalidPath("".into()));
}

#[test]
fn test_glob_spec_clone() {
    let glob_spec = GlobSpec::new();
    let _ = glob_spec.clone();
}

#[test]
fn test_glob_spec_debug() {
    let glob_spec = GlobSpec::new();
    let _ = format!("{:?}", glob_spec);
}

#[test]
fn test_arg_spec_clone() {
    let arg_spec = ArgSpec::new("src/**/*.txt");
    let _ = arg_spec.clone();
}

#[test]
fn test_arg_spec_debug() {
    let arg_spec = ArgSpec::new("src/**/*.txt");
    let _ = format!("{:?}", arg_spec);
}
