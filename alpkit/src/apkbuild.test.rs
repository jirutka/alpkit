use indoc::indoc;
use serde_json::json;

use super::*;
use crate::internal::test_utils::{assert, assert_from_to_json, assert_let, S};

fn valid_apkbuild() -> Apkbuild {
    Apkbuild {
        maintainer: Some(S!("Jakub Jirutka <jakub@jirutka.cz>")),
        contributors: vec![
            S!("Francesco Colista <fcolista@alpinelinux.org>"),
            S!("Natanael Copa <ncopa@alpinelinux.org>")
        ],
        pkgname: S!("sample"),
        pkgver: S!("1.2.3"),
        pkgrel: 2,
        pkgdesc: S!("A sample aport for testing"),
        url: S!("https://example.org/sample"),
        arch: ARCH_ALL
            .iter()
            .filter(|s| !matches!(**s, "riscv64" | "s390x"))
            .map(ToString::to_string)
            .collect(),
        license: S!("ISC and BSD-2-Clause and BSD-3-Clause"),
        depends: Dependencies::parse([
            "ruby>=3.0",
            "!sample-legacy",
        ]).unwrap(),
        makedepends: Dependencies::parse([
            "openssl-dev>3",
            "zlib-dev",
        ]).unwrap(),
        makedepends_build: Dependencies::default(),
        makedepends_host: Dependencies::default(),
        checkdepends: Dependencies::parse([
            "ruby-rspec",
        ]).unwrap(),
        install_if: Dependencies::default(),
        pkgusers: vec![],
        pkggroups: vec![],
        provides: Dependencies::parse([
            "sample2=1.2.3-r2",
        ]).unwrap(),
        provider_priority: Some(100),
        pcprefix: None,
        sonameprefix: Some(S!("smpl")),
        replaces: Dependencies::parse([
            "sample2",
        ]).unwrap(),
        replaces_priority: None,
        install: vec![S!("sample.post-install"), S!("sample.post-upgrade")],
        triggers: vec![S!("sample.trigger=/usr/share/sample/*")],
        subpackages: vec![
            S!("sample-doc"),
            S!("sample-dev"),
        ],
        source: vec![
            Source::new("sample-1.2.3.tar.gz", "https://example.org/sample/sample-1.2.3.tar.gz", "54286070812a47b629f68757046d3c9a1bdd2b5d1c3b84a5c8e4cb92f1331afa745443f7238175835d8cfbe5b8dd442e00c75c3a5b5b8f8efd8d2ec8f636dad4"),
            Source::new("sample.initd", "sample.initd", "b512bcb8bae11853a3006e2122d7e652806d4bf2234638d8809fd823375b5b0bd590f7d6a90412baffcc3b7b6a0f197a10986728a70f24fe628f91bfb651d266"),
            Source::new("sample.confd", "sample.confd", "6eda39920cccb1238b104bb90ac4be2c32883897c72363560d8d39345819cdeff535680e78396052b2b8f981e169ad9b3c30da724def80a1501785d82ce7fa25")
        ],
        options: vec![S!("!check")],
        secfixes: vec![
            Secfix::new("1.2.3-r2", vec![S!("CVE-2022-12347"), S!("CVE-2022-12346")]),
            Secfix::new("1.2.0-r0", vec![S!("CVE-2021-12345")]),
        ]
    }
}

fn invalid_apkbuild() -> Apkbuild {
    Apkbuild {
        maintainer: Some(S!("Invalid em@il")),
        contributors: vec![
            S!("invalid@form.at"),
        ],
        pkgname: S!("samp/le"),
        pkgver: S!("1.2.3+alpha"),
        pkgdesc: S!("A sample aport\nfor testing"),
        url: S!("https://example.org/sample space"),
        arch: vec![
            S!("all spaces"),
            S!("!riscv64"),
            S!("s390x")
        ],
        depends: Dependencies::parse([
            "ruby#>=3.0",
            "!!sample-legacy",
        ]).unwrap(),
        makedepends: Dependencies::parse([
            "openssl!dev>3",
            "zlib-dev",
        ]).unwrap(),
        checkdepends: Dependencies::parse([
            "ruby&rspec",
        ]).unwrap(),
        provides: Dependencies::parse([
            "sample2=1.2.3-rrr2",
        ]).unwrap(),
        pcprefix: Some(S!("wrong prefix")),
        replaces: Dependencies::parse([
            "sample2?",
        ]).unwrap(),
        subpackages: vec![
            S!("sample doc"),
            S!("sample^dev"),
        ],
        source: vec![
            Source::new("sample 1.2.3 tar.gz", "torrent://example.org/sample/sample-1.2.3.tar.gz", "54286070812a47b629f68757046d3c9a1bdd2b5d1c3b84a5c8e4cb92f1331afa745443f7238175835d8cfbe5b8dd442e00c75c3a5b5b8f8efd8d2ec8f636dad4"),
            Source::new("sample.initd", "/etc/init.d/sample", "123"),
        ],
        options: vec![S!("check me")],
        secfixes: vec![
            Secfix::new("1.2.3", vec![S!("CVE-2022-12347"), S!("CVE-2022-12346")]),
            Secfix::new("1.2.0-ra", vec![S!("CVE-2021-12345")]),
        ],
        ..Default::default()
    }
}

