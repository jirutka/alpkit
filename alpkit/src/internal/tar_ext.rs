use std::io::{self, Read};
use std::iter::FilterMap;

use tar::{Entry, PaxExtension, PaxExtensions};

type Xattrs<'a> =
    FilterMap<PaxExtensions<'a>, fn(io::Result<PaxExtension<'a>>) -> Option<(&'a str, &'a [u8])>>;

pub(crate) trait TarEntryExt<'a> {
    /// Returns the SHA-1 checksum of the entry's contents, if present.
    /// This is an apk-specific extension and it's used only on regular files.
    fn apk_checksum(&mut self) -> io::Result<Option<&str>>;

    /// Returns extended file attributes (xattr) of the entry, if present.
    fn xattrs(&mut self) -> io::Result<Xattrs>;
}

impl<'a, R: Read> TarEntryExt<'a> for Entry<'a, R> {
    fn apk_checksum(&mut self) -> io::Result<Option<&str>> {
        if let Some(exts) = self.pax_extensions()? {
            for ext in exts.flatten() {
                if ext.key_bytes() == b"APK-TOOLS.checksum.SHA1" {
                    return ext
                        .value()
                        .map(Some)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
                }
            }
        }
        Ok(None)
    }

    fn xattrs(&mut self) -> io::Result<Xattrs> {
        let exts = self.pax_extensions()?;

        Ok(exts
            .unwrap_or_else(|| PaxExtensions::new(b""))
            .filter_map(|ext| {
                if let Ok(ext) = ext {
                    if let Some(name) = ext.key().unwrap_or("").strip_prefix("SCHILY.xattr.") {
                        return Some((name, ext.value_bytes()));
                    }
                }
                None
            }))
    }
}

pub(crate) trait TarHeaderExt {
    /// Returns the device ID (combined major and minor ID), if this entry
    /// is a device file.
    fn device(&self) -> io::Result<Option<u64>>;
}

impl TarHeaderExt for tar::Header {
    fn device(&self) -> io::Result<Option<u64>> {
        let major = self.device_major();
        let minor = self.device_minor();

        // XXX: device_major() and device_minor() returns Err on a regular file
        // or directory in some APK files with an error:
        // "numeric field was not a number: when getting device_major for ...".
        if let (Ok(Some(major)), Ok(Some(minor))) = (major, minor) {
            Ok(Some(makedev(major, minor)))
        } else {
            Ok(None)
        }
    }
}

// This has been copied from the libc crate.
#[allow(clippy::identity_op)]
const fn makedev(major: u32, minor: u32) -> u64 {
    let major = major as u64;
    let minor = minor as u64;
    let mut dev = 0;
    dev |= (major & 0x00000fff) << 8;
    dev |= (major & 0xfffff000) << 32;
    dev |= (minor & 0x000000ff) << 0;
    dev |= (minor & 0xffffff00) << 12;
    dev
}

#[cfg(test)]
#[path = "tar_ext.test.rs"]
mod test;
