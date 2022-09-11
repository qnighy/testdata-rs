use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlobParseError {
    #[error("No wildcard found: {src:?}")]
    NoWildcard { src: String },
    #[error("Multiple wildcards found: {src:?}")]
    MultipleWildcards { src: String },
    #[error("'**' appeared without '*': {src:?}")]
    StrayRecursiveWildcard { src: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobPattern {
    prefix: String,
    wildcard: Wildcard,
    suffix: String,
}

impl GlobPattern {
    pub fn new(src: &str) -> Self {
        src.parse().unwrap()
    }

    pub fn do_match<'a>(&self, file_name: &'a str) -> Option<&'a str> {
        if file_name.starts_with(&self.prefix) && file_name.ends_with(&self.suffix) {
            let stem = &file_name[self.prefix.len()..file_name.len() - self.suffix.len()];
            if self.wildcard == Wildcard::Recursive || !stem.contains('/') {
                return Some(stem);
            }
        }
        None
    }

    pub fn subst(&self, stem: &str) -> Option<String> {
        if self.wildcard == Wildcard::Recursive || !stem.contains('/') {
            Some(format!("{}{}{}", self.prefix, stem, self.suffix))
        } else {
            None
        }
    }
}

impl FromStr for GlobPattern {
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

impl fmt::Display for GlobPattern {
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
                prefix: "tests/fixtures/".to_owned(),
                wildcard: Wildcard::Recursive,
                suffix: "-in.txt".to_owned(),
            }
        );

        assert_eq!(
            GlobPattern::new("tests/fixtures/*-out.txt"),
            GlobPattern {
                prefix: "tests/fixtures/".to_owned(),
                wildcard: Wildcard::Single,
                suffix: "-out.txt".to_owned(),
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
        ];
        for &case in &cases {
            assert_eq!(GlobPattern::new(case).to_string(), case);
        }
    }

    #[test]
    fn test_match() {
        let pat = GlobPattern::new("tests/fixtures/**/*-in.txt");
        assert_eq!(pat.do_match("tests/fixtures/foo-in.txt"), Some("foo"));
        assert_eq!(
            pat.do_match("tests/fixtures/foo/bar-in.txt"),
            Some("foo/bar")
        );
        assert_eq!(pat.do_match("tests/fixtures/foo-out.txt"), None);

        let pat = GlobPattern::new("tests/fixtures/*-in.txt");
        assert_eq!(pat.do_match("tests/fixtures/foo-in.txt"), Some("foo"));
        assert_eq!(pat.do_match("tests/fixtures/foo/bar-in.txt"), None);
        assert_eq!(pat.do_match("tests/fixtures/foo-out.txt"), None);
    }

    #[test]
    fn test_subst() {
        let pat = GlobPattern::new("tests/fixtures/**/*-in.txt");
        assert_eq!(pat.subst("foo"), Some("tests/fixtures/foo-in.txt".into()));
        assert_eq!(
            pat.subst("foo/bar"),
            Some("tests/fixtures/foo/bar-in.txt".into())
        );

        let pat = GlobPattern::new("tests/fixtures/*-in.txt");
        assert_eq!(pat.subst("foo"), Some("tests/fixtures/foo-in.txt".into()));
        assert_eq!(pat.subst("foo/bar"), None);
    }
}
