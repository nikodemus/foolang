use crate::objects::Foolang;

#[test]
fn test_clock1() {
    let res = Foolang::here()
        .run(
            "class Main { system }
                method run
                  system clock :: Clock toString
             end",
        )
        .unwrap();
    assert_eq!(res.string_as_str(), "#<Clock>");
}

#[test]
fn test_clock2() {
    // FIXME: This init here smells bad.
    crate::time::TimeInfo::init();
    let res = Foolang::here()
        .run(
            "class Main { system }
                method run
                  let clock = system clock.
                  let t0 = clock time.
                  system sleep: 10.
                  let t1 = clock time.
                  t0 real < t1 real
             end",
        )
        .unwrap();
    assert_eq!(res.boolean(), true);
}
