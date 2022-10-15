use testdata_rt::GlobSpec;

use crate::test_files::TestFile;

pub trait GlobSpecExt {
    /// Assigns a specific test name to get the path(s) to the file.
    fn expand(&self, stem: &str) -> Option<Vec<TestFile>>;
}

impl GlobSpecExt for GlobSpec {
    fn expand(&self, stem: &str) -> Option<Vec<TestFile>> {
        let test_files = self.expand_core(stem)?;
        let test_files = test_files
            .into_iter()
            .map(|paths| TestFile { paths })
            .collect::<Vec<_>>();
        if test_files.iter().any(|f| f.exists()) {
            Some(test_files)
        } else {
            None
        }
    }
}
