use crate::eval::utils::{eval_obj, eval_ok, eval_str};
use crate::objects::Slot;
use crate::unwind::Unwind;
use crate::unwind::{Error, Location, SimpleError};

use pretty_assertions::assert_eq;

#[test]
fn test_is() {
    assert_eq!(eval_ok("42 is True").boolean(), false);
    assert_eq!(eval_ok("42 is 42.0").boolean(), false);
    assert_eq!(eval_ok("42 is 42").boolean(), true);
}

#[test]
fn test_cascade1() {
    assert_eq!(eval_ok("1 + 100; + 41 + 1000").integer(), 1142);
}

#[test]
fn test_cascade2() {
    assert_eq!(
        eval_ok(
            "
          class Foo { a }
            method neg
               a = -a.
               self!
            method up: by
               a = a + by!
          end
          Foo a: 44; neg up: 2; neg; a"
        )
        .integer(),
        42
    );
}

#[ignore]
#[test]
fn test_cascade3() {
    assert_eq!(
        eval_ok(
            "
          class Foo { a }
            method neg
               a = -a.
               self!
            method up: by
               a = a + by!
          end
          Foo a: 44
          ; neg up: 2
          ; neg
          ; a"
        )
        .integer(),
        42
    );
}

#[test]
fn test_class_method1() {
    let (class, env) = eval_obj(
        "class Foo { a }
            direct method new
                self a: 42!
            direct method foo
                12!
         end",
    );
    assert_eq!(class.send("foo", &[], &env), Ok(env.foo.make_integer(12)));
    assert_eq!(
        class.send("new", &[], &env).unwrap().send("a", &[], &env),
        Ok(env.foo.make_integer(42))
    );
}

#[test]
fn test_class_method2() {
    let (class, env) = eval_obj(
        "class Foo { _a }
            direct method new
                self _a: 42!
            direct method foo
                12!
            method a
                _a!
         end",
    );
    assert_eq!(class.send("foo", &[], &env), Ok(env.foo.make_integer(12)));
    assert_eq!(
        class.send("new", &[], &env).unwrap().send("a", &[], &env),
        Ok(env.foo.make_integer(42))
    );
}

#[test]
fn test_instance_variable1() {
    assert_eq!(
        eval_ok(
            "class Foo { bar }
               method zot
                  bar!
             end
             (Foo bar: 42) zot"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable2() {
    assert_eq!(
        eval_ok(
            "class Foo { bar }
               method zit
                  bar = bar + 1.
                  self!
               method zot
                  bar!
             end
             (Foo bar: 41) zit zot"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable3() {
    assert_eq!(
        eval_ok(
            "class Foo { bar::Integer }
               method foo: x
                  bar = bar + x.
                  self!
             end
             ((Foo bar: 41) foo: 1) bar"
        )
        .integer(),
        42
    );
}

#[test]
fn test_extend1() {
    assert_eq!(
        eval_ok(
            "
         class Foo {}
            direct method perform: s with: args
               666!
         end
         extend Foo
            method bar
               42!
         end
         Foo new bar",
        )
        .integer(),
        42
    );
}

#[test]
fn test_extend2() {
    assert_eq!(
        eval_ok(
            "
         class Foo {}
            method perform: s with: args
               666!
         end
         extend Foo
            direct method bar
               42!
         end
         Foo bar",
        )
        .integer(),
        42
    );
}

#[test]
fn test_eval_let1() {
    assert_eq!(eval_ok("let x = 42. x").integer(), 42);
}

#[test]
fn test_eval_let2() {
    assert_eq!(eval_ok("let x = 1. let x = 42. x").integer(), 42);
}

#[test]
fn test_eval_let3() {
    assert_eq!(eval_ok("let x = 42. let y = 1. x").integer(), 42);
}

#[test]
fn test_assign1() {
    assert_eq!(eval_ok("let x = 1. x = x + 1. let y = x. y").integer(), 2);
}

#[test]
fn test_assign_unbound() {
    assert_eq!(
        eval_str("let x = 1. z = x + 1. let y = x. y"),
        Err(Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: "Cannot assign to an unbound variable".to_string(),
            }),
            Location::from(
                11..12,
                concat!(
                    "001 let x = 1. z = x + 1. let y = x. y\n",
                    "               ^ Cannot assign to an unbound variable\n"
                )
            )
        ))
    );
}

#[test]
fn eval_unary() {
    assert_eq!(eval_ok("42 asFloat round").integer(), 42);
}

#[test]
fn test_unbound() {
    assert_eq!(
        eval_str("let foo = 41. foo + bar"),
        Err(Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: "Unbound variable: bar".to_string(),
            }),
            Location::from(
                20..23,
                concat!(
                    "001 let foo = 41. foo + bar\n",
                    "                        ^^^ Unbound variable: bar\n"
                )
            )
        ))
    );
}

