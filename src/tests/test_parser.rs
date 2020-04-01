use crate::def::*;
use crate::expr::*;
use crate::parse::utils::*;
use crate::parse::*;
use crate::source_location::SourceLocation;
use crate::unwind::Unwind;

fn parse_str(source: &str) -> Parse {
    Parser::new(source, "test/").parse().map_err(|unwind| unwind.with_context(source))
}

fn parse_expr(source: &str) -> ExprParse {
    Ok(parse_str(source)?.expr())
}

fn parse_def(source: &str) -> Result<Def, Unwind> {
    Ok(parse_str(source)?.def())
}

use pretty_assertions::assert_eq;

#[test]
fn parse_decimal() {
    assert_eq!(parse_expr("123"), Ok(int(0..3, 123)));
}

#[test]
fn parse_hexadecimal() {
    assert_eq!(parse_expr("0xFF"), Ok(int(0..4, 0xFF)));
}

#[test]
fn parse_binary() {
    assert_eq!(parse_expr("0b101"), Ok(int(0..5, 0b101)));
}

#[test]
fn parse_float1() {
    assert_eq!(parse_expr("1.123"), Ok(float(0..5, 1.123)));
}

#[test]
fn parse_float2() {
    assert_eq!(parse_expr("1.1e6"), Ok(float(0..5, 1.1e6)));
}

#[test]
fn parse_float3() {
    assert_eq!(parse_expr("2e6"), Ok(float(0..3, 2e6)));
}

#[test]
fn parse_var1() {
    assert_eq!(parse_expr("foo"), Ok(var(0..3, "foo")));
}

#[test]
fn parse_var2() {
    assert_eq!(parse_expr(" c"), Ok(var(1..2, "c")));
}

#[test]
fn parse_operators1() {
    assert_eq!(
        parse_expr("a + b * c"),
        Ok(binary(2..3, "+", var(0..1, "a"), binary(6..7, "*", var(4..5, "b"), var(8..9, "c"))))
    );
}

#[test]
fn parse_operators2() {
    assert_eq!(
        parse_expr("a * b + c"),
        Ok(binary(6..7, "+", binary(2..3, "*", var(0..1, "a"), var(4..5, "b")), var(8..9, "c")))
    );
}

#[test]
fn parse_operators3() {
    assert_eq!(
        parse_expr("-a - b"),
        Ok(var(1..2, "a")
            .send(Message {
                source_location: SourceLocation::span(&(0..1)),
                selector: "prefix-".to_string(),
                args: vec![]
            })
            .send(Message {
                source_location: SourceLocation::span(&(3..4)),
                selector: "-".to_string(),
                args: vec![var(5..6, "b")]
            }))
    );
}

#[test]
fn test_sequence1() {
    assert_eq!(
        parse_expr("foo bar. quux"),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(9..13, "quux")]))
    );
}

#[test]
fn test_sequence2() {
    assert_eq!(
        parse_expr(
            "foo bar.
             quux"
        ),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(22..26, "quux")]))
    );
}

#[test]
fn test_sequence4() {
    assert_eq!(
        parse_expr("foo BAR. quux."),
        Ok(seq(vec![unary(4..7, "BAR", var(0..3, "foo")), var(9..13, "quux")]))
    );
}

#[test]
fn test_define1() {
    assert_eq!(
        parse_def("define m 1. end"),
        Ok(Def::DefineDef(DefineDef {
            source_location: SourceLocation::span(&(7..8)),
            name: "m".to_string(),
            init: int(9..10, 1)
        }))
    )
}

#[test]
fn test_define2() {
    assert_eq!(
        parse_def("define m 1 m. end"),
        Ok(Def::DefineDef(DefineDef {
            source_location: SourceLocation::span(&(7..8)),
            name: "m".to_string(),
            init: unary(11..12, "m", int(9..10, 1))
        }))
    )
}

#[test]
fn test_let1() {
    assert_eq!(
        parse_expr("let x = 21 + 21. x"),
        Ok(bind(
            SourceLocation::span(&(4..5)),
            "x",
            binary(11..12, "+", int(8..10, 21), int(13..15, 21)),
            var(17..18, "x")
        ))
    );
}

#[test]
fn test_let2() {
    assert_eq!(
        parse_expr(
            "let x = 21 + 21.
             x"
        ),
        Ok(bind(
            SourceLocation::span(&(4..5)),
            "x",
            binary(11..12, "+", int(8..10, 21), int(13..15, 21)),
            var(30..31, "x")
        ))
    );
}

#[test]
fn test_keyword1() {
    assert_eq!(
        parse_expr("foo x: 1 y: 2. bar"),
        Ok(seq(vec![
            keyword(4..13, "x:y:", var(0..3, "foo"), vec![int(7..8, 1), int(12..13, 2)]),
            var(15..18, "bar")
        ]))
    );
}

