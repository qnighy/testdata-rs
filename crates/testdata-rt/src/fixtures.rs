use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub paths: Vec<PathBuf>,
}

impl Fixture {
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
            panic!("Fixture.paths is empty");
        }
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
        self.paths.first().expect("Fixture.paths is empty")
    }
}
