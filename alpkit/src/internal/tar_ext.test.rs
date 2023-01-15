use std::fs::File;

use super::*;
use crate::internal::test_utils::assert_let;

#[test]
fn entry_apk_checksum() {
    let mut tar = tar::Archive::new(open_fixture("tar/apk.tar"));
    let mut entry = last_entry(&mut tar);

    assert_let!(Ok(Some("4cfc8ab36752dafcea42ee50fec8b8608a340a62")) = entry.apk_checksum());
}

#[test]
fn entry_xattrs() {
    let mut tar = tar::Archive::new(open_fixture("tar/xattrs.tar"));
    let mut entry = last_entry(&mut tar);

    let xattrs = entry.xattrs().unwrap().collect::<Vec<_>>();

    assert_let!(Some(("user.pax.flags", b"epm")) = xattrs.first());
}

#[test]
fn header_device() {
    let mut tar = tar::Archive::new(open_fixture("tar/device.tar"));
    let entry = last_entry(&mut tar);

    assert_let!(Ok(Some(259)) = entry.header().device());
}

fn open_fixture(path: &str) -> File {
    let path = format!("../fixtures/{}", path);
    File::open(&path).unwrap_or_else(|_| panic!("Fixture file `{}` not found", &path))
}

fn last_entry<T: Read + Sized>(archive: &mut tar::Archive<T>) -> Entry<'_, T> {
    archive.entries().unwrap().last().unwrap().unwrap()
}
