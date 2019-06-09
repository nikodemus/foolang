use crate::ast;
use crate::ast::{Cascade, Expr, Identifier, Literal, Method};
use crate::parser::*;

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
        Expr::Send(Box::new(variable("foo")), identifier("bar"), vec![])
    );
}
#[test]
fn parse_binary() {
    assert_eq!(
        parse_expr("a + b"),
        Expr::Send(
            Box::new(variable("a")),
            identifier("+"),
            vec![variable("b")]
        )
    );
    assert_eq!(
        parse_expr("a + b ** c"),
        Expr::Send(
            Box::new(Expr::Send(
                Box::new(variable("a")),
                identifier("+"),
                vec![variable("b")]
            )),
            identifier("**"),
            vec![variable("c")]
        )
    );
}
#[test]
fn parse_keyword() {
    assert_eq!(
        parse_expr("x foo: y bar: z"),
        Expr::Send(
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
            Box::new(Expr::Send(
                Box::new(Expr::Send(
                    Box::new(Expr::Variable(Identifier(s("foo")))),
                    Identifier(s("bar")),
                    vec![]
                )),
                Identifier(s("quux")),
                vec![]
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
            statements: vec![Expr::Send(
                Box::new(variable("foo")),
                identifier("bar"),
                vec![]
            )]
        })
    );
    assert_eq!(
        parse_expr("{ foo bar. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![],
            temporaries: vec![],
            statements: vec![
                Expr::Send(Box::new(variable("foo")), identifier("bar"), vec![]),
                variable("quux")
            ]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo bar }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![Expr::Send(
                Box::new(variable("foo")),
                identifier("bar"),
                vec![]
            )]
        })
    );
    assert_eq!(
        parse_expr("{ :a | foo bar. quux }"),
        Expr::Block(ast::Block {
            parameters: vec![identifier("a")],
            temporaries: vec![],
            statements: vec![
                Expr::Send(Box::new(variable("foo")), identifier("bar"), vec![]),
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
                Expr::Send(
                    Box::new(variable("foo")),
                    identifier("+"),
                    vec![variable("bar")]
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
                Expr::Send(
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
            statements: vec![Expr::Return(Box::new(Expr::Send(
                Box::new(variable("Foo")),
                identifier("new"),
                vec![]
            )))]
        })
    );
}

#[test]
fn parse_binary_cascade() {
    assert_eq!(
        parse_expr("a + b; + c"),
        Expr::Cascade(
            Box::new(Expr::Send(
                Box::new(variable("a")),
                identifier("+"),
                vec![variable("b")]
            )),
            vec![Cascade::Message(identifier("+"), vec![variable("c")])]
        )
    );
    assert_eq!(
        parse_expr("1 + 3; + 41"),
        Expr::Cascade(
            Box::new(Expr::Send(
                Box::new(integer(1)),
                identifier("+"),
                vec![integer(3)]
            )),
            vec![Cascade::Message(identifier("+"), vec![integer(41)])]
        )
    );
}

#[test]
fn parse_keyword_cascade() {
    assert_eq!(
        parse_expr("a b c d; then: e; + f; g; then: h and: j"),
        Expr::Cascade(
            Box::new(Expr::Send(
                Box::new(Expr::Send(
                    Box::new(Expr::Send(Box::new(variable("a")), identifier("b"), vec![])),
                    identifier("c"),
                    vec![]
                )),
                identifier("d"),
                vec![]
            )),
            vec![
                Cascade::Message(identifier("then:"), vec![variable("e")]),
                Cascade::Message(identifier("+"), vec![variable("f")]),
                Cascade::Message(identifier("g"), vec![]),
                Cascade::Message(identifier("then:and:"), vec![variable("h"), variable("j")]),
            ]
        )
    )
}

#[test]
fn parse_unary_method() {
    assert_eq!(
        parse_method("foo bar quux"),
        Method {
            selector: identifier("foo"),
            parameters: vec![],
            temporaries: vec![],
            docstring: None,
            statements: vec![Expr::Send(
                Box::new(variable("bar")),
                identifier("quux"),
                vec![]
            )]
        }
    );
    assert_eq!(
        parse_method("foo |x| x := bar quux. ^x zot"),
        Method {
            selector: identifier("foo"),
            parameters: vec![],
            temporaries: vec![identifier("x")],
            docstring: None,
            statements: vec![
                Expr::Assign(
                    identifier("x"),
                    Box::new(Expr::Send(
                        Box::new(variable("bar")),
                        identifier("quux"),
                        vec![]
                    ))
                ),
                Expr::Return(Box::new(Expr::Send(
                    Box::new(variable("x")),
                    identifier("zot"),
                    vec![]
                )))
            ]
        }
    );
}

#[test]
fn parse_binary_method() {
    assert_eq!(
        parse_method(r#"+ x "This adds stuff." ^value + x"#),
        Method {
            selector: identifier("+"),
            parameters: vec![identifier("x")],
            temporaries: vec![],
            docstring: Some(String::from("This adds stuff.")),
            statements: vec![Expr::Return(Box::new(Expr::Send(
                Box::new(variable("value")),
                identifier("+"),
                vec![variable("x")]
            )))]
        }
    );
}

#[test]
fn parse_keyword_method() {
    assert_eq!(
        parse_method("foo: x with: y x frob. y blarg ding: x"),
        Method {
            selector: identifier("foo:with:"),
            parameters: vec![identifier("x"), identifier("y")],
            temporaries: vec![],
            docstring: None,
            statements: vec![
                Expr::Send(Box::new(variable("x")), identifier("frob"), vec![]),
                Expr::Send(
                    Box::new(Expr::Send(
                        Box::new(variable("y")),
                        identifier("blarg"),
                        vec![]
                    )),
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
            Expr::Send(Box::new(integer(2)), identifier("+"), vec![integer(1)]),
            float(3.1),
            integer(4)
        ])
    );
}

#[test]
fn parse_class_description() {
    assert_eq!(
        parse_class("@class Foo [x y z]"),
        ast::ClassDescription {
            name: identifier("Foo"),
            slots: vec![identifier("x"), identifier("y"), identifier("z")]
        }
    )
}

#[test]
fn parse_instance_method_description() {
    assert_eq!(
        parse_instance_method("@method Foo a:x b:y ^x + y"),
        ast::MethodDescription {
            class: identifier("Foo"),
            method: ast::Method {
                selector: identifier("a:b:"),
                parameters: vec![identifier("x"), identifier("y")],
                temporaries: vec![],
                docstring: None,
                statements: vec![Expr::Return(Box::new(Expr::Send(
                    Box::new(variable("x")),
                    identifier("+"),
                    vec![variable("y")]
                )))]
            }
        }
    );
}

#[test]
fn parse_class_method_description() {
    assert_eq!(
        parse_class_method("@class-method Foo a:x b:y ^x + y"),
        ast::MethodDescription {
            class: identifier("Foo"),
            method: ast::Method {
                selector: identifier("a:b:"),
                parameters: vec![identifier("x"), identifier("y")],
                temporaries: vec![],
                docstring: None,
                statements: vec![Expr::Return(Box::new(Expr::Send(
                    Box::new(variable("x")),
                    identifier("+"),
                    vec![variable("y")]
                )))]
            }
        }
    );
}

#[test]
fn parse_program1() {
    let prog = parse_program(
        "
        @class Foo []
        @method Foo theAnswer
            ^42
    ",
    );
    assert_eq!(
        prog,
        vec![
            ast::Definition::Class(ast::ClassDescription {
                name: identifier("Foo"),
                slots: vec![],
            }),
            ast::Definition::InstanceMethod(ast::MethodDescription {
                class: identifier("Foo"),
                method: ast::Method {
                    selector: identifier("theAnswer"),
                    parameters: vec![],
                    temporaries: vec![],
                    docstring: None,
                    statements: vec![Expr::Return(Box::new(Expr::Constant(Literal::Integer(42))))]
                }
            })
        ]
    );
}
