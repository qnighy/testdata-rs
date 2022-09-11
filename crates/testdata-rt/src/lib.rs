mod patterns;

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf, StripPrefixError};

use thiserror::Error as StdError;
use walkdir::WalkDir;

pub use crate::patterns::{GlobParseError, GlobPattern};

#[derive(Debug, StdError)]
pub enum Error {
    #[error("Error during walk: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Cannot compute relative path from {} to {}", .1.display(), .2.display())]
    StripPrefix(#[source] StripPrefixError, PathBuf, PathBuf),
    #[error("Got a non-utf8 path: {0:?}")]
    InvalidPath(PathBuf),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GlobSpec {
    pub root: PathBuf,
    pub args: Vec<ArgSpec>,
}

impl GlobSpec {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
            args: Vec::new(),
        }
    }

    pub fn root(mut self, root: &Path) -> Self {
        self.root = root.to_owned();
        self
    }

    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    pub fn glob(&self) -> Result<Vec<String>, Error> {
        let mut stems = HashSet::new();
        for entry in WalkDir::new(&self.root).sort_by_file_name() {
            let entry = entry?;
            let file_name = entry
                .path()
                .strip_prefix(&self.root)
                .map_err(|e| Error::StripPrefix(e, self.root.clone(), entry.path().to_owned()))?;
            let file_name = file_name
                .to_str()
                .ok_or_else(|| Error::InvalidPath(entry.path().to_owned()))?;
            for arg in &self.args {
                if let Some(stem) = arg.glob.do_match(file_name) {
                    stems.insert(stem.to_owned());
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

    pub fn glob_diff(&self, known_stems: &[String]) -> Result<(Vec<String>, Vec<String>), Error> {
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

    pub fn expand(&self, stem: &str) -> Option<Vec<PathBuf>> {
        let mut eligible = false;
        let mut paths = Vec::new();
        for arg in &self.args {
            let path = arg.glob.subst(stem)?;
            let path = self.root.join(path);
            if path.exists() {
                eligible = true
            }
            paths.push(path);
        }
        if eligible {
            Some(paths)
        } else {
            None
        }
    }
}

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

pub fn touch(path: &Path) -> io::Result<()> {
    // Touch the file containing the test
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utime::set_file_times(path, now as i64, now as i64)?;
    Ok(())
}
