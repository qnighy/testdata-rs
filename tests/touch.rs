#![cfg(unix)]

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;

use testdata::util::touch;

// 2000-01-01
const TIME1: i64 = 946684800;
// 2020-01-01
const TIME2: i64 = 1577836800;

#[test]
fn test_touch() -> io::Result<()> {
    let tmp = tempfile::NamedTempFile::new()?;
    let path = tmp.path();

    utime::set_file_times(path, TIME1, TIME1)?;
    let old_metadata = fs::metadata(path)?;
    eprintln!("old_metadata = {:#?}", old_metadata);

    touch(path)?;

    let new_metadata = fs::metadata(path)?;
    eprintln!("new_metadata = {:#?}", new_metadata);
    assert!(old_metadata.atime() < TIME2);
    assert!(old_metadata.mtime() < TIME2);
    assert!(new_metadata.atime() > TIME2);
    assert!(new_metadata.mtime() > TIME2);
    drop(tmp);
    Ok(())
}
