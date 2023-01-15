use serde::Deserialize;

use super::*;
use crate::internal::test_utils::{assert, assert_let, S};

#[derive(Deserialize, Debug, PartialEq)]
struct StructA {
    f_string: String,
    f_i64: i64,
    f_bool: bool,
    #[serde(default)]
    f_bool_def: bool,
    f_enum: Enum,
    vec_string: Vec<String>,
    #[serde(default)]
    vec_string_def: Vec<String>,
    opt_string: Option<String>,
    opt_u16: Option<u16>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct StructB {
    str: String,
    number: u32,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Enum {
    Small,
    Medium,
    Large,
}

#[test]
fn decodes_valid_input() {
    let input = vec![
        ("f_string", "test"),
        ("f_i64", "1672081283"),
        ("f_bool", "true"),
        ("vec_string", "first"),
        ("opt_string", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        ("vec_string", "second"),
        ("vec_string", "third"),
        ("f_enum", "medium"),
    ];
    let expected = StructA {
        f_string: S!("test"),
        f_i64: 1672081283,
        f_bool: true,
        f_bool_def: false,
        f_enum: Enum::Medium,
        vec_string_def: vec![],
        vec_string: vec![S!("first"), S!("second"), S!("third")],
        opt_string: Some(S!("da39a3ee5e6b4b0d3255bfef95601890afd80709")),
        opt_u16: None,
    };

    assert_eq!(from_pairs::<StructA>(input).unwrap(), expected);

    let input = vec![
        ("f_string", "test"),
        ("f_i64", "1672081283"),
        ("f_bool", "true"),
        ("vec_string", "first"),
        ("vec_string", "second"),
        ("vec_string", "third"),
        ("opt_string", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        ("f_enum", "medium"),
    ];

    assert_eq!(from_ordered_pairs::<_, StructA>(input).unwrap(), expected);
}

#[test]
fn fails_when_missing_field() {
    let input = vec![("f_string", "test"), ("f_i64", "1672081283")];

    assert_let!(Err(Error::MissingField("f_bool")) = from_pairs::<StructA>(input));
}

#[test]
fn fails_when_invalid_value() {
    let input = vec![("str", "foo"), ("number", "wat")];

    assert_let!(Err(Error::InvalidField(source, field)) = from_pairs::<StructB>(input));
    assert!(source.to_string() == "invalid digit found in string");
    assert!(field == "number");
}

#[test]
fn fails_when_invalid_type() {
    let input = vec![("number", "123"), ("str", "foo"), ("str", "bar")];

    assert_let!(Err(Error::InvalidField(source, field)) = from_pairs::<StructB>(input));
    assert!(source.to_string() == "invalid type: sequence, expected a string");
    assert!(field == "str");
}
