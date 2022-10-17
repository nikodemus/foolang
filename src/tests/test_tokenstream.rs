use crate::tokenstream::*;

#[test]
fn run_test_vectors() {
    use serde_json;
    use std::fs;
    let tests = fs::read_to_string("tests/tokenization_tests.json")
        .expect("Could not read tokenization_tests.json");
    let tests: serde_json::Value = serde_json::from_str(tests.as_str()).unwrap();
    let tests = tests.as_array().unwrap();
    // Non-arrays in the test vector are comments.
    for test in tests.into_iter().map(|x| x.as_array()).filter(|x| x.is_some()).map(|x| x.unwrap())
    {
        let src = test[0].as_str().unwrap();
        let mut scanner = TokenStream::new(src);
        for expected in test[1].as_array().unwrap().into_iter().map(|x| x.as_array().unwrap()) {
            let token = scanner.scan().unwrap();
            let wanted = expected[0].as_str().unwrap();
            if wanted != token.name() {
                panic!("Scanning {} failed.\nToken {}, wanted {}", src, token.name(), wanted);
            }
            if wanted != "EOF" && wanted != "NEWLINE" {
                let wanted = expected[1].as_str().unwrap();
                if wanted != scanner.slice() {
                    panic!(
                        "Scanning {} failed.\nSlice {}, wanted {}",
                        src,
                        scanner.slice(),
                        wanted
                    );
                }
            }
        }
    }
}
