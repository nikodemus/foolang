use crate::evaluator::eval_str;
use crate::objects::Object;

#[test]
fn foolang_compiler() {
    assert_eq!(eval_str("Foolang compiler tryParse: '1 +'"), Object::make_boolean(false));
    assert_eq!(eval_str("Foolang compiler tryParse: '1 + 2'"), Object::make_boolean(true));
    assert_eq!(eval_str("Foolang compiler tryParse: '1 + 2'; evaluate"), Object::make_integer(3));
}

#[test]
fn foolang_classes() {
    assert_eq!(eval_str("Foolang classes size > 10"), Object::make_boolean(true));
}

#[test]
fn foolang_class_name() {
    /*
    assert_eq!(
        eval_str("(Foolang classes detect: { :class | class name == #String }) name"),
        Object::make_string("String")
    );
    */
}
