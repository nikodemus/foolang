use crate::parser::parse_expr;

#[test]
fn test_format1() {
    assert_eq!("foo bar", parse_expr("foo  bar ").format());
}
