// This is based on https://gitlab.com/mneumann_ntecs/serde-key-value-vec-map.
use std::marker::PhantomData;
use std::{fmt, iter};

use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

/// A trait for converting a value from/to key-value pair.
pub(crate) trait KeyValueLike<'a>: Sized {
    type Key;
    type Value;
    type Err: fmt::Display;

    fn from_key_value(key: Self::Key, value: Self::Value) -> Result<Self, Self::Err>;
    fn to_key_value(&'a self) -> (Self::Key, Self::Value);
}

#[allow(clippy::type_complexity)]
struct KeyValueVecMapVisitor<T, K, V>(PhantomData<fn() -> (T, K, V)>);

impl<'de, T, K, V> de::Visitor<'de> for KeyValueVecMapVisitor<T, K, V>
where
    T: Deserialize<'de> + KeyValueLike<'de, Key = K, Value = V>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("sequence or map")
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut vec = Vec::with_capacity(map.size_hint().unwrap_or(3));
        while let Some((key, val)) = map.next_entry()? {
            vec.push(T::from_key_value(key, val).map_err(|e| de::Error::custom(e.to_string()))?);
        }
        Ok(vec)
    }

    // XXX: This is a hack for deserializing PkgInfo from apkv2 .PKGINFO format.
    // The problem is that it's not self-describing - a sequence is represented
    // as a repeated field, so we cannot distinguish between scalars and
    // single-element sequences.
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_seq(de::value::SeqDeserializer::new(iter::once(v)))
    }
}

/// Deserializes a value implementing [`Deserialize`] and [`KeyValueLike`] into
/// a vector using the given Serde deserializer.
pub(crate) fn deserialize<'de, D, T, K, V>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + KeyValueLike<'de, Key = K, Value = V>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserializer.deserialize_any(KeyValueVecMapVisitor(PhantomData))
}

/// Serializes a vector of [`KeyValueLike`] elements into a map using
/// the given Serde serializer.
pub(crate) fn serialize<'a, S, T, K, V>(vec: &'a Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: KeyValueLike<'a, Key = K, Value = V>,
    K: Serialize,
    V: Serialize,
{
    use ser::SerializeMap;

    let mut map = serializer.serialize_map(Some(vec.len()))?;
    for item in vec {
        let (ref key, ref value) = item.to_key_value();
        map.serialize_entry(key, value)?;
    }
    map.end()
}

#[cfg(test)]
#[path = "key_value_vec_map.test.rs"]
mod test;
