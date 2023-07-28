use garde::{Error, Result};

use crate::internal::regex;

pub(crate) fn validate_http_url(value: &str, _context: &()) -> Result {
    if regex::URL.is_match(value) {
        Ok(())
    } else {
        Err(Error::new(
            "is not a valid URL with http or https scheme and without userinfo",
        ))
    }
}

pub(crate) fn validate_email(value: &str, _context: &()) -> Result {
    if regex::EMAIL.is_match(value) {
        Ok(())
    } else {
        Err(Error::new(
            "is not a valid email address in the mailbox format (e.g. Foo <foo@example.org>)",
        ))
    }
}

pub(crate) fn validate_some_email(opt: &Option<String>, _context: &()) -> Result {
    if let Some(value) = opt {
        validate_email(value, &())
    } else {
        Ok(())
    }
}

pub(crate) fn validate_source_uri(value: &str, _context: &()) -> Result {
    if value.contains("://") && !regex::URL.is_match(value) {
        validate_http_url(value, &())
    } else if value.starts_with('/') || value.starts_with("../") || value.contains("/../") {
        Err(Error::new("is not a relative path with no '../'"))
    } else if value.contains(|c| char::is_ascii_whitespace(&c)) {
        Err(Error::new("must not contain whitespaces"))
    } else {
        Ok(())
    }
}
