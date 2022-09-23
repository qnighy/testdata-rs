use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// A syntax error in a glob pattern.
#[derive(Debug, Error)]
pub enum GlobParseError {
    #[error("No wildcard found: {src:?}")]
    NoWildcard { src: String },
    #[error("Multiple wildcards found: {src:?}")]
    MultipleWildcards { src: String },
    #[error("'**' appeared without '*': {src:?}")]
    StrayRecursiveWildcard { src: String },
}

/// A parsed glob pattern, like `tests/fixtures/**/*.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobPattern {
    branches: Vec<GlobBranch>,
}

impl GlobPattern {
    /// Creates a glob pattern from a string representation.
    pub fn new(src: &str) -> Self {
        src.parse().unwrap()
    }

    /// Matches a single path against this pattern.
    pub fn do_match<'a>(&self, file_name: &'a str) -> Vec<&'a str> {
        let mut matches = Vec::new();
        for branch in &self.branches {
            if let Some(m) = branch.do_match(file_name) {
                matches.push(m);
            }
        }
        matches
    }

    /// Assigns the match result in the pattern to get the path(s).
    pub fn subst(&self, stem: &str) -> Vec<String> {
        self.branches
            .iter()
            .filter_map(|branch| branch.subst(stem))
            .collect::<Vec<_>>()
    }

    /// Returns known prefixes from this pattern.
    pub fn prefixes(&self) -> Vec<String> {
        self.branches
            .iter()
            .map(|branch| branch.prefix.clone())
            .collect::<Vec<_>>()
    }
}

impl FromStr for GlobPattern {
    type Err = GlobParseError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let branches = src
            .split(",")
            .map(|branch| branch.parse::<GlobBranch>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { branches })
    }
}

impl fmt::Display for GlobPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, branch) in self.branches.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{}", branch)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlobBranch {
    prefix: String,
    wildcard: Wildcard,
    suffix: String,
}

impl GlobBranch {
    fn do_match<'a>(&self, file_name: &'a str) -> Option<&'a str> {
        if file_name.starts_with(&self.prefix) && file_name.ends_with(&self.suffix) {
            let stem = &file_name[self.prefix.len()..file_name.len() - self.suffix.len()];
            if self.wildcard == Wildcard::Recursive || !stem.contains('/') {
                return Some(stem);
            }
        }
        None
    }

    fn subst(&self, stem: &str) -> Option<String> {
        if self.wildcard == Wildcard::Recursive || !stem.contains('/') {
            Some(format!("{}{}{}", self.prefix, stem, self.suffix))
        } else {
            None
        }
    }
}

impl FromStr for GlobBranch {
    type Err = GlobParseError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let pos = src.find('*').ok_or_else(|| GlobParseError::NoWildcard {
            src: src.to_owned(),
        })?;
        let (pos2, glob_type) = if src[pos..].starts_with("**/*") {
            (pos + 4, Wildcard::Recursive)
        } else if src[pos..].starts_with("**") {
            return Err(GlobParseError::StrayRecursiveWildcard {
                src: src.to_owned(),
            });
        } else {
            (pos + 1, Wildcard::Single)
        };
        let suffix = &src[pos2..];
        if suffix.contains('*') {
            return Err(GlobParseError::MultipleWildcards {
                src: src.to_owned(),
            });
        }
        Ok(Self {
            prefix: src[..pos].to_owned(),
            wildcard: glob_type,
            suffix: suffix.to_owned(),
        })
    }
}

impl fmt::Display for GlobBranch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.prefix, self.wildcard, self.suffix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wildcard {
    /// `**/*`
    Recursive,
    /// `*`
    Single,
}

