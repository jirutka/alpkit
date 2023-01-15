use super::*;
use crate::internal::test_utils::assert;

#[test]
fn chunks_exact_with_divisible_input() {
    let actual = ["a", "b", "c", "d"]
        .into_iter()
        .chunks_exact()
        .collect::<Vec<[&str; 2]>>();

    assert!(actual == vec![["a", "b"], ["c", "d"]]);
}

#[test]
fn chunks_exact_with_indivisible_input() {
    let actual = ["a", "b", "c", "d"]
        .into_iter()
        .chunks_exact()
        .collect::<Vec<[&str; 3]>>();

    assert!(actual == vec![["a", "b", "c"]]);
}

#[test]
fn tap_mut() {
    let actual = String::from("foo").tap_mut(|s| {
        assert!(s == "foo");
        s.push_str("bar")
    });

    assert!(actual == "foobar");
}

#[test]
fn tap_mut_if() {
    let actual = String::from("foo").tap_mut_if(true, |s| {
        assert!(s == "foo");
        s.push_str("bar")
    });

    assert!(actual == "foobar");

    let actual = String::from("foo").tap_mut_if(false, |_| {
        unreachable!("this shouldn't be called!");
    });

    assert!(actual == "foo");
}

#[test]
fn pipe_if() {
    let mut called_with = String::new();
    let returned = String::from("foo").pipe_if(true, |s| {
        called_with = s.clone();
        s
    });

    assert!(called_with == "foo");
    assert!(returned == "foo");

    let returned = String::from("foo").pipe_if(false, |_| {
        unreachable!("this shouldn't be called!");
    });

    assert!(returned == "foo");
}
