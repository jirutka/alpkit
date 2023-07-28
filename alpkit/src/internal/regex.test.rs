use super::*;

#[test]
fn email_valid() {
    for input in &[
        "Kevin Flynn <kevin.flynn@encom.com>",
        "<Kevin.Flynn@Encom.COM>",
        "Karel Čapek <karel.capek@history.cz>",
        "Tony Stark, Jr <tony@stark.com>",
        "Tony (Iron Man) <tony@stark.com>",
        "Foo<!def!xyz%abc@example.com>",
    ] {
        assert!(EMAIL.is_match(input), "{input}");
    }
}

#[test]
fn email_invalid() {
    for input in &[
        "kevin flynn@encom.com",
        "Kevin Flynn <kevin flynn@encom.com>",
        "Kevin\nFlynn <kevin.flynn@encom.com>",
        "\"Kevin Flynn <kevin.flynn@encom.com>\"",
        "Alph@ <email@example.org>",
        "<karel.čapek@history.cz>",
        "<email@example.org.>",
        "IP <email@1.2.3.4>",
    ] {
        assert!(!EMAIL.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn file_name_valid() {
    for input in &[
        "file.txt",
        "file123",
        "hello-world.c",
        "c++.BIN",
    ] {
        assert!(FILE_NAME.is_match(input), "{input}");
    }
}

#[test]
fn file_name_invalid() {
    for input in &[
        "file with spaces.txt",
        "file/with/slashes.txt",
        "file\twith\ttabs.txt",
        "file\nwith\nnewlines.txt",
    ] {
        assert!(!FILE_NAME.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn negatable_word_valid() {
    for input in &[
        "word",
        "word_123",
        "!check",
        "!_negated",
        "chmod-clean",
    ] {
        assert!(NEGATABLE_WORD.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn negatable_word_invalid() {
    for input in &[
        "word with spaces",
        "per%cent",
        "!",
        "!negated!",
        "UpperCase",
    ] {
        assert!(!NEGATABLE_WORD.is_match(input), "{input}");
    }
}

#[test]
fn one_line_valid() {
    for input in &[
        "This is a single line string.",
        "123",
        "No_newline_characters",
    ] {
        assert!(ONE_LINE.is_match(input), "{input}");
    }
}

#[test]
fn one_line_invalid() {
    for input in &[
        "This is a multi-line\nstring.",
        "String with\nnewline",
        "String with\rcarriage return",
    ] {
        assert!(!ONE_LINE.is_match(input), "{input}");
    }
}

#[test]
fn pkgname_valid() {
    for input in &[
        "nginx",
        "ruby-pg_query",
        "lua5.4",
        "openssl1.1-compat",
        "gtk+2.0",
        "websocket++",
        "R",
    ] {
        assert!(PKGNAME.is_match(input), "{input}");
    }
}

#[test]
fn pkgname_invalid() {
    for input in &[
        ".package",
        "-package",
        "package name",
        "pkg/name",
        "$neak",
        "p@ckage",
        "αlpha",
    ] {
        assert!(!PKGNAME.is_match(input), "{input}");
    }
}

#[test]
fn pkgver_valid() {
    for input in &[
        "0",
        "0.12",
        "0.12a",
        "0.12b",
        "1.005",
        "20160905",
        "2.5_rc",
        "2.5_rc3",
        "2.5_beta_pre2",
        "2.5_beta3_p1",
        "0_git20230726",
        "1.2b34", // this is semantically wrong (it should be _beta34), but used in aports
    ] {
        assert!(PKGVER.is_match(input), "{input}");
        assert!(PKGVER_MAYBE_REL.is_match(input), "{input}");

        let input_rel = &format!("{input}-r0");
        assert!(PKGVER_REL.is_match(input_rel), "{input_rel}");
        assert!(PKGVER_MAYBE_REL.is_match(input_rel), "{input_rel}");
        assert!(PKGVER_REL_OR_ZERO.is_match(input_rel), "{input_rel}");
    }

    for input in &["2.5_rc3-r42", "2.5_beta_pre2-r123", "0_git20230726-r100"] {
        assert!(PKGVER_MAYBE_REL.is_match(input), "{input}");
        assert!(PKGVER_REL.is_match(input), "{input}");
        assert!(PKGVER_REL_OR_ZERO.is_match(input), "{input}");
    }

    assert!(PKGVER_REL_OR_ZERO.is_match("0"), "0");
}

#[test]
#[rustfmt::skip]
fn pkgver_invalid() {
    for input in &[
        "1_2_3",
        "1.2-3",
        "1.2a.3b",
        "1.2ab",
        "1b.23",
        "1.2-rc1",
        // apk-tools accepts the following, but it doesn't make sense...
        "1.a",
        "a-r0",
        "1.a2.3x4.b5c"
    ] {
        assert!(!PKGVER.is_match(input), "{input}");
        assert!(!PKGVER_MAYBE_REL.is_match(input), "{input}");

        let input_rel = &format!("{input}-r0");
        assert!(!PKGVER_MAYBE_REL.is_match(input_rel), "{input_rel}");
        assert!(!PKGVER_REL.is_match(input_rel), "{input_rel}");
        assert!(!PKGVER_REL_OR_ZERO.is_match(input_rel), "{input_rel}");
    }

    for input in &[
        "1",
        "1.2",
        "1_rc1",
        "1.2-r1-r2",
        "1.2r1",
        "1.2-r1a",
        "1.2-r1.5",
    ] {
        assert!(!PKGVER_REL.is_match(input), "{input}");
        assert!(!PKGVER_REL_OR_ZERO.is_match(input), "{input}");
    }

    assert!(!PKGVER_REL_OR_ZERO.is_match("000"), "000");
}

#[test]
fn provider_valid() {
    for input in &[
        "cmd:rust",
        "cmd:[",
        "so:libconfig++.so.11",
        "so:libGL.so.1",
        "py3.10:future",
        "/bin/sh",
    ] {
        assert!(PROVIDER.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn provider_invalid() {
    for input in &[
        "cmd:>",
        "spa ce",
        "key=val",
        "tilde~",
        "αlpha",
    ] {
        assert!(!PROVIDER.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn repo_pin_valid() {
    for input in &[
        "testing",
        "user-123",
        "user/Test",
    ] {
        assert!(REPO_PIN.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn repo_pin_invalid() {
    for input in &[
        "white space",
        "@testing",
        "user~123",
        ">testing",
    ] {
        assert!(!REPO_PIN.is_match(input), "{input}");
    }
}

#[test]
fn sha1_valid() {
    for input in &[
        "3ca25ae354e192b26879f651a51d92cb16e57c10",
        "0000000000000000000000000000000000000000",
        "ffffffffffffffffffffffffffffffffffffffff",
    ] {
        assert!(SHA1.is_match(input), "{input}");
    }
}

#[test]
fn sha1_invalid() {
    for input in &[
        "abcdef1234567890",
        "xyz1234567890abcdef",
        "ef537f25c895bfa782526529a9b63d97aa631564a",
    ] {
        assert!(!SHA1.is_match(input), "{input}");
    }
}

#[test]
fn sha256_valid() {
    for input in &[
        "6a9e09b0f7202e93e8b7172b5ddc1b9f2a0eb9dc55f9a1cb5820294b3b682ec1",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ] {
        assert!(SHA256.is_match(input), "{input}");
    }
}

#[test]
fn sha256_invalid() {
    for input in &[
        "abcdef1234567890",
        "xyzabcdef1234567890",
        "4096f6f3ca53b40c882d380a4177a223a7f24a249575d82db1a21ef4d1566b132",
    ] {
        assert!(!SHA256.is_match(input), "{input}");
    }
}

#[test]
fn sha512_valid() {
    for input in &[
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ] {
        assert!(SHA512.is_match(input), "{input}");
    }
}

#[test]
fn sha512_invalid() {
    for input in &[
        "abcdef1234567890",
        "xyzabcdef1234567890",
        "2906c2578963049010a354672e05eb03d14110c88dd7b9a51057aa349578b24ca67195a0bfb16be9df8fb61ea30405cda68927d28a6fb3bb0674b29ce2b56da51",
    ] {
        assert!(!SHA512.is_match(input), "{input}");
    }
}

#[test]
fn trigger_path_valid() {
    for input in &[
        "/usr/bin",
        "/usr/*/bin",
        "/var/log/messages",
        "/*/any/*",
        "/usr/lib/postgresql*",
    ] {
        assert!(TRIGGER_PATH.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn trigger_path_invalid() {
    for input in &[
        "usr/bin",
        "*/var",
        "/var/log/foo bar/",
        "/var\n/log",
    ] {
        assert!(!TRIGGER_PATH.is_match(input), "{input}");
    }
}

#[test]
fn url_valid() {
    for input in &[
        "http://foo.com/blah_blah",
        "http://foo.com/blah_blah/",
        "http://foo.COM/blah_blah_(wikipedia)",
        "http://FOO.com/blah_blah_(wikipedia)_(again)",
        "http://www.example.com/wpstyle/?p=364",
        "https://www.example.com/foo/?bar=baz&inga=42&quux",
        "http://142.42.1.1/",
        "http://142.42.1.1:8080/",
        "http://foo.com/blah_(wikipedia)#cite-1",
        "http://foo.com/blah_(wikipedia)_blah#cite-1",
        "http://foo.com/(something)?after=parens",
        "http://code.noodle.com/events/#&product=browser",
        "http://j.mp",
        "http://j.mp?o=",
        "http://foo.bar/?q=Test%20URL-encoded%20stuff",
        "HTTP://1337.net",
        "http://a.b-c.de",
        "http://223.255.255.254",
        "https://[2041:0000:140F::875B:131B]/hello",
        "https://[2041:0000:140F::]/hell",
    ] {
        assert!(URL.is_match(input), "{input}");
    }
}

#[test]
fn url_invalid() {
    for input in &[
        "http:",
        "http://",
        "http://.",
        "http://..",
        "http://../",
        "http://?",
        "http://??",
        "http://??/",
        "http://#",
        "http://##",
        "http://##/",
        "http://foo.bar?q=Spaces should be encoded",
        "http://foo.bar?var[NAME]=666",
        "//",
        "//a",
        "///a",
        "///",
        "http:///a",
        "foo.com",
        "rdar://1234",
        "h://ttp",
        "://",
        "http://foo",
        "http://foo/bar.baz",
        "ftps://foo.bar/",
        "http://-error-.invalid/",
        "http://-a.b.co",
        "http://a.b-.co",
        "http://1.1.1.1.1",
        "http://123.123.123",
        "http://3628126748",
        "http://.www.foo.bar/",
        "http://www.foo.bar./",
        "http://.www.foo.bar./",
        "file://etc/nginx",
        "file:///root",
        "ftp://ancient.com",
        "http://f:b/c",
        "http://f: /c",
        "http://f:999999/c",
        "http://f: 21 / b ? d # e",
        "http://[1::2]:3:4",
        "http://2001::1",
        "http://2001::1]",
        "http://2001::1]:80",
        "http://[::127.0.0.1.]",
        "http://foo.cz:-80/",
        "http:/:@/www.example.com",
        "http://example example.com",
        "http://[]",
        "http://[:]",
        "http://GOO 　goo.com",
        "http://\u{fdd0}zyx.com",
        "https://�",
        "http://％４１.com",
        "http://192.168.0.1 hello",
        "https://x x:12",
        "http://[www.duckduckgo.com]/",
        "http://[duckduckgo.com]",
        "http://[::1.2.3.4x]",
        "http://[::1.2.3.]",
        "http://[::.1.2]",
        "http://[::1.]",
        "http://[::.1]",
        "http://a\u{001Ab}.org/",
        "https://example.com%80/",
        "http://[0:1:2:3:4:5:6:7:8]",
        "https://[0:.0]",
        "https://[0:1:2:3:4:5:6:7.0.0.0.1]",
        "https://[0:1.23.23]",
        "http://f:4294967377/c",
        "http://[::127.0.0.0.1]",
        "http://1.2.3.4.5",
        "http://1.2.3.4.5.",
        "http://0x100.2.3.4",
        "http://foo.2.3.4",
        "http://foo.2.3.4.",
        "http://foo.09",
        "http://💩.com/",
        "https://\u{AD}/",
        "https://xn--/",
    ] {
        assert!(!URL.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn user_name_valid() {
    for input in &[
        "user",
        "_user123",
        "user.name-1",
        "user_2",
        "user$",
    ] {
        assert!(USER_NAME.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn word_valid() {
    for input in &[
        "word",
        "chmod-clean",
        "x86_64"
    ] {
        assert!(WORD.is_match(input), "{input}");
    }
}

#[test]
#[rustfmt::skip]
fn word_invalid() {
    for input in &[
        "word with spaces",
        "per%cent",
        "!",
        "!negated",
        "UpperCase",
    ] {
        assert!(!WORD.is_match(input), "{input}");
    }
}

