pub(crate) use assert2::{assert, let_assert as assert_let};

use crate::dependency::Dependency;

macro_rules! S {
    ($s:expr) => {
        String::from($s)
    };
}
pub(crate) use S;

pub(crate) fn dependency(s: &str) -> Dependency {
    s.parse()
        .unwrap_or_else(|_| panic!("invalid dependency string: `{s}`"))
}
