# testdata-rs

Macros and helper functions for file-based testing.

## Example

The crate's main feature is [`testdata::files`], which automatically
finds test files and expands to multiple tests.

```rust
use std::str;
use testdata::{assert_snapshot, TestFile};

#[testdata::files(rebuild = "tests/example.rs")]
#[test]
fn test_foo(
    #[glob = "tests/fixtures/**/*-in.txt"] input: &TestFile,
    #[glob = "tests/fixtures/**/*-out.txt"] output: &TestFile,
) {
    let s = input.raw_read();
    let s = str::from_utf8(&s).unwrap();
    let result = s.to_uppercase();
    assert_snapshot!(result, snapshot = output);
}
```

More documents will be added in the later versions.
