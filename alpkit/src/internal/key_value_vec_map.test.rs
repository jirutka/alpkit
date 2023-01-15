use std::num;

use indoc::indoc;
use serde::{Deserialize, Serialize};

use super::*;
use crate::internal::serde_key_value;
use crate::internal::test_utils::assert;

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Pair {
    name: String,
    value: u64,
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some((key, value)) = s.split_once('=') {
            Ok(Pair {
                name: key.to_owned(),
                value: value.parse().map_err(de::Error::custom)?,
            })
        } else {
            Err(de::Error::custom("expected key=value"))
        }
    }
}

impl<'a> KeyValueLike<'a> for Pair {
    type Key = &'a str;
    type Value = String;
    type Err = num::ParseIntError;

    fn from_key_value(key: Self::Key, value: Self::Value) -> Result<Self, Self::Err> {
        Ok(Self {
            name: key.to_owned(),
            value: value.parse()?,
        })
    }

    fn to_key_value(&'a self) -> (Self::Key, Self::Value) {
        (&self.name, self.value.to_string())
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Pairs {
    #[serde(with = "self")]
    pairs: Vec<Pair>,
}

fn fixture_struct() -> Pairs {
    Pairs {
        pairs: vec![
            Pair {
                name: "foo".into(),
                value: 42,
            },
            Pair {
                name: "bar".into(),
                value: 55,
            },
        ],
    }
}

#[test]
fn deserialize_json_map() {
    let json = indoc!(
        r#"
        {
            "pairs": {
                "foo": "42",
                "bar": "55"
            }
        }
    "#
    );
    let expected = fixture_struct();

    assert!(serde_json::from_str::<Pairs>(json).unwrap() == expected);
}

#[test]
fn deserialize_json_seq() {
    let json = indoc!(
        r#"
        {
            "pairs": [
                "foo=42",
                "bar=55"
            ]
        }
    "#
    );
    let expected = fixture_struct();

    assert!(serde_json::from_str::<Pairs>(json).unwrap() == expected);
}

#[test]
fn deserialize_key_value_seq() {
    let input = vec![("pairs", "foo=42"), ("pairs", "bar=55")];
    let expected = fixture_struct();
    assert!(
        serde_key_value::from_pairs::<Pairs>(input).unwrap() == expected,
        "multiple elements"
    );

    let input = vec![("pairs", "foo=42")];
    let expected = Pairs {
        pairs: vec![Pair {
            name: "foo".to_owned(),
            value: 42,
        }],
    };
    assert!(
        serde_key_value::from_pairs::<Pairs>(input).unwrap() == expected,
        "single element"
    );
}

#[test]
fn serialize_map() {
    let input = fixture_struct();

    assert!(&serde_json::to_string(&input).unwrap() == r#"{"pairs":{"foo":"42","bar":"55"}}"#);
}
