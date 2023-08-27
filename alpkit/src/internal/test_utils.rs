pub(crate) use assert2::{assert, let_assert as assert_let};

macro_rules! assert_from_to_json {
    ($strukt:expr, $json:expr $(,)?) => {{
        fn assert<T: ::serde::de::DeserializeOwned + ::serde::ser::Serialize>(
            strukt: T,
            json: ::serde_json::Value,
        ) {
            let config = ::assert_json_diff::Config::new(::assert_json_diff::CompareMode::Strict);

            if let Err(error) =
                ::assert_json_diff::assert_json_matches_no_panic(&strukt, &json, config.clone())
            {
                panic!(
                    "\n\nStruct to JSON: {}\n\nActual JSON:\n{}\n\n",
                    error,
                    ::serde_json::to_string_pretty(&strukt).unwrap()
                );
            }
            if let Err(error) = ::assert_json_diff::assert_json_matches_no_panic(
                &::serde_json::from_str::<T>(&json.to_string())
                    .expect("failed to deserialize from JSON"),
                &strukt,
                config,
            ) {
                panic!(
                    "\n\nJSON to struct: {}\n\nGiven JSON:\n{}\n\n",
                    error,
                    ::serde_json::to_string_pretty(&json).unwrap()
                );
            }
        }
        assert($strukt, $json);
    }};
}
pub(crate) use assert_from_to_json;

#[cfg(feature = "schema-gen")]
macro_rules! assert_schema_invalid {
    ($instance:expr, $schema:expr, errors_count = $errors_count:expr) => {
        use ::jsonschema::output::BasicOutput;

        let instance = ::serde_json::to_value($instance).unwrap();
        let result = $schema.apply(&instance).basic();

        assert_let!(BasicOutput::Invalid(ref errors) = result);
        assert!(errors.len() == $errors_count, "{}", ::serde_json::to_string_pretty(&result).unwrap());
    };
}
#[cfg(feature = "schema-gen")]
pub(crate) use assert_schema_invalid;

#[cfg(feature = "schema-gen")]
macro_rules! assert_schema_valid {
    ($instance:expr, $schema:expr) => {
        use ::jsonschema::output::BasicOutput;

        let instance = ::serde_json::to_value($instance).unwrap();
        let result = $schema.apply(&instance).basic();

        assert_let!(
            BasicOutput::Valid(_) = &result,
            "{}",
            ::serde_json::to_string_pretty(&result).unwrap()
        );
    };
}
#[cfg(feature = "schema-gen")]
pub(crate) use assert_schema_valid;

macro_rules! S {
    ($s:expr) => {
        String::from($s)
    };
}
pub(crate) use S;

#[cfg(feature = "schema-gen")]
pub(crate) fn compile_json_schema<T: schemars::JsonSchema>() -> jsonschema::JSONSchema {
    let schema = schemars::gen::SchemaSettings::draft2019_09()
        .into_generator()
        .into_root_schema_for::<T>();
    let schema = serde_json::to_value(schema).unwrap();

    jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft201909)
        .should_validate_formats(true)
        .compile(&schema)
        .expect("valid JSON schema")
}
