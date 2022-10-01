//! Runtime definitions for the `testdata` crate.

#![cfg_attr(all(feature = "__doc_cfg", doc), feature(doc_cfg))]

mod formats;
mod globbing;
mod patterns;
mod snapshots;
mod test_files;
mod test_input;

#[cfg(any(feature = "serde_json", all(feature = "__doc_cfg", doc)))]
pub use crate::formats::json::Json;
pub use crate::globbing::{ArgSpec, GlobError, GlobSpec};
pub use crate::patterns::{GlobParseError, GlobPattern};
pub use crate::snapshots::{assert_snapshot_helper, Snapshot, SnapshotMode};
pub use crate::test_files::{pending, TestFile};
pub use crate::test_input::TestInput;
#[doc(hidden)]
pub extern crate pretty_assertions;