#[test]
fn test_class_not_toplevel() {
    assert_eq!(
        eval_str("{ class Point { x y } end } value"),
        Err(Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: "Definition where expression was expected".to_string(),
            }),
            Location::from(
                2..7,
                concat!(
                    "001 { class Point { x y } end } value\n",
                    "      ^^^^^ Definition where expression was expected\n"
                )
            )
        ))
    );
}

#[test]
fn test_class1() {
    let obj = eval_ok("class Point { x y } end");
    let class = obj.as_class_ref().unwrap();
    assert_eq!(class.instance_vtable.name, "Point");
    assert_eq!(
        class.find_slot("x").unwrap(),
        Slot {
            name: String::from("x"),
            index: 0,
            typed: None,
        }
    );
    assert_eq!(
        class.find_slot("y").unwrap(),
        Slot {
            name: String::from("y"),
            index: 1,
            typed: None,
        }
    );
}

#[test]
fn eval_global1() {
    assert_eq!(
        eval_str("DoesNotExist"),
        Err(Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: "Unbound variable: DoesNotExist".to_string(),
            }),
            Location::from(
                0..12,
                concat!("001 DoesNotExist\n", "    ^^^^^^^^^^^^ Unbound variable: DoesNotExist\n")
            )
        ))
    );
}

#[test]
fn eval_global2() {
    let obj = eval_ok("Integer");
    let class = obj.as_class_ref().unwrap();
    assert_eq!(class.instance_vtable.name, "Integer");
    assert!(class.instance_vtable.slots().is_empty());
}

#[test]
fn test_new_instance1() {
    let (object, env) = eval_obj("class Point { x y } end Point x: 1 y: 2");
    assert_eq!(object.send("x", &[], &env), Ok(env.foo.make_integer(1)));
    assert_eq!(object.send("y", &[], &env), Ok(env.foo.make_integer(2)));
}

#[test]
fn test_new_instance2() {
    let (object, env) = eval_obj(
        "class Oh {}
            method no 42!
            defaultConstructor noes
         end
         Oh noes",
    );
    assert_eq!(object.send("no", &[], &env), Ok(env.foo.make_integer(42)));
}

#[test]
fn test_instance_method1() {
    let (object, env) = eval_obj(
        "class Foo {}
            method bar 311!
         end
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &env), Ok(env.foo.make_integer(311)));
}

#[test]
fn test_instance_method2() {
    let (object, env) = eval_obj(
        "class Foo {}
            method foo
               self bar!
            method bar
               311!
         end
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &env), Ok(env.foo.make_integer(311)));
}

#[test]
fn test_instance_method3() {
    let (object, env) = eval_obj(
        "class Foo { value }
            method + other
               Foo value: value + other value!
         end
         class Bar { a b }
            method sum
              a + b!
         end
         Bar a: (Foo value: 1) b: (Foo value: 10)",
    );
    assert_eq!(
        object.send("sum", &[], &env).unwrap().send("value", &[], &env),
        Ok(env.foo.make_integer(11))
    );
}

#[test]
fn test_return_returns() {
    let (obj, env) = eval_obj(
        "class Foo {}
            method foo
               return 1.
               2!
         end
         Foo new foo",
    );
    assert_eq!(obj, env.foo.make_integer(1));
}

#[test]
fn test_return_from_method_block() {
    let (obj, env) = eval_obj(
        "class Foo {}
            method test
                self boo: { return 42 }.
                31!
            method boo: blk
                blk value!
         end
         Foo new test
        ",
    );
    assert_eq!(obj, env.foo.make_integer(42));
}

#[test]
fn test_return_from_deep_block_to_middle() {
    let (object, env) = eval_obj(
        "class Foo {}
            method test
               return 1 + let x = 41. self test0: x!
            method test0: x
               self test1: { return x }.
               return 100!
            method test1: blk
               self test2: blk.
               return 1000!
            method test2: blk
               blk value.
               return 10000!
         end
         Foo new test
        ",
    );
    assert_eq!(object, env.foo.make_integer(42));
}

#[test]
fn test_not_understood() {
    assert_eq!(
        eval_ok(
            r#"class Foo {}
                method perform: m with: args
                   "not understood: {m} args: {args}"!
               end
               Foo new foo: 1 bar: 2"#
        )
        .string_as_str(),
        "not understood: foo:bar: args: [1, 2]"
    );
}

#[test]
fn test_method_keyword_multiline() {
    assert_eq!(
        eval_ok(
            r#"class Foo {}
                  direct method bar: x
                               quux: y
                    x + y!
               end
               Foo bar: 40 quux: 2"#
        )
        .integer(),
        42
    );
}

#[test]
fn test_method_declares_class_as_argtype() {
    assert_eq!(
        eval_ok(
            r#"class Foo { x }
                   method y: other::Foo
                       other x!
               end
               (Foo x: 42) y: (Foo x: 123)"#
        )
        .integer(),
        123
    );
}
