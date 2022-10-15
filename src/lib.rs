//! Macros and helper functions for file-based testing.
//!
//! ## Example
//!
//! The crate's main feature is [`testdata::files`], which automatically
//! finds test files and expands to multiple tests.
//!
//! ```rust
//! use std::str;
//! use testdata::{assert_snapshot, TestFile};
//!
//! #[testdata::files(rebuild = "tests/example.rs")]
//! #[test]
//! fn test_foo(
//!     #[glob = "tests/fixtures/**/*-in.txt"] input: &TestFile,
//!     #[glob = "tests/fixtures/**/*-out.txt"] output: &TestFile,
//! ) {
//!     let s = input.raw_read();
//!     let s = str::from_utf8(&s).unwrap();
//!     let result = s.to_uppercase();
//!     assert_snapshot!(result, snapshot = output);
//! }
//! ```
//!
//! More documents will be added in the later versions.

pub mod util;

pub use testdata_macros::files;
pub use testdata_rt::*;

pub mod __rt {
    pub use once_cell::sync::Lazy;
    pub use testdata_rt::{ArgSpec, GlobSpec, GlobSpecExt};

    pub use crate::util::{diff, touch};
}
