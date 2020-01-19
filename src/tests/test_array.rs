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
        "let a = [].
         a push: -1.
         a push: 0.
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
        "let x = 0.
         [1,2,3] do: {|y| x = x + y}.
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
    assert_eq!(eval_ok("([0,1,2] * 2.0) toString").string_as_str(), "[0.0, 2.0, 4.0]");
}

#[test]
fn test_array_div() {
    assert_eq!(eval_ok("([0,2,4] / 2) toString").string_as_str(), "[0, 1, 2]");
    assert_eq!(eval_ok("([0,2,4] / 2.0) toString").string_as_str(), "[0.0, 1.0, 2.0]");
    assert_eq!(eval_ok("(20 / [1,2,4]) toString").string_as_str(), "[20, 10, 5]");
    assert_eq!(eval_ok("(20.0 / [1.0,2.0,4.0]) toString").string_as_str(), "[20.0, 10.0, 5.0]");
}

#[test]
fn test_array_add() {
    assert_eq!(eval_ok("([0,1,2] + [1,2,3]) toString").string_as_str(), "[1, 3, 5]");
    assert_eq!(eval_ok("([0,1,2] + 1) toString").string_as_str(), "[1, 2, 3]");
    assert_eq!(eval_ok("(2 + [0,1,2]) toString").string_as_str(), "[2, 3, 4]");
    assert_eq!(
        eval_ok("([0.0,1.0,2.0] + [1.0,2.0,3.0]) toString").string_as_str(),
        "[1.0, 3.0, 5.0]"
    );
    assert_eq!(eval_ok("([0.0,1.0,2.0] + 1.0) toString").string_as_str(), "[1.0, 2.0, 3.0]");
    assert_eq!(eval_ok("(2.0 + [0.0,1.0,2.0]) toString").string_as_str(), "[2.0, 3.0, 4.0]");
}

#[test]
fn test_array_sub() {
    assert_eq!(eval_ok("([0,1,2] - [1,2,13]) toString").string_as_str(), "[-1, -1, -11]");
    assert_eq!(eval_ok("([0,1,2] - 1) toString").string_as_str(), "[-1, 0, 1]");
    assert_eq!(eval_ok("(2 - [0,1,2]) toString").string_as_str(), "[2, 1, 0]");
    assert_eq!(eval_ok("([0,1,2] - [1,2,13]) toString").string_as_str(), "[-1, -1, -11]");
    assert_eq!(eval_ok("([0.0,1.0,2.0] - 1.0) toString").string_as_str(), "[-1.0, 0.0, 1.0]");
    assert_eq!(eval_ok("(2.0 - [0.0,1.0,2.0]) toString").string_as_str(), "[2.0, 1.0, 0.0]");
}

#[test]
fn test_array_normalized() {
    assert_eq!(eval_ok("[0,2,0] normalized toString").string_as_str(), "[0.0, 1.0, 0.0]");
}

#[test]
fn test_array_norm() {
    assert_eq!(eval_ok("[0,2,0] norm").float(), 2.0);
    assert!(eval_ok("[1,2,3] norm").float() - 3.7417 < 0.0001);
}

#[test]
fn test_array_at() {
    assert_eq!(eval_ok("[[1,2] at: 1, [1,2] at: 2] toString").string_as_str(), "[1, 2]");
}

#[test]
fn test_array_put_at() {
    assert_eq!(
        eval_ok(
            "let a = [1,2,3].
             a put: 1.1 at: 1.
             a put: 1.2 at: 2.
             a put: 1.3 at: 3.
             a toString"
        )
        .string_as_str(),
        "[1.1, 1.2, 1.3]"
    );
}

#[test]
fn test_array_sum() {
    assert_eq!(eval_ok("[1,2,3] sum").integer(), 6);
    assert_eq!(
        eval_ok("[{|x| x + 1},{|x| x + 2},{|x| x + 3}] sum: { |b| b value: 1 }").integer(),
        9
    );
}

#[test]
fn test_array_dot() {
    assert_eq!(eval_ok("[] dot: []").integer(), 0);
    assert_eq!(eval_ok("[2] dot: [4]").integer(), 8);
    assert_eq!(eval_ok("[1,2,3] dot: [4,5,6]").integer(), 32);
    assert_eq!(eval_ok("[1.0, 2.0, 3.0] dot: [4,5,6]").float(), 32.0);
}

#[test]
fn test_array_vector_projection_on() {
    assert_eq!(
        eval_ok("([10, 20, 30] vectorProjectionOn: [1, 0, 0]) toString").string_as_str(),
        "[10, 0, 0]"
    );
    assert_eq!(
        eval_ok("([10, 20, 30] vectorProjectionOn: [0, 1, 0]) toString").string_as_str(),
        "[0, 20, 0]"
    );
    assert_eq!(
        eval_ok("([10, 20, 30] vectorProjectionOn: [0, 0, 1]) toString").string_as_str(),
        "[0, 0, 30]"
    );
    assert_eq!(
        eval_ok("([10, 20, 30] vectorProjectionOn: [1, 1, 1]) toString").string_as_str(),
        "[20, 20, 20]"
    );
}

#[test]
fn test_array_scalar_projection_on() {
    assert_eq!(
        eval_ok(
            "let v0 = [10, 20, 30].
             let v1 = [1, 0, 0].
             (v0 vectorProjectionOn: v1) norm == (v0 scalarProjectionOn: v1)"
        )
        .boolean(),
        true
    );
    assert_eq!(
        eval_ok(
            "let v0 = [10, 20, 30].
             let v1 = [0, 1, 0].
             (v0 vectorProjectionOn: v1) norm == (v0 scalarProjectionOn: v1)"
        )
        .boolean(),
        true
    );
    assert_eq!(
        eval_ok(
            "let v0 = [10, 20, 30].
             let v1 = [0, 0, 1].
             (v0 vectorProjectionOn: v1) norm == (v0 scalarProjectionOn: v1)"
        )
        .boolean(),
        true
    );
    assert_eq!(
        eval_ok(
            "let v0 = [10, 20, 30].
             let v1 = [1, 1, 1].
             (v0 vectorProjectionOn: v1) norm == (v0 scalarProjectionOn: v1)"
        )
        .boolean(),
        true
    );
}
