//! Runtime definitions for the `testdata` crate.

#![cfg_attr(all(feature = "__doc_cfg", doc), feature(doc_cfg))]

mod globbing;
mod patterns;

pub use crate::globbing::{ArgSpec, GlobError, GlobSpec};
pub use crate::patterns::{GlobParseError, GlobPattern};
#[doc(hidden)]
pub extern crate pretty_assertions;
