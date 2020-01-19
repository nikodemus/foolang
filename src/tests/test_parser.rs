use crate::parse::utils::*;
use crate::parse::*;
use crate::unwind::Unwind;

fn parse_str(source: &str) -> Result<Expr, Unwind> {
    parse_str_in_path(source, "test/")
}

use pretty_assertions::assert_eq;

#[test]
fn parse_decimal() {
    assert_eq!(parse_str("123"), Ok(int(0..3, 123)));
}

#[test]
fn parse_hexadecimal() {
    assert_eq!(parse_str("0xFF"), Ok(int(0..4, 0xFF)));
}

#[test]
fn parse_binary() {
    assert_eq!(parse_str("0b101"), Ok(int(0..5, 0b101)));
}

#[test]
fn parse_float1() {
    assert_eq!(parse_str("1.123"), Ok(float(0..5, 1.123)));
}

#[test]
fn parse_float2() {
    assert_eq!(parse_str("1.1e6"), Ok(float(0..5, 1.1e6)));
}

#[test]
fn parse_float3() {
    assert_eq!(parse_str("2e6"), Ok(float(0..3, 2e6)));
}

#[test]
fn parse_var1() {
    assert_eq!(parse_str("foo"), Ok(var(0..3, "foo")));
}

#[test]
fn parse_var2() {
    assert_eq!(parse_str(" c"), Ok(var(1..2, "c")));
}

#[test]
fn parse_operators1() {
    assert_eq!(
        parse_str("a + b * c"),
        Ok(binary(2..3, "+", var(0..1, "a"), binary(6..7, "*", var(4..5, "b"), var(8..9, "c"))))
    );
}

#[test]
fn parse_operators2() {
    assert_eq!(
        parse_str("a * b + c"),
        Ok(binary(6..7, "+", binary(2..3, "*", var(0..1, "a"), var(4..5, "b")), var(8..9, "c")))
    );
}

#[test]
fn parse_operators3() {
    assert_eq!(
        parse_str("-a - b"),
        Ok(var(1..2, "a")
            .send(Message {
                span: 0..1,
                selector: "prefix-".to_string(),
                args: vec![]
            })
            .send(Message {
                span: 3..4,
                selector: "-".to_string(),
                args: vec![var(5..6, "b")]
            }))
    );
}

#[test]
fn test_sequence1() {
    assert_eq!(
        parse_str("foo bar. quux"),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(9..13, "quux")]))
    );
}

#[test]
fn test_sequence2() {
    assert_eq!(
        parse_str(
            "foo bar.
             quux"
        ),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(22..26, "quux")]))
    );
}

#[test]
fn test_sequence3() {
    assert_eq!(
        parse_str("foo bar. quux."),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(9..13, "quux")]))
    );
}

#[test]
fn test_let1() {
    assert_eq!(
        parse_str("let x = 21 + 21. x"),
        Ok(bind("x", binary(11..12, "+", int(8..10, 21), int(13..15, 21)), var(17..18, "x")))
    );
}

#[test]
fn test_let2() {
    assert_eq!(
        parse_str(
            "let x = 21 + 21.
             x"
        ),
        Ok(bind("x", binary(11..12, "+", int(8..10, 21), int(13..15, 21)), var(30..31, "x")))
    );
}

#[test]
fn test_keyword1() {
    assert_eq!(
        parse_str("foo x: 1 y: 2. bar"),
        Ok(seq(vec![
            keyword(4..13, "x:y:", var(0..3, "foo"), vec![int(7..8, 1), int(12..13, 2)]),
            var(15..18, "bar")
        ]))
    );
}

#[test]
fn test_keyword2() {
    assert_eq!(
        parse_str(
            "foo x: 1 y: 2.
             bar"
        ),
        Ok(seq(vec![
            keyword(4..13, "x:y:", var(0..3, "foo"), vec![int(7..8, 1), int(12..13, 2)]),
            var(28..31, "bar")
        ]))
    );
}

#[test]
fn parse_block_no_args() {
    assert_eq!(
        parse_str(" { foo bar } "),
        Ok(block(1..12, vec![], unary(7..10, "bar", var(3..6, "foo"))))
    );
}

