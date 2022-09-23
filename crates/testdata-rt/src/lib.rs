//! Runtime definitions for the `testdata` crate.

#![cfg_attr(all(feature = "__doc_cfg", doc), feature(doc_cfg))]

mod fixtures;
mod formats;
mod globbing;
mod patterns;
mod snapshots;
mod test_input;

use std::io;
use std::path::Path;

pub use crate::fixtures::{pending, Fixture};
#[cfg(any(feature = "serde_json", all(feature = "__doc_cfg", doc)))]
pub use crate::formats::json::Json;
pub use crate::globbing::{ArgSpec, Error, GlobSpec};
pub use crate::patterns::{GlobParseError, GlobPattern};
pub use crate::snapshots::{assert_snapshot_helper, Snapshot, SnapshotMode};
pub use crate::test_input::TestInput;

/// An equivalent to the `touch` command.
pub fn touch(path: &Path) -> io::Result<()> {
    // Touch the file containing the test
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utime::set_file_times(path, now as i64, now as i64)?;
    Ok(())
}
