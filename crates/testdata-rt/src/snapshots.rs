use std::borrow::Borrow;
use std::env;

use crate::fixtures::Fixture;
use crate::test_input::TestInput;

#[macro_export]
macro_rules! assert_snapshot {
    ($e:expr, snapshot = $fixture:expr) => {
        match (&($e), &($fixture)) {
            (e, fixture) => $crate::assert_snapshot_helper(e, fixture, |lhs, rhs| {
                $crate::pretty_assertions::assert_eq!(*lhs, *rhs)
            }),
        }
    };
}

pub fn assert_snapshot_helper<T, F>(e: &T, fixture: &Fixture, assertion: F)
where
    T: Snapshot + ?Sized,
    T::Borrowed: PartialEq,
    F: FnOnce(&T::Borrowed, &T::Borrowed),
{
    let mode = SnapshotMode::current();
    let expected = if let Some(expected) = fixture.raw_read_opt() {
        expected
    } else if mode >= SnapshotMode::New {
        write_snapshot(e, fixture);
        return;
    } else {
        panic!(
            "Snapshot does not exist: {}",
            fixture.path_for_writing().display()
        );
    };

    let expected = T::Owned::read_from(&expected);
    if *e.borrow() != *expected.borrow() {
        if mode == SnapshotMode::All {
            write_snapshot(e, fixture);
            return;
        }
        assertion(e.borrow(), expected.borrow());
        unreachable!();
    }
}

fn write_snapshot<T>(e: &T, fixture: &Fixture)
where
    T: Snapshot + ?Sized,
{
    let bytes = e.to_bytes();
    fixture.raw_write(&bytes);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotMode {
    None,
    New,
    All,
}

impl SnapshotMode {
    pub fn current() -> SnapshotMode {
        let update_snapshots = env::var("UPDATE_SNAPSHOTS").unwrap_or_else(|_| String::from(""));
        let ci = env::var("CI").unwrap_or_else(|_| String::from(""));
        if update_snapshots == "all" || update_snapshots == "true" || update_snapshots == "1" {
            return SnapshotMode::All;
        } else if update_snapshots == "new" {
            return SnapshotMode::New;
        } else if update_snapshots == "none"
            || update_snapshots == "false"
            || update_snapshots == "0"
        {
            return SnapshotMode::None;
        }
        if ci == "true" || ci == "1" {
            return SnapshotMode::None;
        }
        SnapshotMode::New
    }
}

pub trait Snapshot {
    type Borrowed: ?Sized;
    type Owned: Borrow<Self::Borrowed> + TestInput;

    fn borrow(&self) -> &Self::Borrowed;
    fn to_bytes(&self) -> Vec<u8>;
}

impl<'a, T> Snapshot for &'a T
where
    T: Snapshot + ?Sized,
{
    type Borrowed = T::Borrowed;
    type Owned = T::Owned;

    fn borrow(&self) -> &Self::Borrowed {
        <T as Snapshot>::borrow(self)
    }

    fn to_bytes(&self) -> Vec<u8> {
        <T as Snapshot>::to_bytes(self)
    }
}

impl<'a, T> Snapshot for &'a mut T
where
    T: Snapshot + ?Sized,
{
    type Borrowed = T::Borrowed;
    type Owned = T::Owned;

    fn borrow(&self) -> &Self::Borrowed {
        <T as Snapshot>::borrow(self)
    }

    fn to_bytes(&self) -> Vec<u8> {
        <T as Snapshot>::to_bytes(self)
    }
}

impl Snapshot for [u8] {
    type Borrowed = [u8];
    type Owned = Vec<u8>;

    fn borrow(&self) -> &Self::Borrowed {
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_owned()
    }
}

impl Snapshot for Vec<u8> {
    type Borrowed = Vec<u8>;
    type Owned = Vec<u8>;

    fn borrow(&self) -> &Self::Borrowed {
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }
}

impl Snapshot for str {
    type Borrowed = str;
    type Owned = String;

    fn borrow(&self) -> &Self::Borrowed {
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_owned()
    }
}

impl Snapshot for String {
    type Borrowed = String;
    type Owned = String;

    fn borrow(&self) -> &Self::Borrowed {
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_owned()
    }
}