#[test]
#[cfg(feature = "validate")]
fn apkbuild_validate_valid() {
    assert!(valid_apkbuild().validate(&()).is_ok())
}

#[test]
#[cfg(feature = "validate")]
fn apkbuild_validate_invalid() {
    assert_let!(Err(e) = invalid_apkbuild().validate(&()));
    assert!(e.flatten().len() == 24);
}

#[test]
fn read_apkbuild() {
    let fixture = Path::new("../fixtures/aports/sample/APKBUILD");
    assert!(ApkbuildReader::new().read_apkbuild(fixture).unwrap() == valid_apkbuild());
}

#[test]
#[rustfmt::skip]
fn test_parse_maintainer() {
    for (input, expected) in [
        ("\n# sample\n# Maintainer: Kevin Flynn\n", Some("Kevin Flynn")            ),
        ("#   Maintainer:  Kevin Flynn  \n"       , Some("Kevin Flynn")            ),
        ("# Maintainer: Flynn <flynn@encom.com>\n", Some("Flynn <flynn@encom.com>")),
        ("#Maintainer: No One\n"                  , None                           ),
        ("# Some comment\n\npkgname=sample\n"     , None                           ),
    ] {
        assert!(parse_maintainer(input) == expected);
    }
}

#[test]
#[rustfmt::skip]
fn test_parse_contributors() {
    for (input, expected) in [
        ("\n# sample\n#  Contributor: Kevin Flynn\n"         , vec!["Kevin Flynn"]),
        ("# Contributor: KF\n# Contributor: AB\n"            , vec!["KF", "AB"]   ),
        ("# Contributor: KF\n\n# sample\n# Contributor: AB\n", vec!["KF", "AB"]   ),
        ("# Maintainer: No One"                              , vec![]             ),
    ] {
        assert!(parse_contributors(input).collect::<Vec<_>>() == expected);
    }
}

