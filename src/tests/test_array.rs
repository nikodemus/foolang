use crate::eval::utils::{eval_obj, eval_ok};

#[test]
fn test_array_ctor_0() {
    let (obj, _foo) = eval_obj("[]");
    obj.as_vec(move |vec| {
        assert_eq!(vec.len(), 0);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_array_ctor_1() {
    let (obj, _foo) = eval_obj("[42]");
    obj.as_vec(move |vec| {
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0].integer(), 42);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_array_ctor_2() {
    let (obj, _foo) = eval_obj("[31,42,53]");
    obj.as_vec(move |vec| {
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].integer(), 31);
        assert_eq!(vec[1].integer(), 42);
        assert_eq!(vec[2].integer(), 53);
        Ok(())
    })
    .unwrap();
}

#[test]
#[should_panic] // trailing comma not supported yet
fn test_array_ctor_3() {
    let (obj, _foo) = eval_obj("[31,42,53,]");
    obj.as_vec(move |vec| {
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].integer(), 31);
        assert_eq!(vec[1].integer(), 42);
        assert_eq!(vec[2].integer(), 53);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_array_push() {
    let (obj, _foo) = eval_obj(
        "let a = []
         a push: -1
         a push: 0
         a push: 1",
    );
    obj.as_vec(move |vec| {
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].integer(), -1);
        assert_eq!(vec[1].integer(), 0);
        assert_eq!(vec[2].integer(), 1);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_array_do() {
    let (obj, _foo) = eval_obj(
        "let x = 0
         [1,2,3] do: {|y| x = x + y}
         x",
    );
    assert_eq!(obj.integer(), 1 + 2 + 3);
}

#[test]
fn test_array_inject_into() {
    let (obj, _foo) = eval_obj("[1,2,3] inject: 0 into: {|sum elt| sum + elt + elt}");
    assert_eq!(obj.integer(), 1 + 1 + 2 + 2 + 3 + 3);
}

#[test]
fn test_array_eq() {
    assert_eq!(eval_ok("[1,2,3] is [1,2,3]").boolean(), false);
    assert_eq!(eval_ok("{ |arr| arr is arr } value: [1,2,3]").boolean(), true);
}

#[test]
fn test_array_to_string() {
    assert_eq!(eval_ok("[1,2,3] toString").string_as_str(), "[1, 2, 3]");
}

#[test]
fn test_array_mul() {
    assert_eq!(eval_ok("([0,1,2] * 2) toString").string_as_str(), "[0, 2, 4]");
    assert_eq!(eval_ok("(2 * [0,1,2]) toString").string_as_str(), "[0, 2, 4]");
    assert_eq!(eval_ok("(2.0 * [0,1,2]) toString").string_as_str(), "[0.0, 2.0, 4.0]");
}

#[test]
fn test_array_add() {
    assert_eq!(eval_ok("([0,1,2] + [1,2,3]) toString").string_as_str(), "[1, 3, 5]");
}
