use std::fmt::{self, Write};
use std::str::FromStr;
use std::{slice, vec};

use bitmask_enum::bitmask;
#[cfg(feature = "validate")]
use garde::Validate;
use mass_cfg_attr::mass_cfg_attr;
use serde::de;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::internal::key_value_vec_map::{self, KeyValueLike};
use crate::internal::macros::bail;
#[cfg(feature = "schema-gen")]
use crate::internal::macros::define_schema_for;
#[cfg(feature = "validate")]
use crate::internal::regex;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
#[error("invalid version constraint: '{0}'")]
pub struct ConstraintParseError(String);

/// A dependency (or conflict) on a package or provider.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct Dependency {
    /// Package or provider name.
    #[garde(pattern(regex::PROVIDER))]
    pub name: String,

    /// Version constraint.
    #[garde(dive)]
    pub constraint: Option<Constraint>,

    /// If true, then this is an “anti-dependency” (conflict).
    #[garde(skip)]
    pub conflict: bool,

    /// Tag of a repository to which this dependency is pinned.
    ///
    /// Please note that this is never used in PKGINFO nor APKBUILD and it's
    /// ignored in [`KeyValueLike`].
    #[garde(ascii, pattern(regex::REPO_PIN))]
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

    pub fn conflict<S: ToString>(name: S) -> Self {
        Dependency {
            name: name.to_string(),
            constraint: None,
            conflict: true,
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
        let (conflict, constraint) = match value.strip_prefix('!') {
            Some("") => (true, None),
            Some(s) => (true, Some(Constraint::from_str(s)?)),
            None if value == "*" => (false, None),
            None => (false, Some(Constraint::from_str(&value)?)),
        };

        Ok(Dependency {
            name: key.to_owned(),
            constraint,
            conflict,
            repo_pin: None,
        })
    }

    fn to_key_value(&'a self) -> (Self::Key, Self::Value) {
        let value = match self.constraint.as_ref() {
            Some(c) if self.conflict => format!("!{} {}", c.op, c.version),
            Some(c) => format!("{} {}", c.op, c.version),
            None if self.conflict => "!".to_owned(),
            None => "*".to_owned(),
        };
        (&self.name, value)
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

/// A list of dependencies (or conflicts) on a package or provider.
#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct Dependencies(
    #[garde(dive, custom(validate_duplicate_names))]
    #[serde(with = "key_value_vec_map")]
    Vec<Dependency>,
);

impl Dependencies {
    /// Parses the given dependencies represented as a strings (see
    /// [`Dependency::from_str`]) into a new `Dependencies` collection.
    pub fn parse<'a, I>(values: I) -> Result<Dependencies, ConstraintParseError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        values.into_iter().map(Dependency::from_str).collect()
    }

    /// Adds the dependency to this collection.
    pub fn add(&mut self, dep: Dependency) {
        self.0.push(dep)
    }

    /// Returns `true` if the collection contains no dependencies.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of dependencies in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Removes a dependency from the collection. Returns whether the dependency
    /// was present or not.
    pub fn remove(&mut self, dep: &Dependency) -> bool {
        let mut found = false;

        self.0.retain(|d| {
            if d == dep {
                found = true;
                false
            } else {
                true
            }
        });
        found
    }
}

impl From<Vec<Dependency>> for Dependencies {
    fn from(value: Vec<Dependency>) -> Self {
        Dependencies(value)
    }
}

impl FromIterator<Dependency> for Dependencies {
    fn from_iter<T: IntoIterator<Item = Dependency>>(iter: T) -> Self {
        Dependencies(Vec::from_iter(iter))
    }
}

impl IntoIterator for Dependencies {
    type Item = Dependency;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = &'a Dependency;
    type IntoIter = slice::Iter<'a, Dependency>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(feature = "validate")]
fn validate_duplicate_names(deps: &Vec<Dependency>, _context: &()) -> garde::Result {
    use std::collections::HashSet;

    if deps.is_empty() {
        return Ok(());
    }
    let mut names: HashSet<_> = deps.iter().map(|d| d.name.as_str()).collect();

    if deps.len() == names.len() {
        return Ok(());
    }
    let dups = deps.iter().fold(Vec::with_capacity(3), |mut dups, dep| {
        let name = dep.name.as_str();
        if !names.remove(name) && !dups.contains(&name) {
            dups.push(name);
        }
        dups
    });

    Err(garde::Error::new(format!(
        "has duplicate dependency names: {}",
        dups.join(", ")
    )))
}

#[cfg(feature = "schema-gen")]
define_schema_for!(Dependencies, {
    "anyOf": [{
        "type": "object",
        "description": "A map of package (or provider) names and their version constraint.",
        "examples": [
            {
                "rust": "*",
                "so:libX11.so.6": "!",
                "cmd:rust": ">= 1.71_rc1-r1",
                "nginx": "!< 1.24",
            },
        ],
        "additionalProperties": false,
        "patternProperties": {
            &regex::PROVIDER.to_string(): {
                "type": "string",
                "title": "Version constraint",
                "description": "The value can be one of:\n\
                    - `*` - any version\n\
                    - `!` - conflict (negative dependency)\n\
                    - `<operator> <version>` - version constraint\n\
                    - `!<operator> <version>` - conflict with version constraint (shouldn't be used)\
                    \n\n\
                    Syntax: `(* | ! | [!]<operator> <version>)`",
                "examples": [
                    "*",
                    "!",
                    ">= 1.71_rc1-r1",
                    "!< 1.2",
                ],
                "pattern": &regex::DEP_CONSTRAINT.to_string(),
            },
        },
    }, {
        "type": "array",
        "items": {
            "type": "string",
            "description": "A package (or provider) name with an optional version constraint in \
                the same format as in APKBUILD. An exclamation mark (`!`) at the beginning \
                indicates a negative dependency (conflict).\
                \n\n\
                Syntax: `[!]<name>[<operator>[<version>]]`",
            "examples": [
                "rust",
                "!so:libX11.so.6",
                "cmd:rust>=1.71_rc1-r1",
                "!nginx<1.24",
            ],
            "pattern": &regex::PROVIDER_WITH_CONSTRAINT.to_string(),
        },
    }]
});

////////////////////////////////////////////////////////////////////////////////

/// A version constraint.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "validate", derive(Validate))]
#[mass_cfg_attr(feature = "validate", garde)]
pub struct Constraint {
    #[garde(skip)]
    pub op: Op,

    #[garde(pattern(regex::PKGVER_MAYBE_REL))]
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
