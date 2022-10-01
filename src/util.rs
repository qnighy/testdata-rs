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
