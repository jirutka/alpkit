use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use super::*;
use crate::dependency::Dependencies;
use crate::internal::test_utils::{assert, assert_let, S};
use fileinfo::FileType;

#[test]
fn signature_info_from_filename() {
    let input = PathBuf::from(".SIGN.RSA.alpine-devel@lists.alpinelinux.org-6165ee59.rsa.pub");
    let expected = SignatureInfo {
        alg: S!("RSA"),
        keyname: S!("alpine-devel@lists.alpinelinux.org-6165ee59.rsa.pub"),
    };
    assert!(SignatureInfo::from_filename(&input).unwrap() == expected);

    let input = PathBuf::from(".SIGNA.RSA.alpine-devel@lists.alpinelinux.org-6165ee59.rsa.pub");
    assert!(SignatureInfo::from_filename(&input) == None);
}

#[test]
fn package_load() {
    let signature = SignatureInfo {
        alg: S!("RSA"),
        keyname: S!("alpine-devel@lists.alpinelinux.org-6165ee59.rsa.pub"),
    };
    let pkginfo = PkgInfo {
        maintainer: Some(S!("Jakub Jirutka <jakub@jirutka.cz>")),
        pkgname: S!("rssh"),
        pkgver: S!("2.3.4-r3"),
        pkgdesc: S!("Restricted shell for use with OpenSSH, allowing only scp, sftp, and/or rsync"),
        url: S!("http://www.pizzashack.org/rssh/"),
        arch: S!("x86_64"),
        license: S!("BSD-2-Clause"),
        depends: Dependencies::parse(["openssh", "/bin/sh", "so:libc.musl-x86_64.so.1"]).unwrap(),
        conflicts: Dependencies::default(),
        install_if: Dependencies::default(),
        provides: Dependencies::parse(["cmd:rssh=2.3.4-r3"]).unwrap(),
        provider_priority: None,
        replaces: Dependencies::default(),
        replaces_priority: None,
        triggers: vec![],
        origin: S!("rssh"),
        commit: Some(S!("c57128b0e49d551220aff88af0f1487d80cdccf8")),
        builddate: 1666619671,
        packager: S!("Buildozer <alpine-devel@lists.alpinelinux.org>"),
        size: 86016,
        datahash: S!("db62becd32465838640f39bd35854bd03e9b5e56b1ea8574e9188c3910121477"),
    };
    let scripts = vec![&PkgScript::PostInstall, &PkgScript::PostDeinstall];

    let files = vec![
        dir("/etc", 0o755),
        file(
            "/etc/rssh.conf.default",
            0o644,
            1791,
            "d371f2e400ee8b8c2f186801514b146668373666",
        ),
        dir("/usr/", 0o755),
        dir("/usr/bin/", 0o755),
        file(
            "/usr/bin/rssh",
            0o755,
            26584,
            "b0b8f3afe3ced5ed9bf9acef9eeaf760dcfccf6d",
        ),
        dir("/usr/lib/", 0o755),
        dir("/usr/lib/rssh/", 0o755),
        file(
            "/usr/lib/rssh/rssh_chroot_helper",
            0o4755,
            26560,
            "509736459697e360eb4bdc63d25c15c7f3903cd2",
        ),
    ];
    let files = files.iter().collect::<Vec<_>>();

    let reader = read_fixture("../fixtures/apk/rssh-2.3.4-r3.apk");

    assert_let!(Ok(pkg) = Package::load(reader));
    assert!(pkg.signatures().collect::<Vec<_>>() == vec![&signature]);
    assert!(pkg.scripts().collect::<Vec<_>>() == scripts);
    assert!(pkg.pkginfo() == &pkginfo);
    assert!(pkg.files_metadata().collect::<Vec<_>>() == files);
}

fn read_fixture(path: &str) -> BufReader<File> {
    let file = File::open(path).unwrap_or_else(|_| panic!("Fixture file `{}` not found", &path));
    BufReader::new(file)
}

fn dir(path: &str, mode: u32) -> FileInfo {
    FileInfo {
        path: PathBuf::from(path),
        file_type: FileType::Directory,
        mode,
        ..Default::default()
    }
}

fn file(path: &str, mode: u32, size: u64, digest: &str) -> FileInfo {
    FileInfo {
        path: PathBuf::from(path),
        file_type: FileType::Regular,
        size: Some(size),
        mode,
        digest: Some(digest.to_owned()),
        ..Default::default()
    }
}
