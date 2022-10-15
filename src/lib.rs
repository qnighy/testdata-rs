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

mod formats;
mod glob_ext;
mod snapshots;
mod test_files;
mod test_input;
pub mod util;

#[cfg(any(feature = "serde_json", all(feature = "__doc_cfg", doc)))]
pub use crate::formats::json::Json;
pub use crate::glob_ext::GlobSpecExt;
pub use crate::snapshots::{assert_snapshot_helper, Snapshot, SnapshotMode};
pub use crate::test_files::{pending, TestFile};
pub use crate::test_input::TestInput;
pub use testdata_macros::files;
pub use testdata_rt::*;

pub mod __rt {
    pub use once_cell::sync::Lazy;
    pub use testdata_rt::{ArgSpec, GlobSpec};

    pub use crate::util::{diff, touch};
    pub use crate::GlobSpecExt;
}
