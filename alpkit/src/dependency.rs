use std::fmt::{self, Write};
use std::str::FromStr;

use bitmask_enum::bitmask;
use serde::de::{self, Deserialize};
use thiserror::Error;

use crate::internal::{key_value_vec_map::KeyValueLike, macros::bail};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
#[error("invalid version constraint: '{0}'")]
pub struct ConstraintParseError(String);

/// A dependency (or conflict) on a package or provider.
#[derive(Debug, PartialEq, Eq)]
pub struct Dependency {
    /// Package or provider name.
    pub name: String,

    /// Version constraint.
    pub constraint: Option<Constraint>,

    /// If true, then this is an “anti-dependency” (conflict).
    pub conflict: bool,

    /// Tag of a repository to which this dependency is pinned.
    ///
    /// Please note that this is never used in PKGINFO nor APKBUILD and it's
    /// ignored in [KeyValueLike].
    pub repo_pin: Option<String>,
}

impl Dependency {
    pub fn new<S: ToString>(name: S, constraint: Option<Constraint>) -> Self {
        Dependency {
            name: name.to_string(),
            constraint,
            conflict: false,
            repo_pin: None,
        }
    }
}

impl FromStr for Dependency {
    type Err = ConstraintParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, repo_pin) = s
            .split_once('@')
            .map_or((s, None), |(a, b)| (a, Some(b.to_owned())));

        let (mut name, constraint) =
            if let Some((name, constraint)) = s.find(is_op).map(|mid| s.split_at(mid)) {
                (name, Some(Constraint::from_str(constraint)?))
            } else {
                (s, None)
            };

        let conflict = name.starts_with('!');
        if conflict {
            name = &name[1..];
        }

        Ok(Dependency {
            name: name.to_string(),
            constraint,
            conflict,
            repo_pin,
        })
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.conflict {
            write!(f, "!")?;
        }
        write!(f, "{}", self.name)?;
        if let Some(constraint) = &self.constraint {
            write!(f, "{constraint}")?;
        }
        if let Some(repo_pin) = &self.repo_pin {
            write!(f, "@{repo_pin}")?;
        }
        Ok(())
    }
}

impl<'a> KeyValueLike<'a> for Dependency {
    type Key = &'a str;
    type Value = String;
    type Err = ConstraintParseError;

    fn from_key_value(key: Self::Key, value: Self::Value) -> Result<Self, Self::Err> {
        let (name, conflict) = if let Some(key) = key.strip_prefix('!') {
            (key.to_owned(), true)
        } else {
            (key.to_owned(), false)
        };
        let constraint = if value == "*" {
            None
        } else {
            Some(Constraint::from_str(&value)?)
        };

        Ok(Dependency {
            name,
            constraint,
            conflict,
            repo_pin: None,
        })
    }

    fn to_key_value(&'a self) -> (Self::Key, Self::Value) {
        (
            &self.name,
            self.constraint
                .as_ref()
                .map_or("*".to_owned(), |c| format!("{} {}", c.op, c.version)),
        )
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Dependency::from_str(&s).map_err(de::Error::custom)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A version constraint.
#[derive(Debug, PartialEq, Eq)]
pub struct Constraint {
    pub op: Op,
    pub version: String,
}

impl Constraint {
    pub fn new<V: ToString>(op: Op, version: V) -> Self {
        Constraint {
            op,
            version: version.to_string(),
        }
    }
}

impl FromStr for Constraint {
    type Err = ConstraintParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (op, version) = s
            .find(|c| !is_op(c))
            .map(|mid| s.split_at(mid))
            .map(|(a, b)| (a.trim(), b.trim()))
            .ok_or_else(|| ConstraintParseError(s.to_owned()))?;

        if op.is_empty() || version.is_empty() {
            bail!(ConstraintParseError(s.to_owned()));
        }

        Ok(Constraint {
            op: op.parse()?,
            version: version.to_owned(),
        })
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.version)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A constraint operator represented as a bit mask.
///
/// Examples:
/// ```ignore
/// Op::from_str("=").unwrap() == Op::Equal
/// Op::from_str(">=").unwrap() == Op::Greater | Op::Equal
/// Op::from_str("*") == Op::Any
/// Op::from_str("*").intersects(Op::Equal) == true
/// ```
#[bitmask(u8)]
pub enum Op {
    /// `=`
    Equal = 1,
    /// `<`
    Less = 2,
    /// `>`
    Greater = 4,
    /// `~`
    Fuzzy = 8,
    /// `><`
    Checksum = Op::Less.bits | Op::Greater.bits,
    /// `*`
    Any = Op::all().bits,
}

impl FromStr for Op {
    type Err = ConstraintParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.len() > 2 {
            bail!(ConstraintParseError(s.to_owned()))
        }
        let mut flags = Op::none();
        for c in s.chars() {
            match c {
                '=' => flags |= Op::Equal,
                '<' => flags |= Op::Less,
                '>' => flags |= Op::Greater,
                '~' => flags |= Op::Fuzzy | Op::Equal,
                '*' => flags |= Op::Any,
                _ => bail!(ConstraintParseError(s.to_owned())),
            }
        }
        Ok(flags)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_all() {
            return f.write_char('*');
        }
        if self.contains(Self::Fuzzy) {
            f.write_char('~')?;
        }
        if self.contains(Self::Greater) {
            f.write_char('>')?;
        }
        if self.contains(Self::Less) {
            f.write_char('<')?;
        }
        if self.contains(Self::Equal) && !self.contains(Self::Fuzzy) {
            f.write_char('=')?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[inline]
fn is_op(s: char) -> bool {
    matches!(s, '<' | '>' | '=' | '~')
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
#[path = "dependency.test.rs"]
mod test;