#[test]
fn test_parse_secfixes() {
    let input = indoc! {"
        # Maintainer: me
        pkgname=sample

        # secfixes:
        #   1.1-r0:
        #   - CVE-2022-1236  # comment
        #   1.0-r0:
        #     - CVE-2022-1235
        #      -  CVE-2022-1234
        #
    "};
    let expected = vec![
        Secfix::new("1.1-r0", vec!["CVE-2022-1236".to_owned()]),
        Secfix::new(
            "1.0-r0",
            vec!["CVE-2022-1235".to_owned(), "CVE-2022-1234".to_owned()],
        ),
    ];
    assert!(parse_secfixes(input).unwrap() == expected);

    let input = indoc! {"
        # Maintainer: me
        pkgname=sample
    "};
    assert!(parse_secfixes(input).unwrap() == vec![]);

    let input = indoc! {"
        # secfixes:
        #   - CVE-2022-1236
        #   - CVE-2022-1235
    "};
    assert_let!(Err(err @ Error::MalformedSecfixes(..)) = parse_secfixes(input));
    assert!(format!("{}", err) == "syntax error in secfixes on line 2: '- CVE-2022-1236'");

    let input = indoc! {"
        # secfixes:
        #   1.2-r0:
        #     - CVE-2022-1235
        #   1.1-r0
        #     - CVE-2022-1234
    "};
    assert_let!(Err(err @ Error::MalformedSecfixes(..)) = parse_secfixes(input));
    assert!(format!("{}", err) == "syntax error in secfixes on line 4: '1.1-r0'");
}

#[test]
fn test_decode_source_and_sha512sums() {
    let source = indoc! {"
        https://example.org/sample-1.2.3.tar.gz
        bar-1.2.tar.gz::https://example.org/bar/1.2.tar.gz
        sample.initd
    "};
    let sha512sums = indoc! {"
        1d468dcfa9bbd348b8a5dc514ac1428a789e73a92384c039b73a51ce376785f74bf942872c5594a9fcda6bbf44758bd727ce15ac2395f1aa989c507014647dcc sample-1.2.3.tar.gz
        0acd8bf9aedeabeef590909c83ad9057063b4d3165fe5e0b0ff2205df6e0d1b97f3fcfd27384a55b4816bbe975e93a737e58df9c6ee01baf7e46ceaabc43c64a bar-1.2.tar.gz
        ee10a5687740dde0c3d18d8b3555f49fcdc6abfc0a3bc2de1de3be0e99951a346fe8027d916aab73071ecd4e2c50871e7c867aca3a7a0fd16e3374c5caed1c57 sample.initd
    "};
    let expected = vec!(
        Source::new("sample-1.2.3.tar.gz", "https://example.org/sample-1.2.3.tar.gz", "1d468dcfa9bbd348b8a5dc514ac1428a789e73a92384c039b73a51ce376785f74bf942872c5594a9fcda6bbf44758bd727ce15ac2395f1aa989c507014647dcc"),
        Source::new("bar-1.2.tar.gz", "https://example.org/bar/1.2.tar.gz", "0acd8bf9aedeabeef590909c83ad9057063b4d3165fe5e0b0ff2205df6e0d1b97f3fcfd27384a55b4816bbe975e93a737e58df9c6ee01baf7e46ceaabc43c64a"),
        Source::new("sample.initd", "sample.initd", "ee10a5687740dde0c3d18d8b3555f49fcdc6abfc0a3bc2de1de3be0e99951a346fe8027d916aab73071ecd4e2c50871e7c867aca3a7a0fd16e3374c5caed1c57"),
    );

    assert!(decode_source_and_sha512sums(source, sha512sums).unwrap() == expected);

    let sha512sums = indoc! {"
        1d468dcfa9bbd348b8a5dc514ac1428a789e73a92384c039b73a51ce376785f74bf942872c5594a9fcda6bbf44758bd727ce15ac2395f1aa989c507014647dcc sample-1.2.3.tar.gz
        ee10a5687740dde0c3d18d8b3555f49fcdc6abfc0a3bc2de1de3be0e99951a346fe8027d916aab73071ecd4e2c50871e7c867aca3a7a0fd16e3374c5caed1c57 sample.initd
    "};

    assert_let!(Err(err @ Error::MissingChecksum(..)) = decode_source_and_sha512sums(source, sha512sums));
    assert!(
        format!("{err}").contains("bar-1.2.tar.gz"),
        "error message should contain name of the missing checksum"
    );
}

#[test]
fn apkbuild_json() {
    assert_from_to_json!(
        valid_apkbuild(),
        json!({
            "maintainer": "Jakub Jirutka <jakub@jirutka.cz>",
            "contributors": [
                "Francesco Colista <fcolista@alpinelinux.org>",
                "Natanael Copa <ncopa@alpinelinux.org>"
            ],
            "pkgname": "sample",
            "pkgver": "1.2.3",
            "pkgrel": 2,
            "pkgdesc": "A sample aport for testing",
            "url": "https://example.org/sample",
            "arch": [ "aarch64", "armhf", "armv7", "ppc64le", "x86", "x86_64" ],
            "license": "ISC and BSD-2-Clause and BSD-3-Clause",
            "depends": {
                "ruby": ">= 3.0",
                "sample-legacy": "!"
            },
            "makedepends": {
                "openssl-dev": "> 3",
                "zlib-dev": "*"
            },
            "makedepends_build": {},
            "makedepends_host": {},
            "checkdepends": {
                "ruby-rspec": "*"
            },
            "install_if": {},
            "pkgusers": [],
            "pkggroups": [],
            "provides": {
                "sample2": "= 1.2.3-r2"
            },
            "provider_priority": 100,
            "sonameprefix": "smpl",
            "replaces": {
                "sample2": "*"
            },
            "install": [ "sample.post-install", "sample.post-upgrade" ],
            "triggers": [ "sample.trigger=/usr/share/sample/*" ],
            "subpackages": [ "sample-doc", "sample-dev" ],
            "sources": [{
                "name": "sample-1.2.3.tar.gz",
                "uri": "https://example.org/sample/sample-1.2.3.tar.gz",
                "checksum": "54286070812a47b629f68757046d3c9a1bdd2b5d1c3b84a5c8e4cb92f1331afa745443f7238175835d8cfbe5b8dd442e00c75c3a5b5b8f8efd8d2ec8f636dad4"
            }, {
                "name": "sample.initd",
                "uri": "sample.initd",
                "checksum": "b512bcb8bae11853a3006e2122d7e652806d4bf2234638d8809fd823375b5b0bd590f7d6a90412baffcc3b7b6a0f197a10986728a70f24fe628f91bfb651d266"
            }, {
                "name": "sample.confd",
                "uri": "sample.confd",
                "checksum": "6eda39920cccb1238b104bb90ac4be2c32883897c72363560d8d39345819cdeff535680e78396052b2b8f981e169ad9b3c30da724def80a1501785d82ce7fa25"
            }],
            "options": [ "!check" ],
            "secfixes": {
                "1.2.3-r2": [ "CVE-2022-12347", "CVE-2022-12346" ],
                "1.2.0-r0": [ "CVE-2021-12345" ]
            }
        }),
    );
}
