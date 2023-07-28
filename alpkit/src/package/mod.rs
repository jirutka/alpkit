mod fileinfo;
mod pkginfo;

use std::io::{self, BufRead, Read};
use std::path::Path;
use std::slice::Iter;
use std::str::{self, FromStr};

use flate2::bufread::GzDecoder;
#[cfg(feature = "validate")]
use garde::Validate;
use mass_cfg_attr::mass_cfg_attr;
use serde::{de, Deserialize, Serialize};
use tar::Archive;
use thiserror::Error;

use crate::internal::macros::bail;
#[cfg(feature = "validate")]
use crate::internal::regex;

pub use fileinfo::*;
pub use pkginfo::*;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid .PKGINFO")]
    InvalidPkginfo(#[from] PkgInfoError),

    #[error("I/O error occurred")]
    Io(#[from] io::Error),

    #[error("no .PKGINFO found in .apk")]
    MissingPkginfo,

    #[error("no signatures found in .apk")]
    MissingSignature,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct Package {
    #[garde(dive)]
    signs: Vec<SignatureInfo>,

    #[garde(dive)]
    #[serde(flatten)]
    pkginfo: PkgInfo,

    #[garde(skip)]
    #[serde(default)]
    scripts: Vec<PkgScript>,

    #[garde(dive)]
    files: Vec<FileInfo>,
}

// The package file consists of three gzip streams concatenated together, each
// containing a TAR segment:
//
// 1. digital signature (`.SIGN.RSA.<key_name>.rsa.pub`)
// 2. control segment (`.PKGINFO` and install scripts)
// 3. package data
impl Package {
    /// Loads a `Package` from the given buffered reader over an APKv2 file.
    ///
    /// Example:
    /// ```no_run
    /// # use std::fs::File;
    /// # use std::io::BufReader;
    /// use alpkit::package::Package;
    ///
    /// let file = File::open("example-1.0-r0.apk").map(BufReader::new).unwrap();
    /// let pkg = Package::load(file).unwrap();
    /// ```
    pub fn load<R: BufRead>(mut reader: R) -> Result<Self, Error> {
        let mut pkg = Self::load_without_files(&mut reader)?;
        pkg.files = Self::read_data(&mut reader)?;

        Ok(pkg)
    }

    /// Loads a `Package` from the given buffered reader over an APKv2 file, as
    /// the `load` method, but doesn't read the package data segment (files) -
    /// the `files` field will be empty. This is the preferred method if you
    /// don't need files, because it's much faster for bigger packages.
    pub fn load_without_files<R: BufRead>(mut reader: R) -> Result<Self, Error> {
        let signs = Self::read_signatures(&mut reader)?;
        let (pkginfo, scripts) = Self::read_control(&mut reader)?;

        Ok(Self {
            signs,
            pkginfo,
            scripts,
            files: vec![],
        })
    }

    pub fn signatures(&self) -> Iter<SignatureInfo> {
        self.signs.iter()
    }

    pub fn pkginfo(&self) -> &PkgInfo {
        &self.pkginfo
    }

    pub fn scripts(&self) -> Iter<PkgScript> {
        self.scripts.iter()
    }

    pub fn files_metadata(&self) -> Iter<FileInfo> {
        self.files.iter()
    }

    fn read_signatures<R: BufRead>(reader: &mut R) -> Result<Vec<SignatureInfo>, Error> {
        let mut archive = Archive::new(GzDecoder::new(reader));

        let mut signs: Vec<SignatureInfo> = Vec::with_capacity(1);
        for entry in archive.entries()? {
            if let Some(sign) = SignatureInfo::from_filename(&entry?.path()?) {
                signs.push(sign);
            }
        }
        if signs.is_empty() {
            bail!(Error::MissingSignature);
        }
        Ok(signs)
    }

    fn read_control<R: BufRead>(reader: &mut R) -> Result<(PkgInfo, Vec<PkgScript>), Error> {
        let mut archive = Archive::new(GzDecoder::new(reader));

        let mut pkginfo: Option<PkgInfo> = None;
        let mut scripts: Vec<PkgScript> = vec![];

        for entry in archive.entries()? {
            let mut entry = entry?;

            match entry.path_bytes().as_ref() {
                b".PKGINFO" => {
                    let mut buf = String::new();
                    entry.read_to_string(&mut buf)?;

                    pkginfo = Some(PkgInfo::parse(&buf)?);
                }
                path => {
                    let name = str::from_utf8(&path[1..]).unwrap_or("");
                    if let Ok(script) = PkgScript::from_str(name) {
                        scripts.push(script);
                    }
                }
            };
        }

        if let Some(pkginfo) = pkginfo {
            Ok((pkginfo, scripts))
        } else {
            bail!(Error::MissingPkginfo)
        }
    }

    fn read_data<R: BufRead>(reader: &mut R) -> io::Result<Vec<FileInfo>> {
        let mut archive = Archive::new(GzDecoder::new(reader));
        let entries = archive.entries()?;

        entries.map(|entry| FileInfo::try_from(entry?)).collect()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct SignatureInfo {
    /// Currently only `RSA` is supported.
    #[garde(ascii)]
    pub alg: String,

    #[garde(ascii, pattern(regex::FILE_NAME))]
    pub keyname: String,
}

impl SignatureInfo {
    fn from_filename(path: &Path) -> Option<Self> {
        path.to_string_lossy()
            .strip_prefix(".SIGN.")
            .and_then(|s| s.split_once('.'))
            .map(|t| SignatureInfo {
                alg: t.0.to_owned(),
                keyname: t.1.to_owned(),
            })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PkgScript {
    PreInstall,
    PostInstall,
    PreUpgrade,
    PostUpgrade,
    PreDeinstall,
    PostDeinstall,
}

impl FromStr for PkgScript {
    type Err = de::value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PkgScript::deserialize(de::value::StrDeserializer::new(s))
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
#[path = "mod.test.rs"]
mod test;
