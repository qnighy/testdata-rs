use std::convert::Infallible;
use std::error::Error;
use std::str::{self, Utf8Error};

pub trait TestInput: Sized {
    type Err: Error;
    fn try_read_from(data: &[u8]) -> Result<Self, Self::Err>;
    fn read_from(data: &[u8]) -> Self {
        Self::try_read_from(data).unwrap()
    }
}

impl<T> TestInput for Box<T>
where
    T: TestInput,
{
    type Err = T::Err;
    fn try_read_from(data: &[u8]) -> Result<Self, Self::Err> {
        let value = T::try_read_from(data)?;
        Ok(Box::new(value))
    }
}

impl TestInput for Vec<u8> {
    type Err = Infallible;
    fn try_read_from(data: &[u8]) -> Result<Self, Self::Err> {
        Ok(data.to_owned())
    }
}

impl TestInput for String {
    type Err = Utf8Error;
    fn try_read_from(data: &[u8]) -> Result<Self, Self::Err> {
        let s = str::from_utf8(data)?;
        Ok(s.to_owned())
    }
}
