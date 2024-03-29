use assert_json_diff::assert_json_eq;
use indoc::indoc;
use serde_json::json;

use crate::internal::test_utils::{assert, assert_from_to_json, assert_let, dependency, S};

use super::*;

fn sample_pkginfo() -> PkgInfo {
    PkgInfo {
        pkgname: S!("sample"),
        pkgver: S!("1.2.3-r2"),
        pkgdesc: S!("A sample aport for testing"),
        url: S!("https://example.org/sample"),
        builddate: 1671582086,
        packager: S!("Jakub Jirutka <jakub@jirutka.cz>"),
        size: 696320,
        arch: S!("x86_64"),
        origin: S!("sample"),
        commit: Some(S!("994dcb4685405e710a1e599cff82d2e45ec9daae")),
        maintainer: Some(S!("Jakub Jirutka <jakub@jirutka.cz>")),
        license: S!("ISC and BSD-2-Clause and BSD-3-Clause"),
        triggers: vec![S!("/bin/*"), S!("/usr/bin/*")],
        depends: vec![
            dependency("ruby>=3.0"),
            dependency("so:libc.musl-x86_64.so.1"),
        ],
        conflicts: vec![dependency("sample-legacy")],
        install_if: vec![dependency("sample=1.2.3-r2"), dependency("bar")],
        provides: vec![dependency("cmd:sample=1.2.3-r2")],
        provider_priority: Some(10),
        datahash: S!("4c36284c04dd1e18e4df59b4bc873fd89b6240861b925cac59341cc66e36d94b"),
        ..Default::default()
    }
}

#[test]
fn pkginfo_parse() {
    let input = indoc! {"
        # Generated by abuild 3.10.0-r0
        # using fakeroot version 1.29
        # Wed Dec 21 00:21:26 UTC 2022
        pkgname = sample
        pkgver = 1.2.3-r2
        pkgdesc = A sample aport for testing
        url = https://example.org/sample
        builddate = 1671582086
        packager = Jakub Jirutka <jakub@jirutka.cz>
        size = 696320
        arch = x86_64
        origin = sample
        commit = 994dcb4685405e710a1e599cff82d2e45ec9daae
        maintainer = Jakub Jirutka <jakub@jirutka.cz>
        license = ISC and BSD-2-Clause and BSD-3-Clause
        triggers = /bin/* /usr/bin/*
        provider_priority = 10
        depend = ruby>=3.0
        depend = !sample-legacy
        install_if = sample=1.2.3-r2 bar
        # automatically detected:
        provides = cmd:sample=1.2.3-r2
        depend = so:libc.musl-x86_64.so.1
        datahash = 4c36284c04dd1e18e4df59b4bc873fd89b6240861b925cac59341cc66e36d94b
    "};

    assert!(PkgInfo::parse(input).unwrap() == sample_pkginfo());
}

#[test]
fn parse_key_value_with_missing_equals() {
    let input = indoc! {"
        pkgname = foo
        depend bar
        depend = baz
    "};
    let mut parsed = parse_key_value(input);

    assert_let!(Some(Ok(("pkgname", "foo"))) = parsed.next());

    assert_let!(Some(Err(PkgInfoError::Syntax(2, line))) = parsed.next());
    assert!(line == "depend bar");

    assert_let!(Some(Ok(("depend", "baz"))) = parsed.next());

    assert!(parsed.next().is_none());
}

#[test]
fn pkginfo_json() {
    assert_from_to_json!(
        sample_pkginfo(),
        json!({
            "maintainer": "Jakub Jirutka <jakub@jirutka.cz>",
            "pkgname": "sample",
            "pkgver": "1.2.3-r2",
            "pkgdesc": "A sample aport for testing",
            "url": "https://example.org/sample",
            "arch": "x86_64",
            "license": "ISC and BSD-2-Clause and BSD-3-Clause",
            "depends": {
                "ruby": ">= 3.0",
                "so:libc.musl-x86_64.so.1": "*"
            },
            "conflicts": {
                "sample-legacy": "*"
            },
            "install_if": {
                "sample": "= 1.2.3-r2",
                "bar": "*"
            },
            "provides": {
                "cmd:sample": "= 1.2.3-r2"
            },
            "provider_priority": 10,
            "replaces": {},
            "triggers": [
                "/bin/*",
                "/usr/bin/*"
            ],
            "origin": "sample",
            "commit": "994dcb4685405e710a1e599cff82d2e45ec9daae",
            "builddate": 1671582086,
            "packager": "Jakub Jirutka <jakub@jirutka.cz>",
            "size": 696320,
            "datahash": "4c36284c04dd1e18e4df59b4bc873fd89b6240861b925cac59341cc66e36d94b"
        }),
    );
}

#[test]
fn pkginfo_json_with_dependency_arrays() {
    let pkginfo_json = json!({
        "maintainer": "Jakub Jirutka <jakub@jirutka.cz>",
        "pkgname": "sample",
        "pkgver": "1.2.3-r2",
        "pkgdesc": "A sample aport for testing",
        "url": "https://example.org/sample",
        "arch": "x86_64",
        "license": "ISC and BSD-2-Clause and BSD-3-Clause",
        "depends": [
            "ruby>=3.0",
            "so:libc.musl-x86_64.so.1"
        ],
        "conflicts": [
            "sample-legacy"
        ],
        "install_if": [
            "sample=1.2.3-r2",
            "bar"
        ],
        "provides": [
            "cmd:sample=1.2.3-r2"
        ],
        "provider_priority": 10,
        "replaces": [],
        "triggers": [
            "/bin/*",
            "/usr/bin/*"
        ],
        "origin": "sample",
        "commit": "994dcb4685405e710a1e599cff82d2e45ec9daae",
        "builddate": 1671582086,
        "packager": "Jakub Jirutka <jakub@jirutka.cz>",
        "size": 696320,
        "datahash": "4c36284c04dd1e18e4df59b4bc873fd89b6240861b925cac59341cc66e36d94b"
    })
    .to_string();

    assert_json_eq!(
        serde_json::from_str::<PkgInfo>(&pkginfo_json).unwrap(),
        sample_pkginfo()
    );
}
