use crate::pratt::{parse_str, parse_str_with_position, Expr, Literal, Message, ParseError};

fn decimal(value: i64) -> Expr {
    Expr::Constant(0, Literal::Decimal(value))
}

fn float(value: f64) -> Expr {
    Expr::Constant(0, Literal::Float(value))
}

fn selector(name: &str) -> Expr {
    Expr::Constant(0, Literal::Selector(name.to_string()))
}

fn string(s: &str) -> Expr {
    Expr::Constant(0, Literal::String(s.to_string()))
}

fn character(c: char) -> Expr {
    Expr::Constant(0, Literal::Character(c))
}

fn lit_record(names: Vec<String>, values: Vec<Literal>) -> Expr {
    Expr::Constant(0, Literal::Record(names, values))
}

fn var(name: &str) -> Expr {
    Expr::Variable(0, name.to_string())
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
                   ; foo: x bar@: y -- zot
                   ; do thing
                   "
        ),
        Err(ParseError {
            position: 39,
            problem: "Invalid token",
            context: "001 obj zoo
002                    ; foo: x bar@: y -- zot
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
            0,
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
            0,
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
fn parse_array_trailing_comma() {
    assert_eq!(
        parse_str("[1,2,3,]"),
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
        parse_str("[$foo, $bar:quux:, $:::] "),
        Ok(Expr::Array(vec![
            selector("foo"),
            selector("bar:quux:"),
            selector(":::"),
        ]))
    );
}

#[test]
fn parse_character() {
    assert_eq!(parse_str("'x'"), Ok(character('x')));
}

#[test]
fn parse_literal_string() {
    assert_eq!(parse_str(r#" $"foo"$$"$ "#), Ok(string(r#"foo"$"#)));
}

#[test]
fn parse_literal_block_string() {
    assert_eq!(
        parse_str(
            r#"   $"""foo
       bar"""$"#
        ),
        Ok(string("foo\nbar"))
    );
}

#[test]
fn parse_interpolated_string_no_interpolation() {
    assert_eq!(parse_str(r#" "foo bar" "#), Ok(string("foo bar")));
}

#[test]
fn parse_interpolated_string_simple_interpolation() {
    assert_eq!(
        parse_str(r#" "foo {42} bar" "#),
        Ok(chain(
            string("foo "),
            &[
                keyword("append:", &[chain(decimal(42), &[unary("toString")])]),
                keyword("append:", &[string(" bar")])
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
                keyword("append:", &[string(" bar ")]),
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
            string("foo\n   "),
            &[
                keyword("append:", &[chain(decimal(42), &[unary("toString")])]),
                keyword("append:", &[string("\nbar")])
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

#[test]
fn parse_literal_record() {
    assert_eq!(
        parse_str("${ foo: 42 }"),
        Ok(lit_record(
            vec![String::from("foo:")],
            vec![Literal::Decimal(42)]
        ))
    );
}

#[test]
fn parse_nary_message() {
    assert_eq!(
        parse_str("foo : 1 : 2 : 3"),
        Ok(chain(
            var("foo"),
            &[keyword(":::", &[decimal(1), decimal(2), decimal(3)])]
        ))
    );
}

#[test]
fn parse_comment() {
    assert_eq!(
        parse_str(
            "# Leading comment
             # of several lines.
             expr"
        ),
        Ok(Expr::LeadingComment(
            Box::new(var("expr")),
            " Leading comment\n of several lines.".to_string()
        ))
    );
    assert_eq!(
        parse_str("expr # Trailing line comment"),
        Ok(Expr::TrailingComment(
            Box::new(var("expr")),
            " Trailing line comment".to_string()
        ))
    );
    assert_eq!(
        parse_str(
            "#Leading comment on sequence
             foo bar
             bong bong"
        ),
        Ok(Expr::LeadingComment(
            Box::new(Expr::Sequence(vec![
                chain(var("foo"), &[unary("bar")]),
                chain(var("bong"), &[unary("bong")])
            ])),
            "Leading comment on sequence".to_string()
        ))
    );
    assert_eq!(
        parse_str(
            "foo bar
             # leading comment in middle of sequence
             bong bing"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            Expr::LeadingComment(
                Box::new(chain(var("bong"), &[unary("bing")])),
                " leading comment in middle of sequence".to_string()
            )
        ]))
    );
}

#[test]
fn constant_position() {
    assert_eq!(
        parse_str_with_position("[1, 2]"),
        Ok(Expr::Array(vec![
            Expr::Constant(1, Literal::Decimal(1)),
            Expr::Constant(4, Literal::Decimal(2))
        ]))
    );
}

#[test]
fn variable_position() {
    assert_eq!(
        parse_str_with_position("[a, b]"),
        Ok(Expr::Array(vec![
            Expr::Variable(1, "a".to_string()),
            Expr::Variable(4, "b".to_string())
        ]))
    );
}

#[test]
fn block_position() {
    assert_eq!(
        parse_str_with_position("   { a }"),
        Ok(Expr::Block(
            3,
            vec![],
            Box::new(Expr::Variable(5, "a".to_string()))
        ))
    );
}
