use crate::evaluator::eval_str;
use crate::objects::Object;

fn make_true() -> Object {
    Object::make_boolean(true)
}

fn make_false() -> Object {
    Object::make_boolean(true)
}

#[test]
fn array_empty() {
    assert_eq!(eval_str("[]"), Object::make_array(&[]));
    assert_eq!(eval_str("#[]"), Object::make_array(&[]));
}

#[test]
fn array_trailing_comma() {
    assert_eq!(
        eval_str("[1,2,3,]"),
        Object::make_array(&[
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_integer(3),
        ])
    );
    assert_eq!(
        eval_str("#[1,2,3,]"),
        Object::make_array(&[
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_integer(3),
        ])
    );
}

#[test]
fn array_eq() {
    assert_eq!(eval_str("[1,2,3] == [1, 2, 3]"), make_false());
    assert_eq!(
        eval_str("{ :arr | arr == arr } value: [1,2,3]"),
        make_true()
    )
}

#[test]
fn array_do() {
    assert_eq!(
        eval_str("{ |x| [1, 2, 3] do: { :elt | x := x + elt }, x } value"),
        Object::make_integer(6)
    );
}

#[test]
fn array_inject_into() {
    assert_eq!(
        eval_str("[1, 2, 3] inject: 4 into: { :sum :each | sum + each }"),
        Object::make_integer(10)
    );
}

#[test]
fn array_push() {
    assert_eq!(
        eval_str("{ |x| x := [0], x push: 1, (x push: 2) push: 3 } value"),
        Object::make_array(&[
            Object::make_integer(0),
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_integer(3),
        ])
    );
}
