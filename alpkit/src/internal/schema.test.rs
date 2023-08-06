use indoc::indoc;

use super::*;
use crate::internal::test_utils::assert;

macro_rules! assert_schema {
    ($schema:expr, $expected:expr) => {{
        assert!(serde_json::to_string_pretty(&$schema).unwrap() == $expected);
    }};
}

#[test]
fn any_of_json() {
    assert_schema!(
        any_of(&[string()]),
        indoc! {r#"
            {
              "anyOf": [
                {
                  "type": "string"
                }
              ]
            }"#}
    )
}

#[test]
fn array_of_items_json() {
    assert_schema!(
        array_of_items(string()),
        indoc! {r#"
            {
              "type": "array",
              "items": {
                "type": "string"
              }
            }"#}
    )
}

#[test]
fn object_with_pattern_properties_json() {
    assert_schema!(
        object_with_pattern_properties("[a-z]", string()),
        indoc! {r#"
            {
              "type": "object",
              "patternProperties": {
                "[a-z]": {
                  "type": "string"
                }
              },
              "additionalProperties": false
            }"#}
    )
}

#[test]
fn string_with_pattern_json() {
    assert_schema!(
        string_with_pattern("[a-z]"),
        indoc! {r#"
            {
              "type": "string",
              "pattern": "[a-z]"
            }"#}
    )
}

fn string() -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        ..Default::default()
    }
    .into()
}
