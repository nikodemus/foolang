use crate::ast;
use crate::ast::{Cascade, Expr, Identifier, Literal, Method, Pattern};
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

fn integer(x: i64) -> Expr {
    Expr::Constant(Literal::Integer(x))
}

fn float(x: f64) -> Expr {
    Expr::Constant(Literal::Float(x))
}

#[test]
fn parse_literals() {
    assert_eq!(parse_expr("42"), integer(42));
    assert_eq!(parse_expr("12.23"), float(12.23));
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
        parse_expr("#[321 34.5 $$ _foobar:quux:zot: 'string' [level2]]"),
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
            identifier("foo:bar:"),
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
fn parse_block_with_temporaries() {
    assert_eq!(
        parse_expr("{ |x| foo }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![identifier("x")],
            statements: vec![variable("foo")]
        })
    );
}

#[test]
fn parse_block() {
    assert_eq!(
        parse_expr("{ foo }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![],
            statements: vec![variable("foo")]
        })
    );
    assert_eq!(
        parse_expr("{ foo bar }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![],
            statements: vec![Expr::Unary(Box::new(variable("foo")), identifier("bar"))]
        })
    );
    assert_eq!(
        parse_expr("{ foo bar. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![],
            statements: vec![
                Expr::Unary(Box::new(variable("foo")), identifier("bar")),
                variable("quux")
            ]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo bar }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![Expr::Unary(Box::new(variable("foo")), identifier("bar"))]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo bar. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![
                Expr::Unary(Box::new(variable("foo")), identifier("bar")),
                variable("quux")
            ]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo + bar. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![
                Expr::Binary(
                    Box::new(variable("foo")),
                    identifier("+"),
                    Box::new(variable("bar"))
                ),
                variable("quux")
            ]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo with: bar and: a. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![
                Expr::Keyword(
                    Box::new(variable("foo")),
                    identifier("with:and:"),
                    vec![variable("bar"), variable("a")]
                ),
                variable("quux")
            ]
        })
    );
    assert_eq!(
        parse_expr("{ ^Foo new }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![],
            statements: vec![Expr::Return(Box::new(Expr::Unary(
                Box::new(variable("Foo")),
                identifier("new")
            )))]
        })
    );
}

#[test]
fn parse_binary_cascade() {
    assert_eq!(
        parse_expr("a + b; + c"),
        Expr::Cascade(
            Box::new(Expr::Binary(
                Box::new(variable("a")),
                identifier("+"),
                Box::new(variable("b"))
            )),
            vec![Cascade::Binary(identifier("+"), variable("c"))]
        )
    );
    assert_eq!(
        parse_expr("1 + 3; + 41"),
        Expr::Cascade(
            Box::new(Expr::Binary(
                Box::new(integer(1)),
                identifier("+"),
                Box::new(integer(3))
            )),
            vec![Cascade::Binary(identifier("+"), integer(41))]
        )
    );
}

#[test]
fn parse_keyword_cascade() {
    assert_eq!(
        parse_expr("a b c d; then: e; + f; g; then: h and: j"),
        Expr::Cascade(
            Box::new(Expr::Unary(
                Box::new(Expr::Unary(
                    Box::new(Expr::Unary(Box::new(variable("a")), identifier("b"))),
                    identifier("c")
                )),
                identifier("d")
            )),
            vec![
                Cascade::Keyword(identifier("then:"), vec![variable("e")]),
                Cascade::Binary(identifier("+"), variable("f")),
                Cascade::Unary(identifier("g")),
                Cascade::Keyword(identifier("then:and:"), vec![variable("h"), variable("j")]),
            ]
        )
    )
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
                identifier("foo:with:"),
                vec![identifier("x"), identifier("y")]
            ),
            temporaries: vec![],
            statements: vec![
                Expr::Unary(Box::new(variable("x")), identifier("frob")),
                Expr::Keyword(
                    Box::new(Expr::Unary(Box::new(variable("y")), identifier("blarg"))),
                    identifier("ding:"),
                    vec![variable("x")]
                )
            ]
        }
    )
}

#[test]
fn parse_array_ctor() {
    assert_eq!(
        parse_expr("[1 . 2 + 1. 3.1. 4]"),
        Expr::ArrayCtor(vec![
            integer(1),
            Expr::Binary(Box::new(integer(2)), identifier("+"), Box::new(integer(1))),
            float(3.1),
            integer(4)
        ])
    );
}
