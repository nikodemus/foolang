// rouille uses try!, silence deprecation warnings for now
#![allow(deprecated)]

use clap::{App, Arg};
use foolang::eval::Env;
use foolang::objects::{Foolang, Object};
use foolang::time::TimeInfo;
use foolang::unwind::Unwind;
use rouille::{match_assets, post_input, session, try_or_400, Request, Response};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use webbrowser;

#[derive(Clone)]
struct Server {
    connections: Arc<Mutex<Vec<Connection>>>,
}

struct Connection {
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl Connection {
    fn serve(&self, env: &Env, out: &Object) -> bool {
        match self.receiver.try_recv() {
            Ok(msg) => {
                let response = match env.eval_all(msg.as_str()) {
                    Ok(obj) => {
                        let outs = out.send("content", &[], &env).unwrap().string();
                        if outs.len() > 0 {
                            format!("--output--\n{}---end---\n{}", outs, obj)
                        } else {
                            format!("{}", obj)
                        }
                    }
                    Err(Unwind::Exception(error, location)) => {
                        format!("ERROR: {}\n\n{}", error.what(), location.context())
                    }
                    _ => format!("BUG: unexpected return-from result from eval_all"),
                };
                self.sender.send(response).unwrap();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                return false;
            }
        }
        true
    }
    fn eval(&self, source: String) -> String {
        println!("client sending: {}", &source);
        self.sender.send(source).unwrap();
        self.receiver.recv().unwrap()
    }
}

impl Server {
    fn connection(&self) -> Connection {
        let (tx0, rx0) = channel();
        let (tx1, rx1) = channel();
        self.connections.lock().unwrap().push(Connection {
            sender: tx1,
            receiver: rx0,
        });
        Connection {
            sender: tx0,
            receiver: rx1,
        }
    }
    fn new(prelude: PathBuf, module_roots: HashMap<String, PathBuf>) -> Server {
        let connections: Arc<Mutex<Vec<Connection>>> = Arc::new(Mutex::new(Vec::new()));
        let connections0 = connections.clone();
        std::thread::spawn(move || loop {
            let foo = match Foolang::new(&prelude, module_roots) {
                Ok(foo) => foo,
                Err(err) => oops(err.to_string(), None),
            };
            let env = foo.toplevel_env();
            let out = env.foo.make_string_output();
            env.define("system", env.foo.make_system(Some(out.clone())));
            //sys.send("setOutput:" env.foo.make_string_stream())
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                connections0.lock().unwrap().retain(|conn| conn.serve(&env, &out));
            }
        });
        Server {
            connections,
        }
    }
}

fn handle_request(request: &Request, server: Server, verbose: bool) -> Response {
    session::session(request, "SID", 3600, |session| {
        if request.method() == "GET" {
            let res = match_assets(&request, "webrepl");
            if verbose {
                println!(
                    "GET {} {} => {}",
                    session.id(),
                    request.url(),
                    if res.is_success() {
                        "ok"
                    } else {
                        "FAILED"
                    }
                );
            }
            res.with_no_cache()
        } else if request.method() == "POST" {
            println!("POST {}", session.id());
            let input = try_or_400!(post_input!(request, { source: String }));
            let conn = server.connection();
            Response::text(format!("{}", conn.eval(input.source))).with_no_cache()
        } else {
            Response::empty_404()
        }
    })
}

fn oops(what: String, app: Option<&App>) -> ! {
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
    TimeInfo::init();
    let app = App::new("Foolang")
        .version("0.1.0")
        .arg(
            Arg::with_name("program")
                .index(1)
                .value_name("PROGRAM")
                .help("Foolang program to execute, must contain a main.")
                .takes_value(true)
                .conflicts_with("webrepl"),
        )
        .arg(
            Arg::with_name("arg")
                .index(2)
                .value_name("ARG")
                .help("Commandline arguments to the Foolang program.")
                .takes_value(true)
                .multiple(true)
                .conflicts_with("webrepl"),
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
        )
        .arg(Arg::with_name("webrepl").long("webrepl").help("Runs the web-REPL"))
        .arg(Arg::with_name("verbose").long("verbose").help("Provides additional output"));
    let matches = app.clone().get_matches();
    let verbose = matches.is_present("verbose");
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
            Err(err) => oops(err.to_string(), Some(&app)),
        };
        let command = foo.into_array(
            matches
                .values_of("arg")
                .map_or(vec![], |args| args.map(|arg| foo.make_string(arg)).collect()),
        );
        // FIXME: pass in env and argv to run
        match foo.run(&program, command) {
            Ok(_) => std::process::exit(0),
            Err(err) => oops(err.to_string(), Some(&app)),
        }
    }
    if matches.is_present("webrepl") {
        println!("Starting server & browsing to http://127.0.0.1:8000/index.html");
        if webbrowser::open("http://127.0.0.1:8000/index.html").is_err() {
            println!("Could not open browser!");
        }
        let server = Server::new(prelude.to_path_buf(), module_roots);
        rouille::start_server("127.0.0.1:8000", move |request| {
            handle_request(request, server.clone(), verbose)
        });
    }
}
