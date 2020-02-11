use crate::eval::utils::{eval_obj, eval_ok};

#[test]
fn test_compiler1() {
    assert_eq!(
        eval_ok(
            r#"
            let compiler = Compiler new.
            compiler parse: "41 + 1".
            compiler evaluate
         "#
        )
        .integer(),
        42
    );
}

#[test]
fn test_compiler2() {
    let (compiler, env) = eval_obj("Compiler new");
    compiler
        .send(
            "parse:",
            &[env.foo.make_string(
                "
               class Foo { bar }
                 method quux: n
                    bar * n
               end
               let foo = Foo bar: 21.
               foo quux: 2
        ",
            )],
            &env,
        )
        .unwrap();
    let res = compiler.send("evaluate", &[], &env).unwrap();
    assert_eq!(res, env.foo.make_integer(42));
    match env.find_global_or_unwind("Foo") {
        Err(_) => {}
        Ok(_) => panic!("Class leaked from compiler to parent!"),
    }
}

#[test]
fn test_parse_on_eof() {
    assert_eq!(
        eval_ok(
            r#"
            let compiler = Compiler new.
            compiler parse: "41 +" onEof: { |err| err }
         "#
        )
        .string_as_str(),
        "Unexpected EOF in value position"
    );
}
