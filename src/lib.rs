pub use testdata_macros::testdata;

pub mod __rt {
    pub use once_cell::sync::Lazy;
    pub use testdata_glob::{ArgSpec, GlobSpec};
}
