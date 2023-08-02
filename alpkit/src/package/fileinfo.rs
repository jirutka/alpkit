use std::borrow::Cow;
use std::error;
use std::io::{self, Read};
use std::path::PathBuf;

use cfg_iif::cfg_iif;
use serde::{de, Deserialize, Serialize};

use crate::internal::key_value_vec_map::{self, KeyValueLike};
use crate::internal::macros::bail;

////////////////////////////////////////////////////////////////////////////////

/// This struct represents a file (in general sense, so also a directory) in
/// an APK package archive.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FileInfo {
    /// An absolute path of the file.
    pub path: PathBuf,

    /// The type of the file.
    #[serde(rename = "type")]
    pub file_type: FileType,

    /// If the entry is a symlink or hardlink, then this is a path the link
    /// points to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_target: Option<PathBuf>,

    /// The name of the system user who owns the file.
    #[serde(default = "root", skip_serializing_if = "is_root")]
    pub uname: String,

    /// The name of the sytem group that owns the file.
    #[serde(default = "root", skip_serializing_if = "is_root")]
    pub gname: String,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// The file mode bits (permissions).
    #[serde(
        deserialize_with = "deserialize_mode",
        serialize_with = "serialize_mode"
    )]
    pub mode: u32,

    /// The device ID (combined major and minor ID), if this file is a block or
    /// character device, otherwise `0`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub device: u64,

    /// The SHA-1 checksum of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,

    /// Extended file attributes (xattr) of the entry.
    #[serde(
        default,
        with = "key_value_vec_map",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub xattrs: Vec<Xattr>,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            path: PathBuf::new(),
            file_type: FileType::Regular,
            link_target: None,
            uname: "root".to_owned(),
            gname: "root".to_owned(),
            size: None,
            mode: 0o644,
            device: 0,
            digest: None,
            xattrs: vec![],
        }
    }
}

impl<'a, R: Read> TryFrom<tar::Entry<'a, R>> for FileInfo {
    type Error = io::Error;

    fn try_from(mut entry: tar::Entry<'a, R>) -> Result<Self, Self::Error> {
        use crate::internal::tar_ext::*;

        let header = entry.header();
        let is_dir = header.entry_type().is_dir();

        Ok(FileInfo {
            path: PathBuf::from("/").join(entry.path()?),
            file_type: header.entry_type().try_into()?,
            link_target: entry.link_name()?.map(Cow::into_owned),
            uname: header
                .username()
                .map_err(io_error_other)?
                .unwrap_or("root")
                .to_owned(),
            gname: header
                .groupname()
                .map_err(io_error_other)?
                .unwrap_or("root")
                .to_owned(),
            size: (!is_dir).then_some(entry.size()),
            mode: header.mode()?,
            device: header.device()?.unwrap_or(0),
            xattrs: entry.xattrs()?.map(Xattr::from).collect(),
            digest: entry.apk_checksum()?.map(str::to_owned),
        })
    }
}

fn root() -> String {
    "root".to_owned()
}

fn is_root(name: &String) -> bool {
    name == "root"
}

fn is_zero(num: &u64) -> bool {
    num == &0
}

fn serialize_mode<S: serde::Serializer>(value: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("0{value:o}"))
}

fn deserialize_mode<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    let s: &str = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(s, 8)
        .map_err(|_| de::Error::custom(format!("invalid value: `{s}`, expected octal number")))
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum FileType {
    /// Regular file
    #[serde(rename = "r")]
    Regular,

    /// Hard link
    #[serde(rename = "H")]
    Link,

    /// Symbolic link
    #[serde(rename = "l")]
    Symlink,

    /// Character device
    #[serde(rename = "c")]
    Char,

    /// Block device
    #[serde(rename = "b")]
    Block,

    /// Directory
    #[serde(rename = "d")]
    Directory,

    /// Named pipe (fifo)
    #[serde(rename = "p")]
    Fifo,
}

impl TryFrom<tar::EntryType> for FileType {
    type Error = io::Error;

    fn try_from(value: tar::EntryType) -> Result<Self, Self::Error> {
        use tar::EntryType;

        let file_type = match value {
            EntryType::Regular | EntryType::Continuous => FileType::Regular,
            EntryType::Link => FileType::Link,
            EntryType::Symlink => FileType::Symlink,
            EntryType::Char => FileType::Char,
            EntryType::Block => FileType::Block,
            EntryType::Directory => FileType::Directory,
            EntryType::Fifo => FileType::Fifo,
            _ => bail!(io_error_other(format!(
                "unexpected tar entry type: {value:?}"
            ))),
        };
        Ok(file_type)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Xattr {
    pub name: String,
    pub value: Vec<u8>,
}

impl From<(&str, &[u8])> for Xattr {
    fn from(pair: (&str, &[u8])) -> Self {
        Xattr {
            name: pair.0.to_owned(),
            value: pair.1.to_owned(),
        }
    }
}

impl<'a> KeyValueLike<'a> for Xattr {
    type Key = &'a str;
    type Value = String;

    #[cfg(feature = "base64")]
    type Err = base64::DecodeError;
    #[cfg(not(feature = "base64"))]
    type Err = std::convert::Infallible;

    fn from_key_value(key: Self::Key, value: Self::Value) -> Result<Self, Self::Err> {
        Ok(Xattr {
            name: key.to_owned(),
            value: cfg_iif!(feature = "base64" {
                base64::decode(value)?
            } else {
                value.into()
            }),
        })
    }

    fn to_key_value(&'a self) -> (Self::Key, Self::Value) {
        let value = cfg_iif!(feature = "base64" {
            base64::encode(&self.value)
        } else {
            "".to_string()
        });

        (&self.name, value)
    }
}

////////////////////////////////////////////////////////////////////////////////

fn io_error_other<E>(error: E) -> io::Error
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, error.into())
}
