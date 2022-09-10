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
        let args = self
            .args
            .iter()
            .map(|arg| arg.parse())
            .collect::<Result<Vec<_>, _>>()?;
        if !args.is_empty() {
            let glob_type = args[0].glob_type;
            if !args.iter().all(|arg| arg.glob_type == glob_type) {
                return Err(Error::MixedGlob);
            }
        };
        for entry in WalkDir::new(&self.root).sort_by_file_name() {
            let entry = entry?;
            let file_name = entry
                .path()
                .strip_prefix(&self.root)
                .map_err(|e| Error::StripPrefix(e, self.root.clone(), entry.path().to_owned()))?;
            let file_name = file_name
                .to_str()
                .ok_or_else(|| Error::InvalidPath(entry.path().to_owned()))?;
            for arg in &args {
                if let Some(stem) = arg.extract(file_name) {
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
            // TODO: memoize parsing
            let path = arg.parse().unwrap().subst(stem)?;
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

impl ParsedArgSpec {
    fn extract<'a>(&self, file_name: &'a str) -> Option<&'a str> {
        if file_name.starts_with(&self.prefix) && file_name.ends_with(&self.suffix) {
            let stem = &file_name[self.prefix.len()..file_name.len() - self.suffix.len()];
            if self.glob_type == GlobType::Recursive || !stem.contains('/') {
                return Some(stem);
            }
        }
        None
    }

    fn subst(&self, stem: &str) -> Option<String> {
        if self.glob_type == GlobType::Recursive || !stem.contains('/') {
            Some(format!("{}{}{}", self.prefix, stem, self.suffix))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobType {
    /// `**/*`
    Recursive,
    /// `*`
    Single,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_type_clone() {
        let _ = GlobType::Recursive.clone();
        let _ = GlobType::Single.clone();
    }

    #[test]
    fn test_glob_type_debug() {
        let _ = format!("{:?}", GlobType::Recursive);
        let _ = format!("{:?}", GlobType::Single);
    }
}
