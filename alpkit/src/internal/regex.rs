use once_cell::sync::Lazy;
use regex::Regex;

macro_rules! lazy_regex {
    ($re:expr) => {
        Lazy::new(|| Regex::new($re).unwrap())
    };
    ($($re:expr),*) => {
        Lazy::new(|| Regex::new(&[$($re),*].concat()).unwrap())
    };
}

// This is a bit stricter than apk-tools, which allows letters in more places
// than is sensible, but it's compatible with what's used in aports.
const PKGVER_PART: &str = r"[0-9]+(?:\.[0-9]+)*[a-z]?[0-9]*(?:_[a-z]+[0-9]*)*";

pub(crate) static FILE_NAME: Lazy<Regex> = lazy_regex!(r"^[^/\t\n\r ]+$");
pub(crate) static NEGATABLE_WORD: Lazy<Regex> = lazy_regex!(r"^!?[a-z0-9_-]+$");
pub(crate) static ONE_LINE: Lazy<Regex> = lazy_regex!(r"^[^\n\r]*$");
// This is not codified anywhere, this regex is based on the current practice.
pub(crate) static PKGNAME: Lazy<Regex> = lazy_regex!(r"^[a-zA-Z0-9][a-zA-Z0-9_.+-]*$");
pub(crate) static PKGVER: Lazy<Regex> = lazy_regex!("^", PKGVER_PART, "$");
pub(crate) static PKGVER_MAYBE_REL: Lazy<Regex> = lazy_regex!("^", PKGVER_PART, r"(?:-r[0-9]+)?$");
pub(crate) static PKGVER_REL: Lazy<Regex> = lazy_regex!("^", PKGVER_PART, r"-r[0-9]+$");
pub(crate) static PKGVER_REL_OR_ZERO: Lazy<Regex> =
    lazy_regex!("^(?:", PKGVER_PART, "-r[0-9]+|0)$");
// This is not codified anywhere, this regex is based on the current practice.
pub(crate) static PROVIDER: Lazy<Regex> = lazy_regex!(r"^[a-zA-Z0-9_.+\-:/\[\]]+$");
// This is not codified anywhere, this regex is empirical.
pub(crate) static REPO_PIN: Lazy<Regex> = lazy_regex!(r"^[^\t\n\r @<>=~]+$");
pub(crate) static SHA1: Lazy<Regex> = lazy_regex!(r"^[a-f0-9]{40}$");
pub(crate) static SHA256: Lazy<Regex> = lazy_regex!(r"^[a-f0-9]{64}$");
pub(crate) static SHA512: Lazy<Regex> = lazy_regex!(r"^[a-f0-9]{128}$");
pub(crate) static TRIGGER_PATH: Lazy<Regex> = lazy_regex!(r"^(?:/[^/\t\n\r :]+)+/?$");
pub(crate) static USER_NAME: Lazy<Regex> = lazy_regex!(r"^[a-z_][a-z0-9._-]*\$?$");
pub(crate) static WORD: Lazy<Regex> = lazy_regex!(r"^[a-z0-9_-]+$");

/// A simplified regex for validation of email address in the mailbox format
/// (e.g. `Kevin Flynn <kevin.flynn@encom.com>`).
/// It doesn't accept IP address in the domain part.
/// It doesn't accept local-part and IDN in non-ASCII format (IDN is a mess).
pub(crate) static EMAIL: Lazy<Regex> = lazy_regex!(
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
pub(crate) static URL: Lazy<Regex> = lazy_regex!(
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
#[path = "regex.test.rs"]
mod test;
