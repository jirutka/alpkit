use std::error::Error as StdError;
use std::fmt;
use std::iter;
use std::result::Result as StdResult;

use serde::de::value::{MapAccessDeserializer, SeqDeserializer};
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, Visitor};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("missing required field '{0}'")]
    MissingField(&'static str),

    #[error("invalid field '{1}'")]
    InvalidField(#[source] Box<dyn StdError>, String),

    #[error("decode error: {0}")]
    Other(String),

    #[doc(hidden)]
    #[error("decode error: {0}")]
    Internal(#[source] Box<dyn StdError>),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Other(msg.to_string())
    }

    fn missing_field(field: &'static str) -> Self {
        Error::MissingField(field)
    }
}

type Result<T> = StdResult<T, Error>;

type KeyVal<'a> = (&'a str, &'a str);

#[derive(Debug)]
struct Value<T>(T);

impl<'de> IntoDeserializer<'de, Error> for Value<&'de str> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

macro_rules! forward_parsed_values {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(Error::Internal(Box::new(e))),
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for Value<&'de str> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(vec![self.0].into_deserializer())
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(self.0.into_deserializer())
    }

    forward_parsed_values! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    serde::forward_to_deserialize_any! {
        byte_buf
        bytes
        char
        identifier
        ignored_any
        map
        str
        string
        struct
        tuple
        tuple_struct
        unit
        unit_struct
    }
}

struct KeyValueDeserializer<'de, I: Iterator<Item = KeyVal<'de>>> {
    input: iter::Peekable<I>,
}

impl<'de, I: Iterator<Item = KeyVal<'de>>> KeyValueDeserializer<'de, I> {
    fn new(input: I) -> Self {
        KeyValueDeserializer {
            input: input.peekable(),
        }
    }
}

impl<'de, I: Iterator<Item = KeyVal<'de>>> MapAccess<'de> for KeyValueDeserializer<'de, I> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if let Some((key, _)) = self.input.peek() {
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        // Panic because this indicates a bug in the program rather than an expected failure.
        let (key, value) = self
            .input
            .next()
            .expect("MapAccess::next_value_seed invalid state");

        if self.input.peek().map(|t| t.0) != Some(key) {
            seed.deserialize(Value(value).into_deserializer())
        } else {
            let mut values = Vec::with_capacity(16);
            values.push(Value(value));

            while let Some(next) = self.input.next_if(|next| next.0 == key) {
                values.push(Value(next.1));
            }
            seed.deserialize(SeqDeserializer::new(values.into_iter()))
        }
        .map_err(|e| match e {
            Error::Internal(source) => Error::InvalidField(source, key.to_owned()),
            Error::Other(msg) => Error::InvalidField(msg.into(), key.to_owned()),
            _ => e,
        })
    }
}

pub(crate) fn from_pairs<T>(mut pairs: Vec<KeyVal<'_>>) -> Result<T>
where
    T: de::DeserializeOwned,
{
    pairs.sort_by_key(|kv| kv.0);
    from_ordered_pairs(pairs)
}

pub(crate) fn from_ordered_pairs<'de, I, T>(pairs: I) -> Result<T>
where
    I: IntoIterator<Item = KeyVal<'de>>,
    T: de::DeserializeOwned,
{
    let map = KeyValueDeserializer::new(pairs.into_iter());
    let de = MapAccessDeserializer::new(map);

    T::deserialize(de)
}

#[cfg(test)]
#[path = "serde_key_value.test.rs"]
mod test;
