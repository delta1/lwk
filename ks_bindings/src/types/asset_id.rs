use std::{fmt::Display, str::FromStr};

use elements::hex::ToHex;

use crate::UniffiCustomTypeConverter;

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct AssetId {
    inner: elements::AssetId,
}

impl Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<elements::AssetId> for AssetId {
    fn from(inner: elements::AssetId) -> Self {
        AssetId { inner }
    }
}

impl From<AssetId> for elements::AssetId {
    fn from(value: AssetId) -> Self {
        value.inner
    }
}

uniffi::custom_type!(AssetId, String);
impl UniffiCustomTypeConverter for AssetId {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        let inner = elements::AssetId::from_str(&val)?;
        Ok(AssetId { inner })
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.inner.to_hex()
    }
}
