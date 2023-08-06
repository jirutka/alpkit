use schemars::gen::SchemaGenerator;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, StringValidation,
    SubschemaValidation,
};
use schemars::{JsonSchema, Map};

use crate::apkbuild::Secfix;
use crate::dependency::Dependency;
use crate::internal::regex;
use crate::internal::std_ext::Tap;

/// This wrapper type is only used for `schemars`.
pub(crate) struct Dependencies(Vec<Dependency>);

impl JsonSchema for Dependencies {
    fn schema_name() -> String {
        "Dependencies".to_owned()
    }

    /// Generates schema:
    /// ```json
    /// {
    ///   "anyOf": [{
    ///     "type": "object",
    ///     "additionalProperties": false,
    ///     "patternProperties": {
    ///       "^[a-zA-Z0-9_.+\\-:/\\[\\]]+$": {
    ///         "type": "string",
    ///         "pattern": "^(?:\\*|[<>=~]{1,2} ?[0-9]+(?:\\.[0-9]+)*[a-z]?(?:_[a-z]+[0-9]*)*(?:-r[0-9]+)?)$"
    ///       }
    ///     }
    ///   }, {
    ///     "type": "array",
    ///     "items": {
    ///       "type": "string",
    ///       "pattern": "^[a-zA-Z0-9_.+\\-:/\\[\\]]+(?:[<>=~]{1,2}[0-9]+(?:\\.[0-9]+)*[a-z]?(?:_[a-z]+[0-9]*)*(?:-r[0-9]+)?)?$"
    ///     }
    ///   }]
    /// }
    /// ```
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let array = array_of_items(string_with_pattern(&regex::PROVIDER_WITH_CONSTRAINT));
        let object = object_with_pattern_properties(
            &regex::PROVIDER,
            string_with_pattern(&regex::DEP_CONSTRAINT),
        );
        any_of(&[array, object])
    }
}

/// This wrapper type is only used for `schemars`.
pub(crate) struct Secfixes(Vec<Secfix>);

impl JsonSchema for Secfixes {
    fn schema_name() -> String {
        "Secfixes".to_owned()
    }

    /// Generates schema:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "additionalProperties": false,
    ///   "patternProperties": {
    ///     "^(?:[0-9]+(?:\\.[0-9]+)*[a-z]?(?:_[a-z]+[0-9]*)*-r[0-9]+|0)$": {
    ///       "type": "array",
    ///       "items": {
    ///         "type": "string"
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        object_with_pattern_properties(
            &regex::PKGVER_REL_OR_ZERO,
            gen.subschema_for::<Vec<String>>(),
        )
    }
}

fn any_of(subschemas: &[Schema]) -> Schema {
    SchemaObject {
        subschemas: Some(Box::new(SubschemaValidation {
            any_of: Some(subschemas.into()),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

fn array_of_items(items: Schema) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            items: Some(items.into()),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

fn object_with_pattern_properties<P: ToString>(pattern: P, value_schema: Schema) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            additional_properties: Some(Box::new(Schema::Bool(false))),
            pattern_properties: Map::new().tap_mut(|map| {
                map.insert(pattern.to_string(), value_schema);
            }),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

fn string_with_pattern<P: ToString>(pattern: P) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        string: Some(Box::new(StringValidation {
            pattern: Some(pattern.to_string()),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

#[cfg(test)]
#[path = "schema.test.rs"]
mod test;
