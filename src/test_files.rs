use std::env;
use std::fs;
use std::io;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFile {
    pub paths: Vec<PathBuf>,
}

impl TestFile {
    pub fn raw_read(&self) -> Vec<u8> {
        self.try_raw_read().unwrap()
    }

    pub fn raw_read_opt(&self) -> Option<Vec<u8>> {
        match self.try_raw_read() {
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            result => Some(result.unwrap()),
        }
    }

    pub fn try_raw_read(&self) -> io::Result<Vec<u8>> {
        let mut first_error = None;
        for path in &self.paths {
            match fs::read(path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                    continue;
                }
                result => return result,
            }
        }
        if let Some(first_error) = first_error {
            Err(first_error)
        } else {
            panic!("TestFile.paths is empty");
        }
    }

    pub fn remove(&self) {
        self.try_remove().unwrap();
    }

    pub fn try_remove(&self) -> io::Result<()> {
        fs::remove_file(self.path_for_writing())?;
        for path in &self.paths[1..] {
            if path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Cannot remove readonly test file: {}", path.display()),
                ));
            }
        }
        Ok(())
    }

    pub fn raw_write(&self, contents: &[u8]) {
        self.try_raw_write(contents).unwrap();
    }

    pub fn try_raw_write(&self, contents: &[u8]) -> io::Result<()> {
        let path = self.path_for_writing();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)
    }

    pub fn exists(&self) -> bool {
        self.paths.iter().any(|path| path.exists())
    }

    pub fn path(&self) -> Option<&Path> {
        self.paths
            .iter()
            .map(|path| &**path)
            .find(|&path| path.exists())
    }

    pub fn path_for_writing(&self) -> &Path {
        self.paths.first().expect("TestFile.paths is empty")
    }
}

pub fn pending<F>(test_file: &TestFile, f: F)
where
    F: FnOnce(),
{
    let update_pending = env::var_os("UPDATE_PENDING").unwrap_or_default() == "true";
    let result = catch_unwind(AssertUnwindSafe(f));
    let actual = result.as_ref().copied().map_err(|e| {
        if let Some(e) = e.downcast_ref::<String>() {
            &e[..]
        } else if let Some(&e) = e.downcast_ref::<&'static str>() {
            e
        } else {
            "Box<Any>"
        }
        .to_owned()
    });
    let expected = if let Some(s) = test_file.raw_read_opt() {
        Err(String::from_utf8_lossy(&s).trim_end().to_owned())
    } else {
        Ok(())
    };
    let ok = match (&expected, &actual) {
        (Ok(_), Ok(_)) => true,
        (Err(expected), Err(actual)) => actual.contains(expected),
        (_, _) => false,
    };
    if ok {
        // do nothing
    } else if update_pending {
        match &actual {
            Ok(_) => test_file.remove(),
            Err(e) => {
                let e = if e.is_empty() || e.ends_with("\n") {
                    e.to_owned()
                } else {
                    format!("{}\n", e)
                };
                test_file.raw_write(e.as_bytes());
            }
        }
    } else {
        match result {
            Ok(()) => panic!(
                "Expected the test to panic (pending):\n{}",
                expected.as_ref().unwrap_err()
            ),
            Err(e) => resume_unwind(e),
        }
    }
}
