/// TODO: add docs
macro_rules! impl_serde_for_ark_serde_checked {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut bytes =
                    Vec::with_capacity(ark_serialize::CanonicalSerialize::compressed_size(self));
                ark_serialize::CanonicalSerialize::serialize_compressed(self, &mut bytes)
                    .map_err(serde::ser::Error::custom)?;
                bytes.serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                ark_serialize::CanonicalDeserialize::deserialize_compressed(
                    Vec::deserialize(deserializer)?.as_slice(),
                )
                .map_err(serde::de::Error::custom)
            }
        }
    };
}

/// TODO: add docs
macro_rules! impl_serde_for_ark_serde_unchecked {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut bytes =
                    Vec::with_capacity(ark_serialize::CanonicalSerialize::compressed_size(self));
                ark_serialize::CanonicalSerialize::serialize_compressed(self, &mut bytes)
                    .map_err(serde::ser::Error::custom)?;
                bytes.serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                ark_serialize::CanonicalDeserialize::deserialize_compressed_unchecked(
                    Vec::deserialize(deserializer)?.as_slice(),
                )
                .map_err(serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use impl_serde_for_ark_serde_checked;
pub(crate) use impl_serde_for_ark_serde_unchecked;
