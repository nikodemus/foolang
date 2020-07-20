use crate::objects::Foolang;

#[test]
fn test_clock1() {
    let foo = Foolang::here();
    let cmd = foo.into_array(vec![], foo.toplevel_env().find_global("String"));
    let res = foo
        .run(
            "class Main {}
             class method run: command in: system
                 system clock :: Clock toString!
         end",
            cmd,
        )
        .unwrap();
    assert_eq!(res.string_as_str(), "#<Clock>");
}

#[test]
fn test_clock2() {
    // FIXME: This init here smells bad.
    crate::time::TimeInfo::init();
    let foo = Foolang::here();
    let cmd = foo.into_array(vec![], foo.toplevel_env().find_global("String"));
    let res = foo
        .run(
            "class Main {}
             class method run: command in: system
                 let clock = system clock.
                 let t0 = clock time.
                 system sleep: 10.
                 let t1 = clock time.
                 t0 real < t1 real!
         end",
            cmd,
        )
        .unwrap();
    assert_eq!(res.boolean(), true);
}