#[test]
fn parse_block_args() {
    assert_eq!(
        parse_str(" { |x| foo bar: x } "),
        Ok(block(
            1..19,
            vec!["x"],
            keyword(11..17, "bar:", var(7..10, "foo"), vec![var(16..17, "x")])
        ))
    );
}

#[test]
fn test_parse_class1() {
    assert_eq!(parse_str("class Point { x y } end"), Ok(class(0..5, "Point", vec!["x", "y"])));
}

#[test]
fn parse_method1() {
    let mut class = class(0..5, "Foo", vec![]);
    class.add_method(MethodKind::Instance, method(13..19, "bar", vec![], int(24..26, 42)));
    assert_eq!(parse_str("class Foo {} method bar 42 end"), Ok(class));
}

#[test]
fn parse_method2() {
    let mut class = class(18..23, "Foo", vec![]);
    class.add_method(
        MethodKind::Instance,
        method(52..58, "foo", vec![], unary(92..95, "bar", var(87..91, "self"))),
    );
    class.add_method(MethodKind::Instance, method(117..123, "bar", vec![], int(152..154, 42)));
    assert_eq!(
        parse_str(
            "
                 class Foo {}
                     method foo
                        self bar
                     method bar
                        42
                 end"
        ),
        Ok(class)
    );
}

#[test]
fn parse_method3() {
    let mut class = class(18..23, "Foo", vec![]);
    class.add_method(
        MethodKind::Instance,
        method(52..58, "foo", vec![], unary(92..95, "bar", var(87..91, "self"))),
    );
    class.add_method(MethodKind::Instance, method(118..124, "bar", vec![], int(153..155, 42)));
    assert_eq!(
        parse_str(
            "
                 class Foo {}
                     method foo
                        self bar.
                     method bar
                        42.
                 end"
        ),
        Ok(class)
    );
}

#[test]
fn parse_return1() {
    assert_eq!(parse_str("return 12"), Ok(Return::expr(0..6, int(7..9, 12))));
}

#[test]
fn test_comments1() {
    assert_eq!(
        parse_str(
            "foo --- inline block comment --- foo. -- Foo it up!
             ---
             Multiline
             block
             comment
             ---
             bar"
        ),
        Ok(seq(vec![unary(33..36, "foo", var(0..3, "foo")), var(162..165, "bar")]))
    );
}

