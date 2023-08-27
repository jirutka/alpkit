use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use field_names::FieldNames;
#[cfg(feature = "validate")]
use garde::Validate;
use mass_cfg_attr::mass_cfg_attr;
#[cfg(feature = "shell-timeout")]
use process_control::{ChildExt, Control};
#[cfg(feature = "schema-gen")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::Dependency;
use crate::internal::exit_status_error::{ExitStatusError, ExitStatusExt};
use crate::internal::key_value_vec_map::{self, KeyValueLike};
use crate::internal::macros::bail;
#[cfg(feature = "validate")]
use crate::internal::regex;
#[cfg(feature = "schema-gen")]
use crate::internal::schema;
use crate::internal::serde_key_value;
use crate::internal::std_ext::{ChunksExactIterator, Tap};
#[cfg(feature = "validate")]
use crate::internal::validators::{
    validate_email, validate_http_url, validate_some_email, validate_source_uri,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Decode(#[from] serde_key_value::Error),

    #[error("shell exited unsuccessfully: '{1}'")]
    Evaluate(#[source] ExitStatusError, String),

    #[error("I/O error occurred when {1}")]
    Io(#[source] io::Error, &'static str),

    #[error("syntax error in secfixes on line {0}: '{1}'")]
    MalformedSecfixes(usize, String),

    #[error("missing sha512sum for: '{0}'")]
    MissingChecksum(String),

    #[error("failed to read file '{1}'")]
    ReadFile(#[source] io::Error, PathBuf),

    #[error("failed to execute shell '{1}'")]
    SpawnShell(#[source] io::Error, String),

    #[error("exceeded timeout {0} ms")]
    Timeout(u128),
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, FieldNames)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[cfg_attr(feature = "schema-gen", derive(JsonSchema))]
#[mass_cfg_attr(feature = "validate", garde)]
#[mass_cfg_attr(feature = "schema-gen", schemars)]
#[garde(allow_unvalidated)]
pub struct Apkbuild {
    /// The name and email address of the package's maintainer. It should be in
    /// the RFC5322 mailbox format, e.g. `Kevin Flynn <kevin.flynn@encom.com>`.
    #[field_names(skip)] // parsed from comments
    #[garde(custom(validate_some_email))]
    #[schemars(email)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,

    #[field_names(skip)] // parsed from comments
    #[garde(inner(custom(validate_email)))]
    #[schemars(inner(email))]
    #[serde(default)]
    pub contributors: Vec<String>,

    /// The name of the main package built from this APKBUILD.
    #[garde(pattern(regex::PKGNAME))]
    #[schemars(regex = "regex::PKGNAME")]
    pub pkgname: String,

    /// The version of the software being packaged.
    #[garde(pattern(regex::PKGVER))]
    #[schemars(regex = "regex::PKGVER")]
    pub pkgver: String,

    /// Alpine package release number (starts at 0).
    pub pkgrel: u32,

    /// A brief, one-line description of the APKBUILD's main package.
    #[garde(length(max = 128), pattern(regex::ONE_LINE))]
    #[schemars(length(max = 128), regex = "regex::ONE_LINE")]
    pub pkgdesc: String,

    /// Homepage of the software being packaged.
    #[garde(custom(validate_http_url))]
    #[schemars(url)]
    pub url: String,

    /// Package architecture(s) to build for. It doesn't contain `all`, `noarch`
    /// or negated architectures -- `arch` is resolved on APKBUILD parsing as
    /// per [`ApkbuildReader::arch_all`].
    #[garde(inner(pattern(regex::WORD)))]
    #[schemars(inner(regex = "regex::WORD"))]
    #[serde(default)]
    pub arch: Vec<String>,

    /// License(s) of the source code from which the main package (and typically
    /// also all subpackages) is built. It should be a SPDX license expression
    /// or a list of SPDX license identifiers separated by a space.
    #[garde(ascii, pattern(regex::ONE_LINE))]
    #[schemars(regex = "regex::ONE_LINE")]
    pub license: String,

    /// Manually specified run-time dependencies of the main package. This
    /// doesn't include dependencies that are autodiscovered by the `abuild`
    /// tool during the build of the package (e.g. shared object dependencies).
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub depends: Vec<Dependency>,

    /// Build-time dependencies.
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub makedepends: Vec<Dependency>,

    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub makedepends_build: Vec<Dependency>,

    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub makedepends_host: Vec<Dependency>,

    /// Dependencies that are only required during the check phase (i.e. for
    /// running tests).
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub checkdepends: Vec<Dependency>,

    /// A set of dependencies that, if all installed, induce installation of the
    /// APKBUILD's main package. `install_if` can be used when a package needs
    /// to be installed when some packages are already installed or are in the
    /// dependency tree.
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub install_if: Vec<Dependency>,

    /// System users to be created when building the package(s).
    #[garde(inner(pattern(regex::USER_NAME)))]
    #[schemars(inner(regex = "regex::USER_NAME"))]
    #[serde(default)]
    pub pkgusers: Vec<String>,

    /// System groups to be created when building the package(s).
    #[garde(inner(pattern(regex::USER_NAME)))]
    #[schemars(inner(regex = "regex::USER_NAME"))]
    #[serde(default)]
    pub pkggroups: Vec<String>,

    /// Providers (packages) that the APKBUILD's main package provides.
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub provides: Vec<Dependency>,

    /// A numeric value which is used by apk-tools to break ties when choosing
    /// a virtual package to satisfy a dependency. Higher values have higher
    /// priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_priority: Option<u32>,

    /// The prefix for all providers derived by parsing pkg-config's name or
    /// `Requires:`.
    #[garde(pattern(regex::PROVIDER))]
    #[schemars(regex = "regex::PROVIDER")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pcprefix: Option<String>,

    /// The prefix for all providers derived by parsing shared objects.
    #[garde(pattern(regex::PROVIDER))]
    #[schemars(regex = "regex::PROVIDER")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sonameprefix: Option<String>,

    /// The packages whose files the APKBUILD's main package is allowed to
    /// overwrite (i.e. both can be installed even if they have conflicting
    /// files).
    #[garde(dive)]
    #[schemars(with = "schema::Dependencies")]
    #[serde(default, with = "key_value_vec_map")]
    pub replaces: Vec<Dependency>,

    /// The priority of the `replaces`. If multiple packages replace files of
    /// each other, then the package with the highest `replaces_priority` wins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces_priority: Option<u32>,

    #[serde(default)]
    pub install: Vec<String>,

    /// Triggers installed `<pkgname>.trigger=<dir1>[:<dir2>...​]`
    #[serde(default)]
    pub triggers: Vec<String>,

    /// Subpackages (names) built from this APKBUILD.
    #[garde(inner(pattern(regex::PKGNAME)))]
    #[schemars(inner(regex = "regex::PKGNAME"))]
    #[serde(default)]
    pub subpackages: Vec<String>,

    /// Both remote and local source files needed for building the package(s).
    #[garde(dive)]
    #[serde(default, rename = "sources")]
    pub source: Vec<Source>,

    /// Build-time options for the `abuild` tool.
    #[garde(inner(pattern(regex::NEGATABLE_WORD)))]
    #[schemars(inner(regex = "regex::NEGATABLE_WORD"))]
    #[serde(default)]
    pub options: Vec<String>,

    /// A map of security vulnerabilities (CVE identifier) fixed in each version
    /// of the APKBUILD's package(s).
    #[field_names(skip)] // parsed from comments
    #[garde(dive)]
    #[schemars(with = "schema::Secfixes")]
    #[serde(default, with = "key_value_vec_map")]
    pub secfixes: Vec<Secfix>,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[cfg_attr(feature = "schema-gen", derive(JsonSchema))]
#[mass_cfg_attr(feature = "validate", garde)]
#[mass_cfg_attr(feature = "schema-gen", schemars)]
pub struct Source {
    /// The file name.
    #[garde(pattern(regex::FILE_NAME))]
    #[schemars(regex = "regex::FILE_NAME")]
    pub name: String,

    /// URI of the file. This is either URL of the remote file or path of the
    /// local file relative to the APKBUILD's directory.
    #[garde(custom(validate_source_uri))]
    pub uri: String,

    /// SHA-512 checksum of the file.
    #[garde(pattern(regex::SHA512))]
    #[schemars(regex = "regex::SHA512")]
    pub checksum: String,
}

impl Source {
    pub fn new<N, U, C>(name: N, uri: U, checksum: C) -> Self
    where
        N: ToString,
        U: ToString,
        C: ToString,
    {
        Source {
            name: name.to_string(),
            uri: uri.to_string(),
            checksum: checksum.to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct Secfix {
    /// A full version of the package that _fixes_ the vulnerabilities, or `0`
    /// for the vulnerabilities that were issued for the upstream but never
    /// affected the package.
    #[garde(pattern(regex::PKGVER_REL_OR_ZERO))]
    pub version: String,

    /// A set of vulnerability IDs (typically CVE).
    #[garde(skip)] // FIXME
    pub fixes: Vec<String>,
}

impl Secfix {
    pub fn new<S: ToString>(version: S, fixes: Vec<String>) -> Self {
        Secfix {
            version: version.to_string(),
            fixes,
        }
    }
}

impl<'a> KeyValueLike<'a> for Secfix {
    type Key = &'a str;
    type Value = Vec<String>;
    type Err = Infallible;

    fn from_key_value(key: Self::Key, value: Self::Value) -> Result<Self, Self::Err> {
        Ok(Secfix::new(key, value))
    }

    fn to_key_value(&'a self) -> (Self::Key, Self::Value) {
        (&self.version, self.fixes.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// The default list of CPU architectures (arch) to which the `all` and `noarch`
/// keywords are expanded.
pub const ARCH_ALL: &[&str] = &[
    "aarch64", "armhf", "armv7", "ppc64le", "riscv64", "s390x", "x86", "x86_64",
];

pub struct ApkbuildReader {
    arch_all: Vec<String>,
    env: HashMap<OsString, OsString>,
    inherit_env: bool,
    shell_cmd: OsString,
    #[allow(unused)]
    time_limit: Duration,

    eval_fields: Vec<&'static str>,
    eval_script: Vec<u8>,
}

impl ApkbuildReader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the list of CPU architectures (arch) to which the `all` and
    /// `noarch` keywords are expanded. The default is [`ARCH_ALL`].
    pub fn arch_all<S: ToString>(&mut self, arches: &[S]) -> &mut Self {
        self.arch_all.extend(arches.iter().map(|s| s.to_string()));
        self
    }

    /// Inserts or updates an environment variable mapping.
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env.insert(OsString::from(&key), OsString::from(&val));
        self
    }

    /// Adds or updates multiple environment variable mappings.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (ref key, ref val) in vars {
            self.env.insert(OsString::from(&key), OsString::from(&val));
        }
        self
    }

    /// Sets if the spawned shell process should inherit environment variables
    /// from the parent process, or the environment should be cleared (default).
    pub fn inherit_env(&mut self, cond: bool) -> &mut Self {
        self.inherit_env = cond;
        self
    }

    /// Changes the shell command used to evaluate an APKBUILD.
    pub fn shell_cmd<S: AsRef<OsStr>>(&mut self, cmd: S) -> &mut Self {
        self.shell_cmd = OsString::from(&cmd);
        self
    }

    #[cfg(feature = "shell-timeout")]
    pub fn time_limit(&mut self, limit: Duration) -> &mut Self {
        self.time_limit = limit;
        self
    }

    pub fn read_apkbuild<P: AsRef<Path>>(&self, filepath: P) -> Result<Apkbuild, Error> {
        let filepath = filepath.as_ref();
        let apkbuild_str =
            fs::read_to_string(filepath).map_err(|e| Error::ReadFile(e, filepath.to_owned()))?;

        let values = self.evaluate(filepath)?;

        let mut arch: Option<&str> = None;
        let mut sha512sums: Option<&str> = None;
        let mut source: Option<&str> = None;

        let parsed = self
            .eval_fields
            .iter()
            .zip(values.trim_end().split_terminator('\x1E'))
            .fold(Vec::with_capacity(64), |mut acc, (key, val)| {
                match *key {
                    "arch" => arch = Some(val),
                    "source" => source = Some(val),
                    "sha512sums" => sha512sums = Some(val),
                    "license" | "pkgdesc" | "pkgver" | "url" => {
                        acc.push((*key, val));
                    }
                    _ => {
                        for mut word in val.split_ascii_whitespace() {
                            if *key == "subpackages" {
                                word = word.split(':').next().unwrap(); // this cannot panic
                            }
                            acc.push((*key, word));
                        }
                    }
                };
                acc
            });

        let mut apkbuild: Apkbuild = serde_key_value::from_ordered_pairs(parsed)?;

        if let Some(arch) = arch {
            apkbuild.arch = parse_and_expand_arch(arch, &self.arch_all);
        }
        if let Some(source) = source {
            apkbuild.source = decode_source_and_sha512sums(source, sha512sums.unwrap_or(""))?;
        }

        apkbuild.maintainer = parse_maintainer(&apkbuild_str).map(|s| s.to_owned());
        apkbuild.contributors = parse_contributors(&apkbuild_str)
            .map(|s| s.to_owned())
            .collect();
        apkbuild.secfixes = parse_secfixes(&apkbuild_str)?;

        Ok(apkbuild)
    }

    fn evaluate(&self, filepath: &Path) -> Result<String, Error> {
        // filepath is validated in `.read_apkbuild`.
        let startdir = filepath
            .parent()
            .unwrap_or_else(|| panic!("invalid APKBUILD path: `{filepath:?}`"));
        let filename = filepath
            .file_name()
            .unwrap_or_else(|| panic!("invalid APKBUILD path: `{filepath:?}`"));

        let mut child = Command::new(&self.shell_cmd)
            .tap_mut_if(!self.inherit_env, |cmd| {
                cmd.env_clear();
            })
            .envs(self.env.iter())
            .env("APKBUILD", filename)
            .tap_mut_if(!startdir.as_os_str().is_empty(), |cmd| {
                cmd.current_dir(startdir);
            })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::SpawnShell(e, self.shell_cmd.to_string_lossy().into_owned()))?;

        let mut stdin = child.stdin.take().unwrap(); // this should never fail
        stdin
            .write_all(&self.eval_script)
            .map_err(|e| Error::Io(e, "writing data to stdin of shell"))?;
        drop(stdin);

        #[cfg(feature = "shell-timeout")]
        let output = child
            .controlled_with_output()
            .pipe_if(!self.time_limit.is_zero(), |ctrl| {
                ctrl.terminate_for_timeout().time_limit(self.time_limit)
            })
            .wait()
            .map_err(|e| Error::Io(e, "waiting on shell process"))?
            .ok_or(Error::Timeout(self.time_limit.as_millis()))?;

        #[cfg(not(feature = "shell-timeout"))]
        let output = child
            .wait_with_output()
            .map_err(|e| Error::Io(e, "waiting on shell process"))?;

        output
            .status
            .exit_ok()
            .map_err(|e| Error::Evaluate(e, String::from_utf8_lossy(&output.stderr).into()))?;

        String::from_utf8(output.stdout).map_err(|e| {
            Error::Io(
                io::Error::new(io::ErrorKind::InvalidData, e),
                "reading shell stdout",
            )
        })
    }
}

impl Default for ApkbuildReader {
    fn default() -> Self {
        // TODO: Remove PATH?
        let path = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());

        // `sha512sums` is not in Apkbuild struct, because it's merged into `source`.
        let eval_fields: Vec<_> = Apkbuild::FIELDS.into_iter().chain(["sha512sums"]).collect();

        let eval_script = eval_fields
            .iter()
            .fold(
                r#". ./"$APKBUILD" >/dev/null; echo "#.to_owned(),
                |acc, field| acc + "$" + field + "\x1E",
            )
            .into_bytes();

        Self {
            arch_all: ARCH_ALL.iter().map(|s| s.to_string()).collect(), // this is suboptiomal :/
            shell_cmd: "/bin/sh".into(),
            env: HashMap::from([("PATH".into(), path)]),
            inherit_env: false,
            time_limit: Duration::from_millis(500),
            eval_fields,
            eval_script,
        }
    }
}

fn parse_and_expand_arch<'v, 's: 'v>(value: &'v str, arch_all: &'s [String]) -> Vec<String> {
    value
        .split_ascii_whitespace()
        .fold(vec![], |mut acc, token| {
            match token {
                "all" | "noarch" => acc.extend(arch_all.iter().map(String::clone)),
                s if s.starts_with('!') => acc.retain(|arch| arch != &s[1..]),
                s => acc.push(s.to_owned()),
            };
            acc
        })
        .tap_mut(|v| {
            v.sort();
            v.dedup();
        })
}

fn parse_comment_attribute<'a>(name: &str, line: &'a str) -> Option<&'a str> {
    line.trim()
        .strip_prefix("# ")
        .and_then(|s| s.trim_start().strip_prefix(name))
        .map(str::trim_start)
        .and_then(|s| (!s.is_empty()).then_some(s))
}

fn parse_maintainer(apkbuild: &str) -> Option<&str> {
    apkbuild
        .lines()
        .find_map(|s| parse_comment_attribute("Maintainer:", s))
}

fn parse_contributors(apkbuild: &str) -> impl Iterator<Item = &str> {
    apkbuild
        .lines()
        .take(10)
        .filter_map(|s| parse_comment_attribute("Contributor:", s))
}

fn parse_secfixes(apkbuild: &str) -> Result<Vec<Secfix>, Error> {
    let mut lines = apkbuild.lines().enumerate();
    let mut secfixes: Vec<Secfix> = vec![];

    if !lines.any(|(_, s)| s.starts_with("# secfixes:")) {
        return Ok(secfixes);
    }

    for pair in lines.map_while(|(i, s)| s.strip_prefix("#   ").map(|s| (i, s))) {
        let line_no = pair.0 + 1;
        let line = pair.1.split(" #").next().unwrap().trim(); // this cannot panic

        if let Some(line) = line.strip_prefix("- ") {
            if let Some(Secfix { fixes, .. }) = secfixes.last_mut() {
                fixes.push(line.trim_start().to_string());
            } else {
                bail!(Error::MalformedSecfixes(line_no, pair.1.to_owned()));
            }
        } else if let Some(key) = line.strip_suffix(':') {
            secfixes.push(Secfix {
                version: key.to_owned(),
                fixes: Vec::with_capacity(3),
            });
        } else {
            bail!(Error::MalformedSecfixes(line_no, pair.1.to_owned()));
        }
    }
    Ok(secfixes)
}

fn decode_source_and_sha512sums(source: &str, sha512sums: &str) -> Result<Vec<Source>, Error> {
    let mut sha512sums: HashMap<&str, &str> = sha512sums
        .split_ascii_whitespace()
        .chunks_exact()
        .map(|[a, b]| (b, a))
        .collect();

    source
        .split_ascii_whitespace()
        .map(|item| {
            let (name, uri) = if let Some((name, uri)) = item.split_once("::") {
                (name, uri)
            } else if let Some((_, name)) = item.rsplit_once('/') {
                (name, item)
            } else {
                (item, item)
            };
            sha512sums
                .remove(name)
                .map(|checksum| Source::new(name, uri, checksum))
                .ok_or_else(|| Error::MissingChecksum(name.to_owned()))
        })
        .collect()
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
#[path = "apkbuild.test.rs"]
mod test;
