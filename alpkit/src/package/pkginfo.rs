use serde::{self, Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::Dependency;
use crate::internal::key_value_vec_map;
use crate::internal::macros::bail;
use crate::internal::serde_key_value;

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
pub struct PkgInfo {
    /// The name and email address of the package's maintainer. It should be in
    /// the RFC5322 mailbox format, e.g. `Kevin Flynn <kevin.flynn@encom.com>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,

    /// The package name.
    pub pkgname: String,

    /// A full version of the package (including the release number `-r<n>`).
    pub pkgver: String,

    /// A brief, one-line description of the package.
    pub pkgdesc: String,

    /// The homepage of the packaged software.
    pub url: String,

    /// The architecture of the package (e.g.: `x86_64`).
    pub arch: String,

    /// License(s) of the source code from which the package was built. It
    /// should be a SPDX license expression or a list of SPDX license
    /// identifiers separated by a space.
    pub license: String,

    /// Dependencies of this package. It doesn't contain “anti-dependencies”
    /// (conflicts, e.g. `!foo`), these are separated in the `conflicts` field.
    /// This also means that the `conflict` field in each [Dependency] is always
    /// `false`.
    #[serde(default, alias = "depend", with = "key_value_vec_map")]
    pub depends: Vec<Dependency>,

    /// Conflicts of this package, i.e. it cannot be installed if any of the
    /// named packages is installed.
    ///
    /// This field actually does not exist in `PKGINFO` – it contains
    /// “anti-dependencies” (conflicts, e.g. `!foo`) extracted from the
    /// `depend` field. The `conflict` field in each [Dependency] is always
    /// `false`.
    #[serde(default, with = "key_value_vec_map")]
    pub conflicts: Vec<Dependency>,

    /// A set of dependencies that, if all installed, induce installation of
    /// this package. `install_if` can be used when a package needs to be
    /// installed when some packages are already installed or are in the
    /// dependency tree.
    #[serde(default, with = "key_value_vec_map")]
    pub install_if: Vec<Dependency>,

    /// Providers (packages) that this package provides.
    #[serde(default, with = "key_value_vec_map")]
    pub provides: Vec<Dependency>,

    /// A numeric value which is used by apk-tools to break ties when choosing
    /// a virtual package to satisfy a dependency. Higher values have higher
    /// priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_priority: Option<u16>,

    /// Packages whose files this package is allowed to overwrite (i.e. both can
    /// be installed even if they have conflicting files).
    #[serde(default, with = "key_value_vec_map")]
    pub replaces: Vec<Dependency>,

    /// The priority of the `replaces`. If multiple packages replace files of
    /// each other, then the package with the highest `replaces_priority` wins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces_priority: Option<u16>,

    /// A list of monitored directories, if this package installs a trigger. The
    /// directory paths may contain wildcards (`*`).
    ///
    /// apk-tools can "monitor" directories and execute a trigger if any package
    /// installed/uninstalled any file in the monitored directory.
    #[serde(default)]
    pub triggers: Vec<String>,

    /// The name of the APKBUILD (its main package) from which the package was built.
    pub origin: String,

    /// The SHA-1 hash of the git commit from which the package was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// An unix timestamp of the package build date/time.
    pub builddate: i64,

    /// The name and email address of the person (or machine) who built the
    /// package. It should be in the RFC5322 mailbox format, e.g.
    /// `Kevin Flynn <kevin.flynn@encom.com>`.
    pub packager: String,

    /// The installed-size of the package in bytes.
    pub size: usize,

    /// The hex-encoded SHA-256 checksum of the data tarball.
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
