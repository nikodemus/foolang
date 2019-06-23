use crate::pratt::{parse_str, Expr, Literal, Message, ParseError};

fn decimal(value: i64) -> Expr {
    Expr::Constant(Literal::Decimal(value))
}

fn float(value: f64) -> Expr {
    Expr::Constant(Literal::Float(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable(name.to_string())
}

fn chain(object: Expr, messages: &[Message]) -> Expr {
    Expr::Chain(Box::new(object), messages.to_vec())
}

fn unary(name: &str) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![],
    }
}

fn binary(name: &str, expr: Expr) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![expr],
    }
}

fn keyword(name: &str, exprs: &[Expr]) -> Message {
    Message {
        selector: name.to_string(),
        arguments: exprs.to_vec(),
    }
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str(" 123 "), Ok(decimal(123)));
}

#[test]
fn parse_float() {
    assert_eq!(parse_str(" 123.123 "), Ok(float(123.123)));
}

#[test]
fn parse_variable() {
    assert_eq!(parse_str(" abc "), Ok(var("abc")));
}

#[test]
fn parse_unary_send() {
    assert_eq!(
        parse_str(" abc foo bar "),
        Ok(chain(var("abc"), &[unary("foo"), unary("bar")]))
    );
}

#[test]
fn parse_binary_send() {
    assert_eq!(
        parse_str(" abc + bar "),
        Ok(chain(var("abc"), &[binary("+", var("bar"))]))
    );
}

#[test]
fn parse_unary_prefix() {
    assert_eq!(parse_str(" -a "), Ok(chain(var("a"), &[unary("neg")])));
}

#[test]
fn parse_binary_precedence() {
    assert_eq!(
        parse_str(" abc + bar * quux"),
        Ok(chain(
            var("abc"),
            &[binary("+", chain(var("bar"), &[binary("*", var("quux"))]))]
        ))
    );
}

#[test]
fn parse_parens() {
    assert_eq!(
        parse_str(" abc * (bar + quux)"),
        Ok(chain(
            var("abc"),
            &[binary("*", chain(var("bar"), &[binary("+", var("quux"))]))]
        ))
    );
}

#[test]
fn parse_unary_and_binary_send() {
    assert_eq!(
        parse_str(" abc foo + bar foo2"),
        Ok(chain(
            var("abc"),
            &[
                unary("foo"),
                binary("+", chain(var("bar"), &[unary("foo2")]))
            ]
        ))
    );
}

#[test]
fn parse_keyword_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 key2: arg2"),
        Ok(chain(
            var("obj"),
            &[keyword("key1:key2:", &[var("arg1"), var("arg2")])]
        ))
    );
}

#[test]
fn parse_keyword_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- send2: arg2"),
        Ok(chain(
            var("obj"),
            &[
                keyword("send1:", &[var("arg1")]),
                keyword("send2:", &[var("arg2")])
            ]
        ))
    );
}

#[test]
fn parse_keyword_unary_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- bar"),
        Ok(chain(
            var("obj"),
            &[keyword("send1:", &[var("arg1")]), unary("bar")]
        ))
    );
}

#[test]
fn parse_keyword_and_binary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 + x key2: arg2 + y"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[binary("+", var("x"))]),
                    chain(var("arg2"), &[binary("+", var("y"))]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_keyword_and_unary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 foo bar key2: arg2 quux zot"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[unary("foo"), unary("bar")]),
                    chain(var("arg2"), &[unary("quux"), unary("zot")]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_cascade() {
    assert_eq!(
        // This is not like smalltalk cascade!
        parse_str(
            "obj zoo
                   ; foo: x bar: y -- zot
                   ; do thing
                   "
        ),
        Ok(Expr::Cascade(
            Box::new(chain(var("obj"), &[unary("zoo")])),
            vec![
                vec![keyword("foo:bar:", &[var("x"), var("y")]), unary("zot")],
                vec![unary("do"), unary("thing"),]
            ]
        ))
    );
}

#[test]
fn parse_error_context() {
    assert_eq!(
        parse_str(
            "obj zoo
                   ; foo: x bar!: y -- zot
                   ; do thing
                   "
        ),
        Err(ParseError {
            position: 40,
            problem: "Invalid token",
            context: "001 obj zoo
002                    ; foo: x bar!: y -- zot
                                    ^-- Invalid token
003                    ; do thing
"
            .to_string()
        })
    );
}

#[test]
fn test_parse_sequence() {
    assert_eq!(
        parse_str("foo bar, quux zot"),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            "
            foo bar
            quux zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            r"
            zoo foo +
              barz
            quux \
              + zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("zoo"), &[unary("foo"), binary("+", var("barz"))]),
            chain(var("quux"), &[binary("+", var("zot"))])
        ]))
    );
}

