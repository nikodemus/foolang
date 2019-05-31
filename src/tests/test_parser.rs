use crate::ast::{Expr, Identifier, Literal, Method, Pattern};
use crate::parser::{parse_expr, parse_method};

// helpers
fn s(s: &str) -> String {
    s.to_string()
}
fn identifier(s: &str) -> Identifier {
    Identifier(s.to_string())
}
fn variable(s: &str) -> Expr {
    Expr::Variable(identifier(s))
}

#[test]
fn parse_literals() {
    assert_eq!(parse_expr("42"), Expr::Constant(Literal::Integer(42)));
    assert_eq!(parse_expr("12.23"), Expr::Constant(Literal::Float(12.23)));
    assert_eq!(parse_expr("$x"), Expr::Constant(Literal::Character(s("x"))));
    assert_eq!(
        parse_expr("#foo:bar:"),
        Expr::Constant(Literal::Symbol(s("foo:bar:")))
    );
    assert_eq!(
        parse_expr("'bleep''bloop'"),
        Expr::Constant(Literal::String(s("bleep''bloop")))
    );
    assert_eq!(
        parse_expr("#(321 34.5 $$ _foobar:quux:zot: 'string' (level2))"),
        Expr::Constant(Literal::Array(vec![
            Literal::Integer(321),
            Literal::Float(34.5),
            Literal::Character("$".to_string()),
            Literal::Symbol(s("_foobar:quux:zot:")),
            Literal::String(s("string")),
            Literal::Array(vec![Literal::Symbol(s("level2"))]),
        ]))
    );
}
#[test]
fn parse_variable() {
    assert_eq!(parse_expr("foo"), variable("foo"));
}
#[test]
fn parse_unary() {
    assert_eq!(
        parse_expr("foo bar"),
        Expr::Unary(Box::new(variable("foo")), identifier("bar"))
    );
}
#[test]
fn parse_binary() {
    assert_eq!(
        parse_expr("a + b"),
        Expr::Binary(
            Box::new(variable("a")),
            identifier("+"),
            Box::new(variable("b"))
        )
    );
    assert_eq!(
        parse_expr("a + b ** c"),
        Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(variable("a")),
                identifier("+"),
                Box::new(variable("b"))
            )),
            identifier("**"),
            Box::new(variable("c"))
        )
    );
}
#[test]
fn parse_keyword() {
    assert_eq!(
        parse_expr("x foo: y bar: z"),
        Expr::Keyword(
            Box::new(variable("x")),
            vec![identifier("foo:"), identifier("bar:")],
            vec![variable("y"), variable("z")]
        )
    );
}
#[test]
fn parse_assign() {
    assert_eq!(
        parse_expr("foo := foo bar quux"),
        Expr::Assign(
            Identifier(s("foo")),
            Box::new(Expr::Unary(
                Box::new(Expr::Unary(
                    Box::new(Expr::Variable(Identifier(s("foo")))),
                    Identifier(s("bar"))
                )),
                Identifier(s("quux"))
            ))
        )
    );
}
#[test]
fn parse_block() {
    assert_eq!(
        parse_expr("{ foo }"),
        Expr::Block(vec![], vec![variable("foo")])
    );
    assert_eq!(
        parse_expr("{ foo bar }"),
        Expr::Block(
            vec![],
            vec![Expr::Unary(Box::new(variable("foo")), identifier("bar"))]
        )
    );
    assert_eq!(
        parse_expr("{ foo bar. quux }"),
        Expr::Block(
            vec![],
            vec![
                Expr::Unary(Box::new(variable("foo")), identifier("bar")),
                variable("quux")
            ]
        )
    );
    assert_eq!(
        parse_expr("{ :a | foo bar }"),
        Expr::Block(
            vec![identifier("a")],
            vec![Expr::Unary(Box::new(variable("foo")), identifier("bar"))]
        )
    );
    assert_eq!(
        parse_expr("{ :a | foo bar. quux }"),
        Expr::Block(
            vec![identifier("a")],
            vec![
                Expr::Unary(Box::new(variable("foo")), identifier("bar")),
                variable("quux")
            ]
        )
    );
    assert_eq!(
        parse_expr("{ :a | foo + bar. quux }"),
        Expr::Block(
            vec![identifier("a")],
            vec![
                Expr::Binary(
                    Box::new(variable("foo")),
                    identifier("+"),
                    Box::new(variable("bar"))
                ),
                variable("quux")
            ]
        )
    );
    assert_eq!(
        parse_expr("{ :a | foo with: bar and: a. quux }"),
        Expr::Block(
            vec![identifier("a")],
            vec![
                Expr::Keyword(
                    Box::new(variable("foo")),
                    vec![identifier("with:"), identifier("and:")],
                    vec![variable("bar"), variable("a")]
                ),
                variable("quux")
            ]
        )
    );
    assert_eq!(
        parse_expr("{ ^Foo new }"),
        Expr::Block(
            vec![],
            vec![Expr::Return(Box::new(Expr::Unary(
                Box::new(variable("Foo")),
                identifier("new")
            )))]
        )
    );
}

#[test]
fn parse_unary_method() {
    assert_eq!(
        parse_method("foo bar quux"),
        Method {
            pattern: Pattern::Unary(identifier("foo")),
            temporaries: vec![],
            statements: vec![Expr::Unary(Box::new(variable("bar")), identifier("quux"))]
        }
    );
    assert_eq!(
        parse_method("foo |x| x := bar quux. ^x zot"),
        Method {
            pattern: Pattern::Unary(identifier("foo")),
            temporaries: vec![identifier("x")],
            statements: vec![
                Expr::Assign(
                    identifier("x"),
                    Box::new(Expr::Unary(Box::new(variable("bar")), identifier("quux")))
                ),
                Expr::Return(Box::new(Expr::Unary(
                    Box::new(variable("x")),
                    identifier("zot")
                )))
            ]
        }
    );
}

#[test]
fn parse_binary_method() {
    assert_eq!(
        parse_method("+ x ^value + x"),
        Method {
            pattern: Pattern::Binary(identifier("+"), identifier("x")),
            temporaries: vec![],
            statements: vec![Expr::Return(Box::new(Expr::Binary(
                Box::new(variable("value")),
                identifier("+"),
                Box::new(variable("x"))
            )))]
        }
    );
}

#[test]
fn parse_keyword_method() {
    assert_eq!(
        parse_method("foo: x with: y x frob. y blarg ding: x"),
        Method {
            pattern: Pattern::Keyword(
                vec![identifier("foo:"), identifier("with:")],
                vec![identifier("x"), identifier("y")]
            ),
            temporaries: vec![],
            statements: vec![
                Expr::Unary(Box::new(variable("x")), identifier("frob")),
                Expr::Keyword(
                    Box::new(Expr::Unary(Box::new(variable("y")), identifier("blarg"))),
                    vec![identifier("ding:")],
                    vec![variable("x")]
                )
            ]
        }
    )
}
