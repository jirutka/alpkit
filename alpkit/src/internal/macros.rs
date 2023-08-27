macro_rules! bail {
    ($err:expr) => {
        return Err($err)
    };
}
pub(crate) use bail;

#[cfg(feature = "schema-gen")]
macro_rules! define_schema_for {
    ($type:ty, $json:tt) => {
        impl ::schemars::JsonSchema for $type {
            fn schema_name() -> String {
                stringify!($type).to_owned()
            }

            fn json_schema(_: &mut ::schemars::gen::SchemaGenerator) -> ::schemars::schema::Schema {
                ::serde_json::from_value(::serde_json::json!($json)).expect(concat!(
                    "unable to process JSON schema for ",
                    stringify!($type)
                ))
            }
        }
    };
}
#[cfg(feature = "schema-gen")]
pub(crate) use define_schema_for;