#[test]
fn parse_block() {
    assert_eq!(
        parse_str("{ a + b }"),
        Ok(Expr::Block(
            vec![],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}

#[test]
fn parse_block_with_args() {
    assert_eq!(
        parse_str("{ :a :b | a + b }"),
        Ok(Expr::Block(
            vec!["a".to_string(), "b".to_string()],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        parse_str("[1,2,3]"),
        Ok(Expr::Array(vec![decimal(1), decimal(2), decimal(3)]))
    );
}

#[test]
fn parse_bind() {
    assert_eq!(
        parse_str("let x := 42, x foo, x + 1"),
        Ok(Expr::Bind(
            "x".to_string(),
            Box::new(decimal(42)),
            Box::new(Expr::Sequence(vec![
                chain(var("x"), &[unary("foo")]),
                chain(var("x"), &[binary("+", decimal(1))]),
            ]))
        ))
    );
}

#[test]
fn parse_assign() {
    assert_eq!(
        parse_str("x := 42, x"),
        Ok(Expr::Sequence(vec![
            Expr::Assign("x".to_string(), Box::new(decimal(42))),
            var("x")
        ]))
    );
}

#[test]
fn parse_return() {
    assert_eq!(
        parse_str("return 42, 666"),
        Ok(Expr::Sequence(vec![
            Expr::Return(Box::new(decimal(42))),
            decimal(666)
        ]))
    );
}

#[test]
fn parse_type() {
    assert_eq!(
        parse_str("x <Int> + y <Int>"),
        Ok(chain(
            Expr::Type("Int".to_string(), Box::new(var("x"))),
            &[binary(
                "+",
                Expr::Type("Int".to_string(), Box::new(var("y")))
            )]
        ))
    );
}

#[test]
fn parse_selector() {
    assert_eq!(
        parse_str("[$foo, $bar:quux:] "),
        Ok(Expr::Array(vec![
            Expr::Constant(Literal::Selector("foo".to_string())),
            Expr::Constant(Literal::Selector("bar:quux:".to_string()))
        ]))
    );
}

#[test]
fn parse_character() {
    assert_eq!(
        parse_str("'x'"),
        Ok(Expr::Constant(Literal::Character('x')))
    );
}

#[test]
fn parse_literal_string() {
    assert_eq!(
        parse_str(r#" $"foo"$$"$ "#),
        Ok(Expr::Constant(Literal::String(r#"foo"$"#.to_string())))
    );
}

#[test]
fn parse_literal_block_string() {
    assert_eq!(
        parse_str(
            r#"   $"""foo
       bar"""$"#
        ),
        Ok(Expr::Constant(Literal::String("foo\nbar".to_string())))
    );
}

#[test]
fn parse_interpolated_string_no_interpolation() {
    assert_eq!(
        parse_str(r#" "foo bar" "#),
        Ok(Expr::Constant(Literal::String("foo bar".to_string())))
    );
}

#[test]
fn parse_interpolated_string_simple_interpolation() {
    assert_eq!(
        parse_str(r#" "foo {42} bar" "#),
        Ok(chain(
            Expr::Constant(Literal::String("foo ".to_string())),
            &[
                keyword("append:", &[chain(decimal(42), &[unary("toString")])]),
                keyword(
                    "append:",
                    &[Expr::Constant(Literal::String(" bar".to_string()))]
                )
            ]
        ))
    );
}

#[test]
fn parse_interpolated_string_head_and_tail() {
    assert_eq!(
        parse_str(r#" "{1} bar {2}" "#),
        Ok(chain(
            decimal(1),
            &[
                unary("toString"),
                keyword(
                    "append:",
                    &[Expr::Constant(Literal::String(" bar ".to_string()))]
                ),
                keyword("append:", &[chain(decimal(2), &[unary("toString")])])
            ]
        ))
    );
}

#[test]
fn parse_interpolated_block_string() {
    assert_eq!(
        parse_str(
            r#"   """foo
         {42}
      bar""""#
        ),
        Ok(chain(
            Expr::Constant(Literal::String("foo\n   ".to_string())),
            &[
                keyword("append:", &[chain(decimal(42), &[unary("toString")])]),
                keyword(
                    "append:",
                    &[Expr::Constant(Literal::String("\nbar".to_string()))]
                )
            ]
        ))
    );
}

#[test]
fn parse_keyword_error() {
    assert_eq!(
        parse_str("x bar: foo: 42"),
        Err(ParseError {
            position: 7,
            problem: "Not a value expression",
            context: "001 x bar: foo: 42
           ^-- Not a value expression
"
            .to_string()
        })
    );
}

#[test]
fn parse_record() {
    assert_eq!(
        parse_str("{ foo: 42 }"),
        Ok(chain(var("Record"), &[keyword("foo:", &[decimal(42)])]))
    );
}
