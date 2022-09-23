pub use testdata_macros::files;
pub use testdata_rt::*;

pub mod __rt {
    pub use once_cell::sync::Lazy;
    pub use testdata_rt::{touch, ArgSpec, GlobSpec};
}
