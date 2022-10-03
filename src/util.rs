use std::collections::HashSet;
use std::io;
use std::path::Path;

/// An equivalent to the `touch` command.
///
/// # Example
///
/// ```rust
/// # use std::path::Path;
/// # use std::io;
/// use testdata::util::touch;
///
/// # fn main() -> io::Result<()> {
/// let path = Path::new("tests/my_test.rs");
/// # drop(path);
/// # let tmp = tempfile::NamedTempFile::new()?;
/// # let path = tmp.path();
///
/// // Trigger rebuild
/// touch(path)?;
/// # Ok(())
/// # }
/// ```
pub fn touch(path: &Path) -> io::Result<()> {
    // Touch the file containing the test
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utime::set_file_times(path, now as i64, now as i64)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    pub has_diff: bool,
    pub extra: Vec<String>,
    pub missing: Vec<String>,
}

/// Computs difference between two sets of strings.
///
/// # Example
///
/// ```rust
/// use testdata::util::{diff, DiffResult};
/// let d = diff(
///     &["a".to_owned(), "b".to_owned(), "d".to_owned()],
///     &["a".to_owned(), "c".to_owned(), "d".to_owned()]
/// );
/// assert_eq!(
///     d,
///     DiffResult {
///         has_diff: true,
///         extra: vec!["b".to_owned()],
///         missing: vec!["c".to_owned()],
///     },
/// );
/// ```
pub fn diff(set1: &[String], set2: &[String]) -> DiffResult {
    let missing = {
        let set1 = set1.iter().collect::<HashSet<_>>();
        set2.iter()
            .cloned()
            .filter(|stem| !set1.contains(stem))
            .collect::<Vec<_>>()
    };
    let extra = {
        let set2 = set2.iter().collect::<HashSet<_>>();
        set1.iter()
            .cloned()
            .filter(|stem| !set2.contains(stem))
            .collect::<Vec<_>>()
    };
    DiffResult {
        has_diff: !extra.is_empty() || !missing.is_empty(),
        extra,
        missing,
    }
}
