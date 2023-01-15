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
    for (kv        , constraint) in vec![
        (("foo-doc", S!("*")), Dependency::new("foo-doc", None)                                               ),
        (("foo-doc", S!("= 1.2.3")), Dependency::new("foo-doc", Some(Constraint::new(Op::Equal, "1.2.3")))    ),
        (("foo"    , S!("<= 1.2")), Dependency::new("foo", Some(Constraint::new(Op::Less | Op::Equal, "1.2")))),
        (("foo"    , S!("~ 1.2")), Dependency::new("foo", Some(Constraint::new(Op::Fuzzy | Op::Equal, "1.2")))),
    ] {
        assert!(constraint.to_key_value() == kv);
        assert!(Dependency::from_key_value(kv.0, kv.1).unwrap() == constraint);
    }
}
