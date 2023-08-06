use std::fmt::Display;

use garde::rules::pattern::Matcher;
use garde::rules::AsStr;
use once_cell::sync::Lazy;
use regex_lite::Regex;

macro_rules! lazy_regex {
    ($re:expr) => {
        LazyRegex(Lazy::new(|| Regex::new($re).unwrap()))
    };
    ($($re:expr),*) => {
        LazyRegex(Lazy::new(|| Regex::new(&[$($re),*].concat()).unwrap()))
    };
}

// We need this to implement the `Matcher` trait from `garde` on `regex_lite`
// (because of the Orphan Rule).
pub(crate) struct LazyRegex(Lazy<Regex>);

impl Matcher for LazyRegex {
    fn is_match(&self, haystack: &str) -> bool {
        self.0.is_match(haystack)
    }
}

impl AsStr for LazyRegex {
    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for LazyRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// This is a bit stricter than apk-tools, which allows letters in more places
// than is sensible, but it's compatible with what's used in aports.
const PKGVER_PART: &str = r"[0-9]+(?:\.[0-9]+)*[a-z]?[0-9]*(?:_[a-z]+[0-9]*)*";
const PROVIDER_PART: &str = r"[a-zA-Z0-9_.+\-:/\[\]]+";

#[cfg(feature = "schema-gen")]
pub(crate) static DEP_CONSTRAINT: LazyRegex = lazy_regex!(
    r"^(?:\*||!|!?[<>=~]{1,2} ?",
    PKGVER_PART,
    r"(?:-r[0-9]+)?)$"
);
pub(crate) static FILE_NAME: LazyRegex = lazy_regex!(r"^[^/\t\n\r ]+$");
pub(crate) static NEGATABLE_WORD: LazyRegex = lazy_regex!(r"^!?[a-z0-9_-]+$");
pub(crate) static ONE_LINE: LazyRegex = lazy_regex!(r"^[^\n\r]*$");
// This is not codified anywhere, this regex is based on the current practice.
pub(crate) static PKGNAME: LazyRegex = lazy_regex!(r"^[a-zA-Z0-9][a-zA-Z0-9_.+-]*$");
pub(crate) static PKGVER: LazyRegex = lazy_regex!("^", PKGVER_PART, "$");
pub(crate) static PKGVER_MAYBE_REL: LazyRegex = lazy_regex!("^", PKGVER_PART, r"(?:-r[0-9]+)?$");
pub(crate) static PKGVER_REL: LazyRegex = lazy_regex!("^", PKGVER_PART, r"-r[0-9]+$");
pub(crate) static PKGVER_REL_OR_ZERO: LazyRegex = lazy_regex!("^(?:", PKGVER_PART, "-r[0-9]+|0)$");
// This is not codified anywhere, this regex is based on the current practice.
pub(crate) static PROVIDER: LazyRegex = lazy_regex!("^", PROVIDER_PART, "$");
// This is not codified anywhere, this regex is empirical.
pub(crate) static REPO_PIN: LazyRegex = lazy_regex!(r"^[^\t\n\r @<>=~]+$");
pub(crate) static SHA1: LazyRegex = lazy_regex!(r"^[a-f0-9]{40}$");
pub(crate) static SHA256: LazyRegex = lazy_regex!(r"^[a-f0-9]{64}$");
pub(crate) static SHA512: LazyRegex = lazy_regex!(r"^[a-f0-9]{128}$");
pub(crate) static TRIGGER_PATH: LazyRegex = lazy_regex!(r"^(?:/[^/\t\n\r :]+)+/?$");
pub(crate) static USER_NAME: LazyRegex = lazy_regex!(r"^[a-z_][a-z0-9._-]*\$?$");
pub(crate) static WORD: LazyRegex = lazy_regex!(r"^[a-z0-9_-]+$");

#[cfg(feature = "schema-gen")]
pub(crate) static PROVIDER_WITH_CONSTRAINT: LazyRegex = lazy_regex!(
    r"^!?",
    PROVIDER_PART,
    r"(?:[<>=~]{1,2}",
    PKGVER_PART,
    r"(?:-r[0-9]+)?)?$"
);

/// A simplified regex for validation of email address in the mailbox format
/// (e.g. `Kevin Flynn <kevin.flynn@encom.com>`).
/// It doesn't accept IP address in the domain part.
/// It doesn't accept local-part and IDN in non-ASCII format (IDN is a mess).
pub(crate) static EMAIL: LazyRegex = lazy_regex!(
    r#"(?x)^
    [^\n\r@<>"]*  # display-name (allows non-ASCII)
    <
    [a-zA-Z0-9.!\#$%&*+/=?^_{|}~-]{1,64}  # local-part per RFC 5322 w/o single quote (') and backtick (`)
    @
    (?:  # domain
        [a-zA-Z0-9]
        (?:[a-zA-Z0-9-]*[a-zA-Z0-9])?
        \.
    )+
    [a-zA-Z][a-zA-Z0-9-]*[a-zA-Z]  # TLD
    >
    $"#
);

/// A simplified regex for URL validation according to RFC 3986. It only
/// accepts http and https schemes. It doesn't accept the userinfo component
/// and IDN in non-ASCII format (IDN is a mess). It doesn't distinguish
/// between the path, query and fragment components (the main focus is on
/// the domain name). It accepts even invalid IPv4 and IPv6 addresses.
///
/// We don't use the `url` validator (`#[validate(url)]`) because it has
/// unreasonable overhead in terms of dependencies and binary size due to
/// IDN support (which we don't want anyway).
pub(crate) static URL: LazyRegex = lazy_regex!(
    r"(?ix-u)^
    https?://  # scheme
    (?:    # IPv6-like
        \[(?:[a-f0-9]{1,4}::?){1,7}[a-f0-9]{0,4}\]
        |  # IPv4-like
        [0-9]{1,3}(?:\.[0-9]{1,3}){3}
        |  # domain
        (?:
            [a-z0-9]
            (?:[a-z0-9-]*[a-z0-9])?
            \.
        )+
        [a-z][a-z0-9-]*[a-z]  # TLD
    )
    (?::[0-9]+)?  # port
    (?:[/?\#][a-z0-9\-._~!$&'()*+,;=:/?\#@%]*)?  # path, query and fragment
    $"
);

#[cfg(test)]
#[rustfmt::skip]
#[path = "regex.test.rs"]
mod test;
