use clap::{App, Arg};
use foolang::objects::Foolang;
use foolang::time::TimeInfo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn oops<T: std::fmt::Display>(what: T, app: Option<&App>) -> ! {
    println!("FATAL - {}\n---", what);
    if app.is_some() {
        app.unwrap().clone().print_help().unwrap();
    }
    std::process::exit(1)
}

fn find_module_or_abort(spec: &str, app: &App) -> (String, PathBuf) {
    let path = match std::fs::canonicalize(Path::new(&spec)) {
        Ok(path) => path,
        Err(_) => oops(format!("cannot find module: {}", spec), Some(app)),
    };
    let root = match path.parent() {
        Some(path) => path.to_path_buf(),
        None => oops(format!("cannot determine root of module: {}", spec), Some(app)),
    };
    let name = match path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name.to_string(),
            None => oops(format!("module has invalid filename: {}", spec), Some(app)),
        },
        None => oops(format!("cannot determine name of module: {}", spec), Some(app)),
    };
    return (name, root);
}

fn main() {
    // This is easier then controlling the main thread stack size.
    std::thread::Builder::new()
        .name(String::from("foo_main"))
        .stack_size(2 * 1024 * 1024)
        .spawn(foo_main)
        .unwrap()
        .join()
        .unwrap();
}

fn foo_main() {
    TimeInfo::init();
    let app = App::new("Foolang")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("program")
                .index(1)
                .value_name("PROGRAM")
                .help("Foolang program to execute, must contain a main.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("arg")
                .index(2)
                .value_name("ARG")
                .help("Commandline arguments to the Foolang program.")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("use")
                .long("use")
                .value_name("MODULE")
                .help("Path to a module to use.")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("prelude")
                .long("prelude")
                .value_name("PRELUDE")
                .help("Override standard prelude.")
                .takes_value(true)
                .default_value("foo/prelude.foo")
                .multiple(false),
        );
    let matches = app.clone().get_matches();
    let prelude = Path::new(matches.value_of("prelude").unwrap());
    let mut module_roots: HashMap<String, PathBuf> = HashMap::new();
    if let Some(values) = matches.values_of("use") {
        for spec in values {
            let (name, root) = find_module_or_abort(spec, &app);
            if module_roots.contains_key(&name) && module_roots[&name] != root {
                panic!("ERROR: module {} specified multiple times with inconsistent paths");
            }
            module_roots.insert(name.to_string(), root.to_path_buf());
        }
    }
    module_roots.insert(".".to_string(), std::env::current_dir().unwrap());
    if let Some(fname) = matches.value_of("program") {
        let (_, root) = find_module_or_abort(fname, &app);
        module_roots.insert(".".to_string(), root);
        let program = match std::fs::read_to_string(fname) {
            Ok(prog) => prog,
            Err(err) => {
                println!("ERROR - cannot load program '{}': {}", fname, err);
                app.clone().print_help().unwrap();
                std::process::exit(1)
            }
        };
        let foo = match Foolang::new(prelude, module_roots) {
            Ok(foo) => foo,
            Err(err) => oops(err, Some(&app)),
        };
        let command = foo.into_array(
            matches
                .values_of("arg")
                .map_or(vec![], |args| args.map(|arg| foo.make_string(arg)).collect()),
            foo.toplevel_env().find_global("String"),
        );
        // FIXME: pass in env and argv to run
        match foo.run(&program, command) {
            Ok(_) => std::process::exit(0),
            Err(err) => oops(err, Some(&app)),
        }
    }
}
