use crate::eval::utils::eval_ok;

#[test]
fn test_interval1() {
    assert_eq!(
        eval_ok(
            "let x = 0
             let range = 1 to: 6
             range do: {|i| x = x + i }
             x"
        )
        .integer(),
        1 + 2 + 3 + 4 + 5 + 6
    );
}

#[test]
fn test_interval2() {
    assert_eq!(
        eval_ok(
            "let x = 0
             let range = 1 to: -6
             range do: {|i| x = x + i }
             x"
        )
        .integer(),
        1 + 0 + -1 + -2 + -3 + -4 + -5 + -6
    );
}

#[test]
fn test_interval3() {
    assert_eq!(
        eval_ok(
            "let x = 0
             1 to: 6 do: {|i| x = x + i }
             x"
        )
        .integer(),
        1 + 2 + 3 + 4 + 5 + 6
    );
}

#[test]
fn test_interval4() {
    assert_eq!(
        eval_ok(
            "let x = 0
             1 to: -6 do: {|i| x = x + i }
             x"
        )
        .integer(),
        1 + 0 + -1 + -2 + -3 + -4 + -5 + -6
    );
}
