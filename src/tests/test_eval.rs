use crate::eval::utils::{eval_exception, eval_obj, eval_ok, eval_str};
use crate::objects::Slot;
use crate::unwind::Unwind;
use crate::unwind::{Error, Location, SimpleError, TypeError};

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
               self
            method up: by
               a = a + by
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
               self
            method up: by
               a = a + by
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
            class method new
                self a: 42
            class method foo
                12
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
            class method new
                self _a: 42
            class method foo
                12
            method a
                _a
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
                  bar
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
                  self
               method zot
                  bar
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
                  self
             end
             ((Foo bar: 41) foo: 1) bar"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable4() {
    let (exception, env) = eval_exception(
        "class Foo { bar::Integer }
           method foo: x
              bar = bar + x.
              self
         end
         ((Foo bar: 41) foo: 1.0) bar",
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(42.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(66..69),
                context: Some(
                    concat!(
                        "002            method foo: x\n",
                        "003               bar = bar + x.\n",
                        "                  ^^^ Integer expected, got: Float 42.0\n",
                        "004               self\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck1() {
    let (object, env) = eval_obj("123::Integer");
    assert_eq!(object, env.foo.make_integer(123));
}

#[test]
fn test_typecheck2() {
    let (exception, env) = eval_exception("123::String");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_integer(123),
                expected: "String".to_string()
            }),
            Location {
                span: Some(0..3),
                context: Some(
                    concat!("001 123::String\n", "    ^^^ String expected, got: Integer 123\n")
                        .to_string()
                ),
            }
        )
    );
}

#[test]
fn test_typecheck3() {
    let (exception, env) = eval_exception("let x::Integer = 42.0. x");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(42.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(17..21),
                context: Some(
                    concat!(
                        "001 let x::Integer = 42.0. x\n",
                        "                     ^^^^ Integer expected, got: Float 42.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck4() {
    let (exception, env) = eval_exception("let x::Integer = 42. x = 1.0. x");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(1.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(21..22),
                context: Some(
                    concat!(
                        "001 let x::Integer = 42. x = 1.0. x\n",
                        "                         ^ Integer expected, got: Float 1.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck5() {
    assert_eq!(eval_ok("{ |x::Integer| x } value: 41").integer(), 41);
}

#[test]
fn test_typecheck6() {
    let (exception, env) = eval_exception("{ |x::Integer| x } value: 41.0");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(41.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(3..4),
                context: Some(
                    concat!(
                        "001 { |x::Integer| x } value: 41.0\n",
                        "       ^ Integer expected, got: Float 41.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck7() {
    let (exception, env) = eval_exception("{ |y x::Integer| x = y } value: 41.0 value: 42");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(41.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(17..18),
                context: Some(
                    concat!(
                        "001 { |y x::Integer| x = y } value: 41.0 value: 42\n",
                        "                     ^ Integer expected, got: Float 41.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck8() {
    let (exception, env) = eval_exception(
        "class Foo {}
            defaultConstructor foo
            method zot: x::Integer
                x
         end
         Foo foo zot: 1.0",
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(1.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(72..73),
                context: Some(
                    concat!(
                        "002             defaultConstructor foo\n",
                        "003             method zot: x::Integer\n",
                        "                            ^ Integer expected, got: Float 1.0\n",
                        "004                 x\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck9() {
    let (exception, env) = eval_exception(
        "class Foo {}
            defaultConstructor foo
            method zot: x -> Integer
                x + 1
         end
         Foo foo zot: 1.0",
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(2.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(101..102),
                context: Some(
                    concat!(
                        "003             method zot: x -> Integer\n",
                        "004                 x + 1\n",
                        "                    ^ Integer expected, got: Float 2.0\n",
                        "005          end\n",
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck10() {
    let (exception, env) = eval_exception("{|x| -> Integer x + 1} value: 1.0");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: env.foo.make_float(2.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(16..17),
                context: Some(
                    concat!(
                        "001 {|x| -> Integer x + 1} value: 1.0\n",
                        "                    ^ Integer expected, got: Float 2.0\n",
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_let1() {
    assert_eq!(eval_ok("let x = 42. x").integer(), 42);
}

#[test]
fn test_let2() {
    assert_eq!(eval_ok("let x = 1. let x = 42. x").integer(), 42);
}

#[test]
fn test_let3() {
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
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Cannot assign to an unbound variable",
            }),
            Location {
                span: Some(11..12),
                context: Some(
                    concat!(
                        "001 let x = 1. z = x + 1. let y = x. y\n",
                        "               ^ Cannot assign to an unbound variable\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_unary() {
    assert_eq!(eval_ok("42 asFloat asInteger").integer(), 42);
}

#[test]
fn test_unbound() {
    assert_eq!(
        eval_str("let foo = 41. foo + bar"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Unbound variable",
            }),
            Location {
                span: Some(20..23),
                context: Some(
                    concat!(
                        "001 let foo = 41. foo + bar\n",
                        "                        ^^^ Unbound variable\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn test_class_not_toplevel() {
    assert_eq!(
        eval_str("let x = 42. class Point { x y } end"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Class definition not at toplevel",
            }),
            Location {
                span: Some(12..17),
                context: Some(
                    concat!(
                        "001 let x = 42. class Point { x y } end\n",
                        "                ^^^^^ Class definition not at toplevel\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn test_class1() {
    let class = eval_ok("class Point { x y } end").class();
    assert_eq!(class.instance_vtable.name, "Point");
    assert_eq!(
        class.instance_vtable.slots["x"],
        Slot {
            index: 0,
            vtable: None,
        }
    );
    assert_eq!(
        class.instance_vtable.slots["y"],
        Slot {
            index: 1,
            vtable: None,
        }
    );
}

#[test]
fn eval_global1() {
    assert_eq!(
        eval_str("DoesNotExist"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Undefined global",
            }),
            Location {
                span: Some(0..12),
                context: Some(
                    concat!("001 DoesNotExist\n", "    ^^^^^^^^^^^^ Undefined global\n")
                        .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_global2() {
    let class = eval_ok("Integer").class();
    assert_eq!(class.instance_vtable.name, "Integer");
    assert!(class.instance_vtable.slots.is_empty());
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
            method no 42
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
            method bar 311
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
               self bar
            method bar
               311
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
               Foo value: value + other value
         end
         class Bar { a b }
            method sum
              a + b
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
               2
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
                31
            method boo: blk
                blk value
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
               return 1 + let x = 41. self test0: x
            method test0: x
               self test1: { return x }.
               return 100
            method test1: blk
               self test2: blk.
               return 1000
            method test2: blk
               blk value.
               return 10000
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
                   "not understood: {m} args: {args}"
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
                  class method bar: x
                               quux: y
                    x + y
               end
               Foo bar: 40 quux: 2"#
        )
        .integer(),
        42
    );
}