#[test]
fn parse_string1() {
    assert_eq!(parse_str(r#" "foo" "#), Ok(string(1..6, "foo")))
}

#[test]
fn parse_string2() {
    assert_eq!(parse_str(r#" "" "#), Ok(string(1..3, "")))
}

#[test]
fn parse_type_assertions1() {
    assert_eq!(parse_str("foo::String"), Ok(typecheck(5..11, var(0..3, "foo"), "String")))
}

#[test]
fn parse_type_assertions2() {
    assert_eq!(
        parse_str("let x::Integer = 42. x"),
        Ok(bind_typed("x", "Integer", int(17..19, 42), var(21..22, "x")))
    )
}

#[test]
fn parse_type_assertions3() {
    assert_eq!(
        parse_str("{ |x::Integer| x }"),
        Ok(block_typed(0..18, vec![("x", "Integer")], var(15..16, "x")))
    );
}

#[test]
fn parse_parens() {
    assert_eq!(
        parse_str("(a+b)*c"),
        Ok(binary(5..6, "*", binary(2..3, "+", var(1..2, "a"), var(3..4, "b")), var(6..7, "c")))
    )
}

#[test]
fn test_newlines1() {
    assert!(parse_str(
        "class Point { x y }
            method add: point
               Point x: x + point x
                     y: y + point y
         end"
    )
    .is_ok());
}

#[test]
fn test_newlines2() {
    assert!(parse_str(
        "class Point

          {
            x
            y
          }

           method add: point
              Point x: x + point x
                    y: y + point y
         end"
    )
    .is_ok());
}

#[test]
fn test_parse_cascade1() {
    assert_eq!(
        parse_str("self foo; ba1 ba2"),
        Ok(Expr::Cascade(
            Box::new(var(0..4, "self").send(Message {
                span: 5..8,
                selector: "foo".to_string(),
                args: vec![]
            })),
            vec![vec![
                Message {
                    span: 10..13,
                    selector: "ba1".to_string(),
                    args: vec![]
                },
                Message {
                    span: 14..17,
                    selector: "ba2".to_string(),
                    args: vec![]
                },
            ]]
        ))
    );
}

#[test]
fn test_parse_cascade2() {
    assert_eq!(
        parse_str("self foo; ba1 ba2; fa1 fa2"),
        Ok(Expr::Cascade(
            Box::new(var(0..4, "self").send(Message {
                span: 5..8,
                selector: "foo".to_string(),
                args: vec![]
            })),
            vec![
                vec![
                    Message {
                        span: 10..13,
                        selector: "ba1".to_string(),
                        args: vec![]
                    },
                    Message {
                        span: 14..17,
                        selector: "ba2".to_string(),
                        args: vec![]
                    },
                ],
                vec![
                    Message {
                        span: 19..22,
                        selector: "fa1".to_string(),
                        args: vec![]
                    },
                    Message {
                        span: 23..26,
                        selector: "fa2".to_string(),
                        args: vec![]
                    },
                ]
            ]
        ))
    );
}

#[test]
fn test_parse_array0() {
    assert_eq!(parse_str("[]"), Ok(Array::expr(0..2, vec![])))
}

#[test]
fn test_parse_array1() {
    assert_eq!(parse_str("[1]"), Ok(Array::expr(0..3, vec![int(1..2, 1)])))
}

#[test]
fn test_parse_array2() {
    assert_eq!(
        parse_str("[1,2,3]"),
        Ok(Array::expr(0..7, vec![int(1..2, 1), int(3..4, 2), int(5..6, 3)]))
    )
}

#[test]
fn test_parse_array3() {
    assert_eq!(
        parse_str(
            "[
                1,
                2,
                3
             ]"
        ),
        Ok(Array::expr(0..72, vec![int(18..19, 1), int(37..38, 2), int(56..57, 3)]))
    )
}

#[test]
fn test_parse_import1() {
    assert_eq!(parse_str("import x"), Ok(Import::expr(0..8, "x.foo", "x", None, None)));
    assert_eq!(parse_str("import x.Y"), Ok(Import::expr(0..10, "x.foo", "", Some("Y"), None)));
    assert_eq!(parse_str("import x.*"), Ok(Import::expr(0..10, "x.foo", "", Some("*"), None)));
    assert_eq!(parse_str("import x.y.Z"), Ok(Import::expr(0..12, "x/y.foo", "", Some("Z"), None)));
    assert_eq!(parse_str("import x.y.z"), Ok(Import::expr(0..12, "x/y/z.foo", "z", None, None)));
}

#[test]
fn test_parse_import2() {
    assert_eq!(parse_str("import .x"), Ok(Import::expr(0..9, "test/x.foo", "x", None, None)));
    assert_eq!(
        parse_str("import .x.Y"),
        Ok(Import::expr(0..11, "test/x.foo", "", Some("Y"), None))
    );
    assert_eq!(
        parse_str("import .x.*"),
        Ok(Import::expr(0..11, "test/x.foo", "", Some("*"), None))
    );
    assert_eq!(
        parse_str("import .x.y.Z"),
        Ok(Import::expr(0..13, "test/x/y.foo", "", Some("Z"), None))
    );
    assert_eq!(
        parse_str("import .x.y.z"),
        Ok(Import::expr(0..13, "test/x/y/z.foo", "z", None, None))
    );
}

#[test]
fn test_parse_extend1() {
    let mut ext = ClassExtension::new(0..6, "Foo");
    ext.add_method(MethodKind::Instance, method(11..17, "bar", vec![], int(22..24, 42)));
    assert_eq!(parse_str("extend Foo method bar 42 end"), Ok(Expr::ClassExtension(ext)));
}
