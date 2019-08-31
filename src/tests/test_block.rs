use crate::eval::utils::eval_ok;

#[test]
fn test_closure0() {
    assert_eq!(eval_ok("{} value").boolean(), false);
}

#[test]
fn test_closure1() {
    assert_eq!(eval_ok("let x = 41. { x + 1 } value").integer(), 42);
}

#[test]
fn test_closure2() {
    assert_eq!(eval_ok("let x = 41. { |y| x + y } value: 1").integer(), 42);
}

#[test]
fn test_closure3() {
    assert_eq!(
        eval_ok(
            "let thunk = { let x = 0. { x = x + 1. x } } value.
             thunk value + thunk value"
        )
        .integer(),
        3
    );
}

#[test]
fn test_closure4() {
    assert_eq!(eval_ok("{ |a b| b * a + 2 } value: 20 value: 2").integer(), 42);
}

#[test]
fn test_closure5() {
    assert_eq!(
        eval_ok(
            "class T {}
               class method closeOver: value
                 return { |x | value + x }
               class method test
                return (self closeOver: 40) value: 2
             end.
             T test"
        )
        .integer(),
        42
    );
}

#[test]
fn test_closure_return() {
    assert_eq!(
        eval_ok(
            "class T {}
               class method test
                 self boo: { return 42 }.
                 return 31
               class method boo: block
                 block value
             end.
             T test",
        )
        .integer(),
        42
    );
}

#[test]
fn test_closure_while_true() {
    assert_eq!(
        eval_ok(
            "let x = 1.
             {
               x = x * 2.
               x < 10
             }
             whileTrue.
             x"
        )
        .integer(),
        16
    );
}

#[test]
fn test_closure_while_false() {
    assert_eq!(
        eval_ok(
            "let x = 1.
             {
                x = x * 2.
                x > 10
             }
             whileFalse.
             x"
        )
        .integer(),
        16
    );
}

#[test]
fn test_closure_while_true_closure() {
    assert_eq!(
        eval_ok(
            "let x = 1.
             { x < 10 } whileTrue: {
               x = x * 2
             }"
        )
        .integer(),
        16
    );
}

#[test]
fn test_closure_while_false_closure() {
    assert_eq!(
        eval_ok(
            "let x = 1.
             { x > 10 } whileFalse: {
                  x = x * 2
             }"
        )
        .integer(),
        16
    );
}

#[test]
fn test_closure_on_error() {
    assert_eq!(eval_ok("{ undefined } onError: { |err| err }").string_as_str(), "Unbound variable");
}
