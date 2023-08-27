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
                "description": "A map of package (or provider) names and their version constraint.",
                "examples": [
                    {
                        "rust": "*",
                        "so:libX11.so.6": "!",
                        "cmd:rust": ">= 1.71_rc1-r1",
                        "nginx": "!< 1.24",
                    },
                ],
                "additionalProperties": false,
                "patternProperties": {
                    &regex::PROVIDER.to_string(): {
                        "type": "string",
                        "title": "Version constraint",
                        "description": "The value can be one of:\n\
                            - `*` - any version\n\
                            - `!` - conflict (negative dependency)\n\
                            - `<operator> <version>` - version constraint\n\
                            - `!<operator> <version>` - conflict with version constraint (shouldn't be used)\
                            \n\n\
                            Syntax: `(* | ! | [!]<operator> <version>)`",
                        "examples": [
                            "*",
                            "!",
                            ">= 1.71_rc1-r1",
                            "!< 1.2",
                        ],
                        "pattern": &regex::DEP_CONSTRAINT.to_string(),
                    },
                },
            }, {
                "type": "array",
                "items": {
                    "type": "string",
                    "description": "A package (or provider) name with an optional version constraint in \
                        the same format as in APKBUILD. An exclamation mark (`!`) at the beginning \
                        indicates a negative dependency (conflict).\
                        \n\n\
                        Syntax: `[!]<name>[<operator>[<version>]]`",
                    "examples": [
                        "rust",
                        "!so:libX11.so.6",
                        "cmd:rust>=1.71_rc1-r1",
                        "!nginx<1.24",
                    ],
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
            "title": "Security fixes",
            "description": "A map of known security vulnerabilities (typically CVEs) fixed in \
                each version of the package. The key is a full version (including the `-r') of \
                the package, and the value is a list of the vulnerability IDs that have been \
                fixed in that version. The special key \"0\" groups together vulnerability IDs \
                that were issued for the upstream but never affected the package.",
            "examples": [
                {
                    "1.1.1l-r0": ["CVE-2021-3711", "CVE-2021-3712"],
                    "0": ["CVE-2022-1292", "CVE-2022-2068"],
                },
            ],
            "additionalProperties": false,
            "patternProperties": {
                &regex::PKGVER_REL_OR_ZERO.to_string(): {
                    "type": "array",
                    "items": {
                        "title": "Vulnerability ID",
                        "examples": [
                            "CVE-2021-3711",
                        ],
                        "type": "string",
                    },
                },
            },
        }))
        .expect("unable to process JSON schema for Secfixes")
    }
}

#[cfg(test)]
#[path = "schema.test.rs"]
mod test;
