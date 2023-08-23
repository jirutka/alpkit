use super::*;
use crate::internal::test_utils::{assert, assert_let, S};

////////////////////////////////////////////////////////////////////////////////

#[test]
#[rustfmt::skip]
fn op_from_str_and_display() {
    for (s, op) in vec![
        ("=" , Op::Equal                          ),
        ("~" , Op::Equal | Op::Fuzzy              ),
        (">" , Op::Greater                        ),
        (">=", Op::Greater | Op::Equal            ),
        ("~>", Op::Greater | Op::Equal | Op::Fuzzy),
        ("<" , Op::Less                           ),
        ("<=", Op::Less | Op::Equal               ),
        ("~<", Op::Less | Op::Equal | Op::Fuzzy   ),
        ("><", Op::Checksum                       ),
        ("*" , Op::Any                            ),
    ] {
        assert!(op.to_string() == s);
        assert!(Op::from_str(s).unwrap() == op);
    }
}

#[test]
fn op_from_str_invalid() {
    for input in ["", "?", "<=>"] {
        assert_let!(Err(ConstraintParseError(_)) = Op::from_str(input));
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
#[rustfmt::skip]
fn constraint_from_str_and_display() {
    for (s       , constraint) in vec![
        ("=1.2.3", Constraint::new(Op::Equal, "1.2.3")            ),
        (">=1.2" , Constraint::new(Op::Greater | Op::Equal, "1.2")),
    ] {
        assert!(Constraint::from_str(s).unwrap() == constraint);
        assert!(constraint.to_string() == s);
    }

    assert!(Constraint::from_str("= 1.2.3").unwrap() == Constraint::new(Op::Equal, "1.2.3"));
}

#[test]
fn constraint_from_str_invalid() {
    for input in ["1.2.3", "foo", "=", "= ", " 1"] {
        assert_let!(Err(ConstraintParseError(_)) = Constraint::from_str(input));
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
#[rustfmt::skip]
fn dependency_key_value() {
    // This is not expected to ever be used, but test it just in case.
    let conflict_with_constraint = Dependency {
        name: S!("foo"),
        conflict: true,
        constraint: Some(Constraint::from_str(">1.2.3").unwrap()),
        repo_pin: None,
    };

    for (kv                         , constraint) in vec![
        (("foo-doc", S!("*"))       , Dependency::new("foo-doc", None)                                           ),
        (("foo-doc", S!("= 1.2.3")) , Dependency::new("foo-doc", Some(Constraint::new(Op::Equal, "1.2.3")))      ),
        (("foo"    , S!("<= 1.2"))  , Dependency::new("foo", Some(Constraint::new(Op::Less | Op::Equal, "1.2"))) ),
        (("foo"    , S!("~ 1.2"))   , Dependency::new("foo", Some(Constraint::new(Op::Fuzzy | Op::Equal, "1.2")))),
        (("foo"    , S!("!"))       , Dependency::conflict("foo")                                                ),
        (("foo"    , S!("!> 1.2.3")), conflict_with_constraint                                                   ),
    ] {
        assert!(constraint.to_key_value() == kv);
        assert!(Dependency::from_key_value(kv.0, kv.1).unwrap() == constraint);
    }
}

#[test]
#[rustfmt::skip]
#[cfg(feature = "validate")]
fn dependency_validate_valid() {
    let dependencies = vec![
        Dependency::new("foo-doc", None),
        Dependency::new("foo-doc", Some(Constraint::new(Op::Equal, "1.2.3"))),
        Dependency::new("foo", Some(Constraint::new(Op::Less | Op::Equal, "1.2_rc1"))),
        Dependency::new("foo", Some(Constraint::new(Op::Fuzzy | Op::Equal, "2.5_beta_pre2-r123"))),
    ];
    for dependency in &dependencies {
        assert!(dependency.validate(&()).is_ok());
    }
    assert!(dependencies.validate(&()).is_ok());
}

#[test]
#[cfg(feature = "validate")]
fn dependency_validate_invalid() {
    let dependencies = vec![
        Dependency::new("!foo", None),
        Dependency::new("foo doc", Some(Constraint::new(Op::Equal, "1.2.3"))),
        Dependency::new("foo", Some(Constraint::new(Op::Less | Op::Equal, "1_2_3"))),
        Dependency::new("foo", Some(Constraint::new(Op::Fuzzy | Op::Equal, "a-r0"))),
    ];
    for dependency in &dependencies {
        assert!(dependency.validate(&()).is_err());
    }
    assert!(dependencies.validate(&()).is_err());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
#[cfg(feature = "validate")]
fn dependencies_validate_duplicates() {
    let deps: Dependencies = vec![
        Dependency::new("foo", None),
        Dependency::new("bar", None),
        Dependency::new("baz", Some(Constraint::new(Op::Greater, "1.0"))),
        Dependency::conflict("foo"),
        Dependency::new("baz", None),
    ]
    .into();

    assert_let!(Err(errors) = deps.validate(&()));
    assert!(errors
        .to_string()
        .ends_with("has duplicate dependency names: foo, baz"));
}

#[test]
fn dependencies_collection_methods() {
    let mut deps = Dependencies::default();

    assert!(deps.len() == 0);
    assert!(deps.is_empty());

    deps.add(Dependency::new("foo", None));
    deps.add(Dependency::new("foo", None));

    assert!(deps.len() == 2);
    assert!(!deps.is_empty());

    deps.remove(&Dependency::new("foo", None));

    assert!(deps.len() == 0);
}
