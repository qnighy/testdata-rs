use std::collections::HashSet;
use std::path::{Path, PathBuf, StripPrefixError};

use thiserror::Error as StdError;
use walkdir::WalkDir;

#[derive(Debug, StdError)]
pub enum Error {
    #[error("Error during walk: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Cannot compute relative path from {} to {}", .1.display(), .2.display())]
    StripPrefix(#[source] StripPrefixError, PathBuf, PathBuf),
    #[error("Got a non-utf8 path: {0:?}")]
    InvalidPath(PathBuf),
    #[error("Invalid glob: {0:?}")]
    InvalidGlob(String),
    #[error("Different glob types are mixed")]
    MixedGlob,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GlobSpec {
    pub args: Vec<ArgSpec>,
}

impl GlobSpec {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    pub fn glob(&self) -> Result<Vec<String>, Error> {
        self.glob_dir(Path::new("."))
    }

    pub fn glob_dir(&self, root: &Path) -> Result<Vec<String>, Error> {
        let mut stems = HashSet::new();
        let args = self
            .args
            .iter()
            .map(|arg| arg.parse())
            .collect::<Result<Vec<_>, _>>()?;
        let glob_type = if !args.is_empty() {
            let glob_type = args[0].glob_type;
            if !args.iter().all(|arg| arg.glob_type == glob_type) {
                return Err(Error::MixedGlob);
            }
            glob_type
        } else {
            GlobType::Recursive
        };
        for entry in WalkDir::new(root).sort_by_file_name() {
            let entry = entry?;
            let file_name = entry
                .path()
                .strip_prefix(root)
                .map_err(|e| Error::StripPrefix(e, root.to_owned(), entry.path().to_owned()))?;
            let file_name = file_name
                .to_str()
                .ok_or_else(|| Error::InvalidPath(entry.path().to_owned()))?;
            for arg in &args {
                if file_name.starts_with(&arg.prefix) && file_name.ends_with(&arg.suffix) {
                    let stem = &file_name[arg.prefix.len()..file_name.len() - arg.suffix.len()];
                    if glob_type == GlobType::Recursive || !stem.contains('/') {
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
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ArgSpec {
    pub path: String,
}

impl ArgSpec {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
        }
    }

    fn parse(&self) -> Result<ParsedArgSpec, Error> {
        let pos = self
            .path
            .find('*')
            .ok_or_else(|| Error::InvalidGlob(self.path.clone()))?;
        let (pos2, glob_type) = if self.path[pos..].starts_with("**/*") {
            (pos + 4, GlobType::Recursive)
        } else {
            (pos + 1, GlobType::Single)
        };
        let suffix = &self.path[pos2..];
        if suffix.contains('*') {
            return Err(Error::InvalidGlob(self.path.clone()));
        }
        Ok(ParsedArgSpec {
            prefix: self.path[..pos].to_owned(),
            glob_type,
            suffix: suffix.to_owned(),
        })
    }
}

#[derive(Debug)]
struct ParsedArgSpec {
    prefix: String,
    glob_type: GlobType,
    suffix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobType {
    /// `**/*`
    Recursive,
    /// `*`
    Single,
}
