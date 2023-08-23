#[cfg(feature = "validate")]
use garde::Validate;
use mass_cfg_attr::mass_cfg_attr;
#[cfg(feature = "schema-gen")]
use schemars::JsonSchema;
use serde::{self, Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::Dependencies;
use crate::internal::macros::bail;
#[cfg(feature = "validate")]
use crate::internal::regex;
use crate::internal::serde_key_value;
#[cfg(feature = "validate")]
use crate::internal::validators::{validate_email, validate_http_url, validate_some_email};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum PkgInfoError {
    #[error(transparent)]
    Decode(#[from] serde_key_value::Error),

    #[error("syntax error on line {0}: missing ' = ' in '{1}'")]
    Syntax(usize, String),
}

////////////////////////////////////////////////////////////////////////////////

/// This struct represents the `.PKGINFO` file.
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[cfg_attr(feature = "schema-gen", derive(JsonSchema))]
#[mass_cfg_attr(feature = "validate", garde)]
#[mass_cfg_attr(feature = "schema-gen", schemars)]
#[garde(allow_unvalidated)]
pub struct PkgInfo {
    /// The name and email address of the package's maintainer. It should be in
    /// the RFC5322 mailbox format, e.g. `Kevin Flynn <kevin.flynn@encom.com>`.
    #[garde(custom(validate_some_email))]
    #[schemars(email)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,

    /// The package name.
    #[garde(pattern(regex::PKGNAME))]
    #[schemars(regex = "regex::PKGNAME")]
    pub pkgname: String,

    /// A full version of the package (including the release number `-r<n>`).
    #[garde(pattern(regex::PKGVER_REL))]
    #[schemars(regex = "regex::PKGVER_REL")]
    pub pkgver: String,

    /// A brief, one-line description of the package.
    #[garde(length(max = 128), pattern(regex::ONE_LINE))]
    #[schemars(length(max = 128), regex = "regex::ONE_LINE")]
    pub pkgdesc: String,

    /// The homepage of the packaged software.
    #[garde(custom(validate_http_url))]
    #[schemars(url)]
    pub url: String,

    /// The architecture of the package (e.g.: `x86_64`).
    #[garde(pattern(regex::WORD))]
    #[schemars(regex = "regex::WORD")]
    pub arch: String,

    /// License(s) of the source code from which the package was built. It
    /// should be a SPDX license expression or a list of SPDX license
    /// identifiers separated by a space.
    #[garde(ascii, pattern(regex::ONE_LINE))]
    #[schemars(regex = "regex::ONE_LINE")]
    pub license: String,

    /// Dependencies of this package. It doesn't contain “anti-dependencies”
    /// (conflicts, e.g. `!foo`), these are separated in the `conflicts` field.
    /// This also means that the `conflict` field in each [Dependency] is always
    /// `false`.
    #[garde(dive)]
    #[serde(default, alias = "depend")]
    pub depends: Dependencies,

    /// Conflicts of this package, i.e. it cannot be installed if any of the
    /// named packages is installed.
    ///
    /// This field actually does not exist in `PKGINFO` – it contains
    /// “anti-dependencies” (conflicts, e.g. `!foo`) extracted from the
    /// `depend` field. The `conflict` field in each [Dependency] is always
    /// `false`.
    #[garde(dive)]
    #[serde(default)]
    pub conflicts: Dependencies,

    /// A set of dependencies that, if all installed, induce installation of
    /// this package. `install_if` can be used when a package needs to be
    /// installed when some packages are already installed or are in the
    /// dependency tree.
    #[garde(dive)]
    #[serde(default)]
    pub install_if: Dependencies,

    /// Providers (packages) that this package provides.
    #[garde(dive)]
    #[serde(default)]
    pub provides: Dependencies,

    /// A numeric value which is used by apk-tools to break ties when choosing
    /// a virtual package to satisfy a dependency. Higher values have higher
    /// priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_priority: Option<u16>,

    /// Packages whose files this package is allowed to overwrite (i.e. both can
    /// be installed even if they have conflicting files).
    #[garde(dive)]
    #[serde(default)]
    pub replaces: Dependencies,

    /// The priority of the `replaces`. If multiple packages replace files of
    /// each other, then the package with the highest `replaces_priority` wins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces_priority: Option<u16>,

    /// A list of monitored directories, if this package installs a trigger. The
    /// directory paths may contain wildcards (`*`).
    ///
    /// apk-tools can "monitor" directories and execute a trigger if any package
    /// installed/uninstalled any file in the monitored directory.
    #[garde(inner(pattern(regex::TRIGGER_PATH)))]
    #[schemars(inner(regex = "regex::TRIGGER_PATH"))]
    #[serde(default)]
    pub triggers: Vec<String>,

    /// The name of the APKBUILD (its main package) from which the package was built.
    #[garde(pattern(regex::PKGNAME))]
    #[schemars(regex = "regex::PKGNAME")]
    pub origin: String,

    /// The SHA-1 hash of the git commit from which the package was built.
    #[garde(pattern(regex::SHA1))]
    #[schemars(regex = "regex::SHA1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// An unix timestamp of the package build date/time.
    #[garde(range(min = 0))]
    pub builddate: i64,

    /// The name and email address of the person (or machine) who built the
    /// package. It should be in the RFC5322 mailbox format, e.g.
    /// `Kevin Flynn <kevin.flynn@encom.com>`.
    #[garde(custom(validate_email))]
    pub packager: String,

    /// The installed-size of the package in bytes.
    pub size: usize,

    /// The hex-encoded SHA-256 checksum of the data tarball.
    #[garde(pattern(regex::SHA256))]
    #[schemars(regex = "regex::SHA256")]
    pub datahash: String,
}

impl PkgInfo {
    /// Parses and deserializes the given `.PKGINFO` file contents.
    pub fn parse(s: &str) -> Result<Self, PkgInfoError> {
        parse_key_value(s)
            .try_fold(Vec::with_capacity(64), |mut acc, kv| {
                match kv {
                    Ok((key @ ("install_if" | "triggers"), val)) => {
                        for word in val.split_ascii_whitespace() {
                            acc.push((key, word));
                        }
                    }
                    Ok(("depend", val)) => {
                        acc.push(if let Some(val) = val.strip_prefix('!') {
                            ("conflicts", val)
                        } else {
                            ("depends", val)
                        });
                    }
                    Ok(kv) => acc.push(kv),
                    Err(e) => bail!(e),
                };
                Ok(acc)
            })
            .and_then(|pairs| serde_key_value::from_pairs(pairs).map_err(PkgInfoError::from))
    }
}

fn parse_key_value(s: &str) -> impl Iterator<Item = Result<(&str, &str), PkgInfoError>> {
    s.lines().enumerate().filter_map(|(lno, line)| {
        if line.is_empty() || line.starts_with('#') {
            None
        } else if let Some(item) = line.split_once(" = ") {
            Some(Ok(item))
        } else {
            Some(Err(PkgInfoError::Syntax(lno + 1, line.to_string())))
        }
    })
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
#[path = "pkginfo.test.rs"]
mod test;
