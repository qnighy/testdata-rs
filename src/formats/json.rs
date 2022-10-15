use bytemuck::TransparentWrapper;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::snapshots::Snapshot;
use crate::test_input::TestInput;

#[cfg_attr(all(feature = "__doc_cfg", doc), doc(cfg(feature = "serde_json")))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, TransparentWrapper)]
#[repr(transparent)]
pub struct Json<T>(pub T);

impl<T> TestInput for Json<T>
where
    T: DeserializeOwned,
{
    type Err = serde_json::Error;

    fn try_read_from(data: &[u8]) -> Result<Self, Self::Err> {
        let value = serde_json::from_slice(data)?;
        Ok(Json(value))
    }
}

impl<T> Snapshot for Json<T>
where
    T: DeserializeOwned + Serialize,
{
    type Borrowed = Json<T>;
    type Owned = Json<T>;

    fn borrow(&self) -> &Self::Borrowed {
        &self
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }
}
