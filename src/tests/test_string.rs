use crate::eval::utils::{eval_exception, eval_obj, eval_ok};
use crate::unwind::Unwind;
use crate::unwind::{Error, Location, SimpleError};

#[test]
fn test_empty_string() {
    assert_eq!(eval_ok(r#" "" "#).string_as_str(), "");
}

#[test]
fn test_string_append() {
    assert_eq!(
        eval_ok(
            r#"
                 "foo" append: "bar"
             "#
        )
        .string_as_str(),
        "foobar"
    );
}

#[test]
fn test_string_interpolation1() {
    let (object, env) = eval_obj(
        r#"let a = 1.
           let b = 3.
           "{a}.{a+1}.{b}.{b+1}"
          "#,
    );
    assert_eq!(object, env.foo.make_string("1.2.3.4"));
}

#[test]
fn test_interpolated_error_location() {
    let (exception, _env) = eval_exception(
        r#"

                let x = 42.
                "{X}"

             "#,
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Undefined global",
            }),
            Location {
                span: Some(47..48),
                context: Some(
                    concat!(
                        "003                 let x = 42.\n",
                        "004                 \"{X}\"\n",
                        "                     ^ Undefined global\n",
                        "005 \n"
                    )
                    .to_string()
                )
            }
        )
    );
}
