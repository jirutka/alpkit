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

macro_rules! S {
    ($s:expr) => {
        String::from($s)
    };
}
pub(crate) use S;
