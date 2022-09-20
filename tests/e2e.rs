use std::process::Command;

#[test]
fn test_e2e() {
    let cmd = Command::new("cargo")
        .args(&["test", "-p", "e2e", "--features", "e2e"])
        .output()
        .unwrap();
    assert_eq!(cmd.status.code(), Some(101));
    let stdout = String::from_utf8_lossy(&cmd.stdout);
    assert!(stdout.contains("test test_foo::foo ... ok"));
    assert!(stdout.contains("test test_foo::bar ... FAILED"));
    assert!(stdout.contains("test test_foo::bar2 ... ok"));
    assert!(stdout.contains("test test_foo::__others ... ok"));
}