impl fmt::Display for Wildcard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Wildcard::*;
        match self {
            Recursive => f.write_str("**/*"),
            Single => f.write_str("*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            GlobPattern::new("tests/fixtures/**/*-in.txt"),
            GlobPattern {
                branches: vec![GlobBranch {
                    prefix: "tests/fixtures/".to_owned(),
                    wildcard: Wildcard::Recursive,
                    suffix: "-in.txt".to_owned(),
                }]
            }
        );

        assert_eq!(
            GlobPattern::new("tests/fixtures/*-out.txt"),
            GlobPattern {
                branches: vec![GlobBranch {
                    prefix: "tests/fixtures/".to_owned(),
                    wildcard: Wildcard::Single,
                    suffix: "-out.txt".to_owned(),
                }]
            }
        );

        assert_eq!(
            GlobPattern::new("foo/*.txt,bar/*.txt"),
            GlobPattern {
                branches: vec![
                    GlobBranch {
                        prefix: "foo/".to_owned(),
                        wildcard: Wildcard::Single,
                        suffix: ".txt".to_owned(),
                    },
                    GlobBranch {
                        prefix: "bar/".to_owned(),
                        wildcard: Wildcard::Single,
                        suffix: ".txt".to_owned(),
                    }
                ]
            }
        );
    }

    #[test]
    fn test_parse_error() {
        let e = "tests/fixtures/in.txt".parse::<GlobPattern>().unwrap_err();
        assert_eq!(
            e.to_string(),
            "No wildcard found: \"tests/fixtures/in.txt\""
        );

        let e = "tests/fixtures/*/*/in.txt"
            .parse::<GlobPattern>()
            .unwrap_err();
        assert_eq!(
            e.to_string(),
            "Multiple wildcards found: \"tests/fixtures/*/*/in.txt\""
        );

        let e = "tests/fixtures/**/in.txt"
            .parse::<GlobPattern>()
            .unwrap_err();
        assert_eq!(
            e.to_string(),
            "'**' appeared without '*': \"tests/fixtures/**/in.txt\""
        );
    }

    #[test]
    fn test_stringify() {
        let cases = [
            "tests/fixtures/**/*-in.txt",
            "tests/fixtures/*-out.txt",
            "*.rs",
            "tests/fixtures/**/*",
            "foo/*.txt,bar/*.rs",
        ];
        for &case in &cases {
            assert_eq!(GlobPattern::new(case).to_string(), case);
        }
    }

    #[test]
    fn test_match() {
        let empty: Vec<&str> = vec![];

        let pat = GlobPattern::new("tests/fixtures/**/*-in.txt");
        assert_eq!(pat.do_match("tests/fixtures/foo-in.txt"), vec!["foo"]);
        assert_eq!(
            pat.do_match("tests/fixtures/foo/bar-in.txt"),
            vec!["foo/bar"]
        );
        assert_eq!(pat.do_match("tests/fixtures/foo-out.txt"), empty);

        let pat = GlobPattern::new("tests/fixtures/*-in.txt");
        assert_eq!(pat.do_match("tests/fixtures/foo-in.txt"), vec!["foo"]);
        assert_eq!(pat.do_match("tests/fixtures/foo/bar-in.txt"), empty);
        assert_eq!(pat.do_match("tests/fixtures/foo-out.txt"), empty);

        let pat = GlobPattern::new("foo/**/*.txt,foo/bar/**/*.txt");
        assert_eq!(pat.do_match("foo/a.txt"), vec!["a"]);
        assert_eq!(pat.do_match("foo/bar/a.txt"), vec!["bar/a", "a"]);
    }

    #[test]
    fn test_subst() {
        let empty: Vec<&str> = vec![];

        let pat = GlobPattern::new("tests/fixtures/**/*-in.txt");
        assert_eq!(
            pat.subst("foo"),
            vec!["tests/fixtures/foo-in.txt".to_owned()]
        );
        assert_eq!(
            pat.subst("foo/bar"),
            vec!["tests/fixtures/foo/bar-in.txt".to_owned()]
        );

        let pat = GlobPattern::new("tests/fixtures/*-in.txt");
        assert_eq!(
            pat.subst("foo"),
            vec!["tests/fixtures/foo-in.txt".to_owned()]
        );
        assert_eq!(pat.subst("foo/bar"), empty);

        let pat = GlobPattern::new("foo/*.txt,bar/*.rs");
        assert_eq!(
            pat.subst("a"),
            vec!["foo/a.txt".to_owned(), "bar/a.rs".to_owned()]
        );
        assert_eq!(pat.subst("foo/bar"), empty);
    }
}
