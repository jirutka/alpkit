use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;
use serde_json::json;

use crate::apkbuild::Secfix;
use crate::dependency::Dependency;
use crate::internal::regex;

/// This wrapper type is only used for `schemars`.
pub(crate) struct Dependencies(Vec<Dependency>);

impl JsonSchema for Dependencies {
    fn schema_name() -> String {
        "Dependencies".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        serde_json::from_value(json!({
            "anyOf": [{
                "type": "object",
                "additionalProperties": false,
                "patternProperties": {
                    "^[a-zA-Z0-9_.+\\-:/\\[\\]]+$": {
                        "type": "string",
                        "pattern": &regex::DEP_CONSTRAINT.to_string(),
                    },
                },
            }, {
                "type": "array",
                "items": {
                    "type": "string",
                    "pattern": &regex::PROVIDER_WITH_CONSTRAINT.to_string(),
                },
            }]
        }))
        .expect("unable to process JSON schema for Dependencies")
    }
}

/// This wrapper type is only used for `schemars`.
pub(crate) struct Secfixes(Vec<Secfix>);

impl JsonSchema for Secfixes {
    fn schema_name() -> String {
        "Secfixes".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> Schema {
        serde_json::from_value(json!({
            "type": "object",
            "additionalProperties": false,
            "patternProperties": {
                &regex::PKGVER_REL_OR_ZERO.to_string(): {
                    "type": "array",
                    "items": {
                        "type": "string",
                    },
                },
            },
        })).expect("unable to process JSON schema for Secfixes")
    }
}

#[cfg(test)]
#[path = "schema.test.rs"]
mod test;
