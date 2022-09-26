use std::collections::HashSet;
use std::path::{Path, PathBuf, StripPrefixError};

use path_slash::PathBufExt as _;
use path_slash::PathExt as _;
use thiserror::Error as StdError;
use walkdir::WalkDir;

use crate::patterns::{GlobParseError, GlobPattern};
use crate::test_files::TestFile;

/// Represents the glob error.
#[derive(Debug, StdError)]
pub enum GlobError {
    #[error("Error during walk: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Cannot compute relative path from {} to {}", .1.display(), .2.display())]
    StripPrefix(#[source] StripPrefixError, PathBuf, PathBuf),
    #[error("Got a non-utf8 path: {0:?}")]
    InvalidPath(PathBuf),
}

/// Configurations for finding test files in a file-based test.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GlobSpec {
    /// Serching root. Defaults to `.`.
    pub root: PathBuf,
    /// List of arguments in the order of appearence.
    pub args: Vec<ArgSpec>,
}

impl GlobSpec {
    /// Creates the default glob configuration.
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
            args: Vec::new(),
        }
    }

    /// Builder utility to set `self.root`.
    pub fn root(mut self, root: &Path) -> Self {
        self.root = root.to_owned();
        self
    }

    /// Builder utility to set `self.args`.
    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    /// Searches for the test files.
    pub fn glob(&self) -> Result<Vec<String>, GlobError> {
        self.glob_from(Path::new(""))
    }
    /// Searches for the test files, with custom working directory.
    pub fn glob_from(&self, cwd: &Path) -> Result<Vec<String>, GlobError> {
        let root = cwd.join(&self.root);
        let mut stems = HashSet::new();
        for prefix in &self.prefixes() {
            let walk_root = root.join(PathBuf::from_slash(prefix));
            for entry in WalkDir::new(&walk_root).sort_by_file_name() {
                let entry = entry?;
                let file_name = entry.path().strip_prefix(&root).map_err(|e| {
                    GlobError::StripPrefix(e, root.clone(), entry.path().to_owned())
                })?;
                let file_name = file_name
                    .to_slash()
                    .ok_or_else(|| GlobError::InvalidPath(entry.path().to_owned()))?;
                for arg in &self.args {
                    for stem in arg.glob.do_match(&file_name) {
                        stems.insert(stem.to_owned());
                    }
                }
            }
        }
        let sorted_stems = {
            let mut sorted_stems = stems.into_iter().collect::<Vec<_>>();
            sorted_stems.sort();
            sorted_stems
        };

        Ok(sorted_stems)
    }

    /// Helper function that does `GlobSpec::glob` and set differene.
    pub fn glob_diff(
        &self,
        known_stems: &[String],
    ) -> Result<(Vec<String>, Vec<String>), GlobError> {
        let stems = self.glob()?;
        let missing_stems = {
            let stems = stems.iter().collect::<HashSet<_>>();
            known_stems
                .iter()
                .cloned()
                .filter(|stem| !stems.contains(stem))
                .collect::<Vec<_>>()
        };
        let extra_stems = {
            let known_stems = known_stems.iter().collect::<HashSet<_>>();
            stems
                .iter()
                .cloned()
                .filter(|stem| !known_stems.contains(stem))
                .collect::<Vec<_>>()
        };
        Ok((extra_stems, missing_stems))
    }

    /// Assigns a specific test name to get the path(s) to the file.
    pub fn expand(&self, stem: &str) -> Option<Vec<TestFile>> {
        let mut test_files = Vec::new();
        for arg in &self.args {
            let paths = arg
                .glob
                .subst(stem)
                .iter()
                .map(|stem| self.root.join(PathBuf::from_slash(stem)))
                .collect::<Vec<_>>();
            if paths.is_empty() {
                return None;
            }
            test_files.push(TestFile { paths });
        }
        if test_files.iter().any(|f| f.exists()) {
            Some(test_files)
        } else {
            None
        }
    }

    fn prefixes(&self) -> Vec<String> {
        let mut prefixes = Vec::new();
        for arg in &self.args {
            prefixes.extend_from_slice(&arg.glob.prefixes());
        }
        for prefix in &mut prefixes {
            let pos = prefix.rfind('/').unwrap_or(0);
            prefix.truncate(pos);
            prefix.push('/');
        }
        prefixes.sort();
        let mut last = 0;
        for i in 1..prefixes.len() {
            if prefixes[i].starts_with(&prefixes[last]) {
                prefixes[i].clear();
            } else {
                last = i;
            }
        }
        // prefixes.drain_filter(|elem| elem.is_empty());
        prefixes = prefixes
            .into_iter()
            .filter(|elem| !elem.is_empty())
            .collect::<Vec<_>>();
        for p in &mut prefixes {
            p.pop();
        }
        prefixes
    }
}

/// Configuration for a specific argument in a file-based test.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ArgSpec {
    pub glob: GlobPattern,
}

impl ArgSpec {
    pub fn new(glob: &str) -> Self {
        Self::parse(glob).unwrap()
    }

    pub fn parse(glob: &str) -> Result<Self, GlobParseError> {
        Ok(Self {
            glob: glob.parse()?,
        })
    }
}
