use std::path::Path;

use testdata_glob::{ArgSpec, Error, GlobSpec};

#[test]
fn test_walk_dir() {
    let spec = GlobSpec::new()
        .arg(ArgSpec::new("data/**/*-in.txt"))
        .arg(ArgSpec::new("data/**/*-out.txt"));
    let stems = spec.glob_dir(Path::new("tests/fixtures/project1")).unwrap();
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
        .arg(ArgSpec::new("data/*-in.txt"))
        .arg(ArgSpec::new("data/*-out.txt"));
    let stems = spec.glob_dir(Path::new("tests/fixtures/project1")).unwrap();
    assert_eq!(
        stems,
        vec!["bar".to_owned(), "baz".to_owned(), "foo".to_owned(),]
    );
}

#[test]
fn test_walk_dir_invalid_glob1() {
    let spec = GlobSpec::new().arg(ArgSpec::new("data/*-*.txt"));
    let e = spec
        .glob_dir(Path::new("tests/fixtures/project1"))
        .unwrap_err();
    assert_eq!(e.to_string(), "Invalid glob: \"data/*-*.txt\"");
}

#[test]
fn test_walk_dir_invalid_glob2() {
    let spec = GlobSpec::new().arg(ArgSpec::new("data/in.txt"));
    let e = spec
        .glob_dir(Path::new("tests/fixtures/project1"))
        .unwrap_err();
    assert_eq!(e.to_string(), "Invalid glob: \"data/in.txt\"");
}

#[test]
fn test_walk_dir_mixed_glob() {
    let spec = GlobSpec::new()
        .arg(ArgSpec::new("data/**/*-in.txt"))
        .arg(ArgSpec::new("data/*-out.txt"));
    let e = spec
        .glob_dir(Path::new("tests/fixtures/project1"))
        .unwrap_err();
    assert_eq!(e.to_string(), "Different glob types are mixed");
}

#[test]
fn test_walk_dir_no_args() {
    let spec = GlobSpec::new();
    let stems = spec.glob_dir(Path::new("tests/fixtures/project1")).unwrap();
    assert_eq!(stems, vec![] as Vec<String>);
}

#[test]
fn test_error_source() {
    use std::error::Error as StdError;
    let _ = Error::InvalidGlob("".into()).source();
}

#[test]
fn test_error_debug() {
    let _ = format!("{:?}", Error::InvalidGlob("".into()));
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