#[test]
fn test_keyword2() {
    assert_eq!(
        parse_expr(
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
        parse_expr(" { foo bar } "),
        Ok(block(1..12, vec![], unary(7..10, "bar", var(3..6, "foo"))))
    );
}

#[test]
fn parse_block_args() {
    assert_eq!(
        parse_expr(" { |x| foo bar: x } "),
        Ok(block(
            1..19,
            vec!["x"],
            keyword(11..17, "bar:", var(7..10, "foo"), vec![var(16..17, "x")])
        ))
    );
}

#[test]
fn test_parse_class1() {
    assert_eq!(parse_def("class Point { x y } end"), Ok(class(0..5, "Point", vec!["x", "y"])));
}

#[test]
fn parse_method1() {
    let mut class = class(0..5, "Foo", vec![]);
    class.add_method(MethodKind::Instance, method(13..19, "bar", vec![], int(24..26, 42)));
    assert_eq!(parse_def("class Foo {} method bar 42 . end"), Ok(class));
}

#[test]
fn parse_method2() {
    let mut class = class(18..23, "Foo", vec![]);
    class.add_method(
        MethodKind::Instance,
        method(90..96, "foo", vec![], unary(130..133, "bar", var(125..129, "self"))),
    );
    class.add_method(MethodKind::Instance, method(240..246, "bar", vec![], int(275..277, 42)));
    assert_eq!(
        parse_def(
            "
                 class Foo {}
                     -- this is a foo
                     method foo
                        self bar
                     ---
                     this is a bar
                     ---
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
        parse_def(
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
    assert_eq!(
        parse_expr("return 12"),
        Ok(Return::expr(SourceLocation::span(&(0..6)), int(7..9, 12)))
    );
}

#[test]
fn test_comments1() {
    assert_eq!(
        parse_expr(
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
fn test_parse_string1() {
    assert_eq!(parse_expr(r#" "foo" "#), Ok(string(1..6, "foo")))
}

#[test]
fn test_parse_string2() {
    assert_eq!(parse_expr(r#" "" "#), Ok(string(1..3, "")))
}

#[test]
fn test_parse_string3() {
    assert_eq!(
        parse_expr(r#" "{42}" "#),
        Ok(keyword(
            6..7,
            "append:",
            keyword(
                3..5,
                "append:",
                string(1..2, ""),
                vec![unary(3..5, "toString", int(3..5, 42))]
            ),
            vec![unary(6..7, "toString", string(6..7, ""))]
        ))
    )
}

#[test]
fn parse_type_assertions1() {
    assert_eq!(
        parse_expr("foo::String"),
        Ok(typecheck(SourceLocation::span(&(5..11)), var(0..3, "foo"), "String"))
    )
}

#[test]
fn parse_type_assertions2() {
    assert_eq!(
        parse_expr("let x::Integer = 42. x"),
        Ok(bind_typed(
            SourceLocation::span(&(4..5)),
            "x",
            "Integer",
            int(17..19, 42),
            var(21..22, "x")
        ))
    )
}

#[test]
fn parse_type_assertions3() {
    assert_eq!(
        parse_expr("{ |x::Integer| x }"),
        Ok(block_typed(0..18, vec![("x", "Integer")], var(15..16, "x")))
    );
}

#[test]
fn parse_parens() {
    assert_eq!(
        parse_expr("(a+b)*c"),
        Ok(binary(5..6, "*", binary(2..3, "+", var(1..2, "a"), var(3..4, "b")), var(6..7, "c")))
    )
}

#[test]
fn test_newlines1() {
    assert!(parse_def(
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
    assert!(parse_def(
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
        parse_expr("self foo; ba1 ba2"),
        Ok(Cascade::expr(
            Box::new(var(0..4, "self").send(Message {
                source_location: SourceLocation::span(&(5..8)),
                selector: "foo".to_string(),
                args: vec![]
            })),
            vec![vec![
                Message {
                    source_location: SourceLocation::span(&(10..13)),
                    selector: "ba1".to_string(),
                    args: vec![]
                },
                Message {
                    source_location: SourceLocation::span(&(14..17)),
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
        parse_expr("self foo; ba1 ba2; fa1 fa2"),
        Ok(Cascade::expr(
            Box::new(var(0..4, "self").send(Message {
                source_location: SourceLocation::span(&(5..8)),
                selector: "foo".to_string(),
                args: vec![]
            })),
            vec![
                vec![
                    Message {
                        source_location: SourceLocation::span(&(10..13)),
                        selector: "ba1".to_string(),
                        args: vec![]
                    },
                    Message {
                        source_location: SourceLocation::span(&(14..17)),
                        selector: "ba2".to_string(),
                        args: vec![]
                    },
                ],
                vec![
                    Message {
                        source_location: SourceLocation::span(&(19..22)),
                        selector: "fa1".to_string(),
                        args: vec![]
                    },
                    Message {
                        source_location: SourceLocation::span(&(23..26)),
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
    assert_eq!(parse_expr("[]"), Ok(Array::expr(SourceLocation::span(&(0..2)), vec![])))
}

#[test]
fn test_parse_array1() {
    assert_eq!(
        parse_expr("[1]"),
        Ok(Array::expr(SourceLocation::span(&(0..3)), vec![int(1..2, 1)]))
    )
}

#[test]
fn test_parse_array2() {
    assert_eq!(
        parse_expr("[1,2,3]"),
        Ok(Array::expr(
            SourceLocation::span(&(0..7)),
            vec![int(1..2, 1), int(3..4, 2), int(5..6, 3)]
        ))
    )
}

#[test]
fn test_parse_array3() {
    assert_eq!(
        parse_expr(
            "[
                1,
                2,
                3
             ]"
        ),
        Ok(Array::expr(
            SourceLocation::span(&(0..72)),
            vec![int(18..19, 1), int(37..38, 2), int(56..57, 3)]
        ))
    )
}

#[test]
fn test_parse_import1() {
    assert_eq!(
        parse_def("import x"),
        Ok(ImportDef::def(SourceLocation::span(&(0..8)), "x.foo", "x", None))
    );
    assert_eq!(
        parse_def("import x.Y"),
        Ok(ImportDef::def(SourceLocation::span(&(0..10)), "x.foo", "", Some("Y")))
    );
    assert_eq!(
        parse_def("import x.*"),
        Ok(ImportDef::def(SourceLocation::span(&(0..10)), "x.foo", "", Some("*")))
    );
    assert_eq!(
        parse_def("import x.y.Z"),
        Ok(ImportDef::def(SourceLocation::span(&(0..12)), "x/y.foo", "", Some("Z")))
    );
    assert_eq!(
        parse_def("import x.y.z"),
        Ok(ImportDef::def(SourceLocation::span(&(0..12)), "x/y/z.foo", "z", None))
    );
}

#[test]
fn test_parse_import2() {
    assert_eq!(
        parse_def("import .x"),
        Ok(ImportDef::def(SourceLocation::span(&(0..9)), "test/x.foo", "x", None))
    );
    assert_eq!(
        parse_def("import .x.Y"),
        Ok(ImportDef::def(SourceLocation::span(&(0..11)), "test/x.foo", "", Some("Y")))
    );
    assert_eq!(
        parse_def("import .x.*"),
        Ok(ImportDef::def(SourceLocation::span(&(0..11)), "test/x.foo", "", Some("*")))
    );
    assert_eq!(
        parse_def("import .x.y.Z"),
        Ok(ImportDef::def(SourceLocation::span(&(0..13)), "test/x/y.foo", "", Some("Z")))
    );
    assert_eq!(
        parse_def("import .x.y.z"),
        Ok(ImportDef::def(SourceLocation::span(&(0..13)), "test/x/y/z.foo", "z", None))
    );
}

#[test]
fn test_parse_extend1() {
    let mut ext = ExtensionDef::new(SourceLocation::span(&(0..6)), "Foo");
    ext.add_method(MethodKind::Instance, method(11..17, "bar", vec![], int(22..24, 42)));
    assert_eq!(parse_def("extend Foo method bar 42 end"), Ok(Def::ExtensionDef(ext)));
}

#[test]
fn test_parse_interface1() {
    let mut interface = InterfaceDef::new(SourceLocation::span(&(1..10)), "Foo");
    interface.add_method(MethodKind::Instance, method(19..25, "bar", vec![], int(38..40, 42)));
    interface.add_method(MethodKind::Instance, method(71..77, "zot", vec![], int(90..93, 123)));
    interface.add_method(
        MethodKind::Required,
        method_signature(SourceLocation::span(&(55..61)), "quux", vec![]),
    );
    assert_eq!(
        parse_def(
            "
interface Foo
    method bar
        42.
    required method quux
    method zot
        123.
end"
        ),
        Ok(Def::InterfaceDef(interface))
    );
}

#[test]
fn test_parse_dict1() {
    assert_eq!(
        parse_expr(r#"{"key1" -> "val1"}"#),
        Ok(Dictionary::expr(
            SourceLocation::span(&(0..18)),
            vec![(string(1..7, "key1"), string(11..17, "val1"))]
        ))
    );
}

#[test]
fn test_parse_dict2() {
    assert_eq!(
        parse_expr(r#"{"key1" -> "val1",}"#),
        Ok(Dictionary::expr(
            SourceLocation::span(&(0..19)),
            vec![(string(1..7, "key1"), string(11..17, "val1"))]
        ))
    );
}

#[test]
fn test_parse_dict3() {
    assert_eq!(
        parse_expr(r#"{"key1" -> "val1", 2 -> 2.0}"#),
        Ok(Dictionary::expr(
            SourceLocation::span(&(0..28)),
            vec![
                (string(1..7, "key1"), string(11..17, "val1")),
                (int(19..20, 2), float(24..27, 2.0))
            ]
        ))
    );
}

#[test]
fn test_parse_dict4() {
    assert_eq!(
        parse_expr(r#"{"key1" -> "val1", foo bar -> 2.0}"#),
        Ok(Dictionary::expr(
            SourceLocation::span(&(0..34)),
            vec![
                (string(1..7, "key1"), string(11..17, "val1")),
                (unary(23..26, "bar", var(19..22, "foo")), float(30..33, 2.0))
            ]
        ))
    );
}
