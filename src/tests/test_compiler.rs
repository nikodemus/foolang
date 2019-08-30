use crate::eval::utils::{eval_obj, eval_ok};

#[test]
fn test_compiler1() {
    assert_eq!(
        eval_ok(
            r#"
            let compiler = Compiler new
            compiler parse: "41 + 1"
            compiler evaluate
         "#
        )
        .integer(),
        42
    );
}

#[test]
fn test_compiler2() {
    let (compiler, foo) = eval_obj("Compiler new");
    compiler
        .send(
            "parse:",
            &[foo.make_string(
                "
               class Foo { bar }
                 method quux: n
                    bar * n
               end,
               let foo = Foo bar: 21
               foo quux: 2
        ",
            )],
            &foo,
        )
        .unwrap();
    let res = compiler.send("evaluate", &[], &foo).unwrap();
    assert_eq!(res, foo.make_integer(42));
    match foo.find_class("Foo", 0..0) {
        Err(_) => {}
        Ok(_) => panic!("Class leaked from compiler to parent!"),
    }
}

#[test]
fn test_parse_on_eof() {
    assert_eq!(
        eval_ok(
            r#"
            let compiler = Compiler new
            compiler parse: "41 +" onEof: { |err| err }
         "#
        )
        .string_as_str(),
        "Unexpected EOF in value position"
    );
}
