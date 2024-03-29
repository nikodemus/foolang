use crate::eval::utils::eval_ok;

#[test]
fn test_float() {
    assert_eq!(eval_ok("1.2").float(), 1.2);
}

#[test]
fn test_float_prefix() {
    assert_eq!(eval_ok("let x = -42.0. -x").float(), 42.0);
    assert_eq!(eval_ok("let x = 42.0. -x").float(), -42.0);
}

#[test]
fn test_float_div() {
    assert_eq!(eval_ok("10.0 / 5.0").float(), 2.0);
    assert_eq!(eval_ok("10.0 / 5").float(), 2.0);
}

#[test]
fn test_float_sub() {
    assert_eq!(eval_ok("10.0 - 5.0").float(), 5.0);
    assert_eq!(eval_ok("10.0 - 5").float(), 5.0);
}

#[test]
fn test_float_add() {
    assert_eq!(eval_ok("10.0 + 5.0").float(), 15.0);
    assert_eq!(eval_ok("10.0 + 5").float(), 15.0);
}

#[test]
fn test_float_mul() {
    assert_eq!(eval_ok("10.0 * 5.0").float(), 50.0);
    assert_eq!(eval_ok("10.0 * 5").float(), 50.0);
}

#[test]
fn test_float_as_float() {
    assert_eq!(eval_ok("42.3 asFloat").float(), 42.3);
}

#[test]
fn test_float_round() {
    assert_eq!(eval_ok("42.1 round").integer(), 42);
    assert_eq!(eval_ok("41.9 round").integer(), 42);
}

#[test]
fn test_addmul_float() {
    assert_eq!(eval_ok("1.0 + 1.0 * 2.0").float(), 3.0);
}

#[test]
fn test_float_lt() {
    assert_eq!(eval_ok("1.0 < 2.0").boolean(), true);
    assert_eq!(eval_ok("2.0 < 1.0").boolean(), false);
    assert_eq!(eval_ok("1.0 < 1.0").boolean(), false);

    assert_eq!(eval_ok("1.0 < 2").boolean(), true);
    assert_eq!(eval_ok("2.0 < 1").boolean(), false);
    assert_eq!(eval_ok("1.0 < 1").boolean(), false);
}

#[test]
fn test_float_gt() {
    assert_eq!(eval_ok("1.0 > 2.0").boolean(), false);
    assert_eq!(eval_ok("2.0 > 1.0").boolean(), true);
    assert_eq!(eval_ok("1.0 > 1.0").boolean(), false);

    assert_eq!(eval_ok("1.0 > 2").boolean(), false);
    assert_eq!(eval_ok("2.0 > 1").boolean(), true);
    assert_eq!(eval_ok("1.0 > 1").boolean(), false);
}

#[test]
fn test_float_equals() {
    assert_eq!(eval_ok("1.0 == 2.0").boolean(), false);
    assert_eq!(eval_ok("2.0 == 1.0").boolean(), false);
    assert_eq!(eval_ok("1.0 == 1.0").boolean(), true);

    assert_eq!(eval_ok("1.0 == 1").boolean(), false);
    assert_eq!(eval_ok("0.9 == 1").boolean(), false);
    assert_eq!(eval_ok("1.1 == 1").boolean(), false);
}

#[test]
fn test_float_lte() {
    assert_eq!(eval_ok("1.0 <= 2.0").boolean(), true);
    assert_eq!(eval_ok("2.0 <= 1.0").boolean(), false);
    assert_eq!(eval_ok("1.0 <= 1.0").boolean(), true);

    assert_eq!(eval_ok("1.0 <= 2").boolean(), true);
    assert_eq!(eval_ok("2.0 <= 1").boolean(), false);
    assert_eq!(eval_ok("1.0 <= 1").boolean(), true);
}

#[test]
fn test_float_gte() {
    assert_eq!(eval_ok("1.0 >= 2.0").boolean(), false);
    assert_eq!(eval_ok("2.0 >= 1.0").boolean(), true);
    assert_eq!(eval_ok("1.0 >= 1.0").boolean(), true);

    assert_eq!(eval_ok("1.0 >= 2").boolean(), false);
    assert_eq!(eval_ok("2.0 >= 1").boolean(), true);
    assert_eq!(eval_ok("1.0 >= 1").boolean(), true);
}

#[test]
fn test_float_at_least_at_most() {
    assert_eq!(eval_ok("1.0 atLeast: 0.0 atMost: 2.0").float(), 1.0);
    assert_eq!(eval_ok("0.0 atLeast: 1.0 atMost: 2.0").float(), 1.0);
    assert_eq!(eval_ok("2.0 atLeast: 0.0 atMost: 1.0").float(), 1.0);
}
