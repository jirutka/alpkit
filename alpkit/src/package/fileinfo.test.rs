use serde_json::json;

use super::*;
use crate::internal::test_utils::{assert_from_to_json, S};

#[test]
fn fileinfo_json_regular() {
    assert_from_to_json!(
        FileInfo {
            path: PathBuf::from("/etc/shadow"),
            file_type: FileType::Regular,
            uname: S!("root"),
            gname: S!("shadow"),
            size: Some(926),
            mode: 0o640,
            digest: Some(S!("7f2f7c17ca2a0e67d74dd09caba7c20a079e7563")),
            ..Default::default()
        },
        json!({
            "path": "/etc/shadow",
            "type": "r",
            "gname": "shadow",
            "size": 926,
            "mode": "0640",
            "digest": "7f2f7c17ca2a0e67d74dd09caba7c20a079e7563"
        }),
    );
}

#[test]
fn fileinfo_json_dir() {
    assert_from_to_json!(
        FileInfo {
            path: PathBuf::from("/etc"),
            file_type: FileType::Directory,
            mode: 0o755,
            ..Default::default()
        },
        json!({
            "path": "/etc",
            "type": "d",
            "mode": "0755"
        }),
    );
}

#[test]
fn fileinfo_json_symlink() {
    assert_from_to_json!(
        FileInfo {
            path: PathBuf::from("/usr/bin/killall"),
            file_type: FileType::Symlink,
            link_target: Some(PathBuf::from("/bin/busybox")),
            size: Some(0),
            digest: Some(S!("d371f2e400ee8b8c2f186801514b146668373666")),
            ..Default::default()
        },
        json!({
            "path": "/usr/bin/killall",
            "type": "l",
            "link_target": "/bin/busybox",
            "size": 0,
            "mode": "0644",
            "digest": "d371f2e400ee8b8c2f186801514b146668373666"
        }),
    )
}

#[test]
fn fileinfo_json_xattrs() {
    assert_from_to_json!(
        FileInfo {
            path: PathBuf::from("/usr/bin/something"),
            file_type: FileType::Regular,
            size: Some(926),
            mode: 0o4755,
            digest: Some(S!("7f2f7c17ca2a0e67d74dd09caba7c20a079e7563")),
            xattrs: vec![Xattr {
                name: S!("user.pax.flags"),
                value: b"epm".to_vec()
            },],
            ..Default::default()
        },
        json!({
            "path": "/usr/bin/something",
            "type": "r",
            "size": 926,
            "mode": "04755",
            "digest": "7f2f7c17ca2a0e67d74dd09caba7c20a079e7563",
            "xattrs": {
                "user.pax.flags": "ZXBt"
            }
        }),
    );
}
