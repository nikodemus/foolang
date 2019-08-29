use crate::eval::utils::eval_ok;

#[test]
fn test_boolean() {
    assert_eq!(eval_ok("True").boolean(), true);
    assert_eq!(eval_ok("False").boolean(), false);
}

#[test]
fn test_boolean_if_true() {
    assert_eq!(eval_ok("True ifTrue: { 1 }").integer(), 1);
    assert_eq!(eval_ok("False ifTrue: { 1 }").boolean(), false);
}

#[test]
fn test_boolean_if_false() {
    assert_eq!(eval_ok("False ifFalse: { 1 }").integer(), 1);
    assert_eq!(eval_ok("True ifFalse: { 1 }").boolean(), true);
}

#[test]
fn test_boolean_if_true_if_false() {
    assert_eq!(eval_ok("True ifTrue: { 1 } ifFalse: { 2 }").integer(), 1);
    assert_eq!(eval_ok("False ifTrue: { 1 } ifFalse: { 2 }").integer(), 2);
}

#[test]
fn test_boolean_and() {
    assert_eq!(eval_ok("True and: True").boolean(), true);
    assert_eq!(eval_ok("False and: True").boolean(), false);
    assert_eq!(eval_ok("True and: False").boolean(), false);
    assert_eq!(eval_ok("False and: False").boolean(), false);
}

#[test]
fn test_boolean_or() {
    assert_eq!(eval_ok("True or: True").boolean(), true);
    assert_eq!(eval_ok("False or: True").boolean(), true);
    assert_eq!(eval_ok("True or: False").boolean(), true);
    assert_eq!(eval_ok("False or: False").boolean(), false);
}
