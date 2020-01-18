// rouille uses try!, silence deprecation warnins for now
#![allow(deprecated)]

use clap::{App, Arg};
use foolang::eval::Env;
use foolang::objects::Foolang;
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
    fn serve(&self, env: &Env) -> bool {
        match self.receiver.try_recv() {
            Ok(msg) => {
                let response = match env.eval_all(msg.as_str()) {
                    Ok(obj) => format!("{}", obj),
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
    fn new() -> Server {
        let connections: Arc<Mutex<Vec<Connection>>> = Arc::new(Mutex::new(Vec::new()));
        let connections0 = connections.clone();
        std::thread::spawn(move || loop {
            let env = Env::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                connections0.lock().unwrap().retain(|conn| conn.serve(&env));
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
            let res = match_assets(&request, "ide");
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

fn oops(what: String, app: &App) -> ! {
    println!("ERROR - {}\n---", what);
    app.clone().print_help().unwrap();
    std::process::exit(1)
}

fn find_module_or_abort(spec: &str, app: &App) -> (String, PathBuf) {
    let path = match std::fs::canonicalize(Path::new(&spec)) {
        Ok(path) => path,
        Err(_) => oops(format!("cannot find module: {}", spec), app),
    };
    let root = match path.parent() {
        Some(path) => path.to_path_buf(),
        None => oops(format!("cannot determine root of module: {}", spec), app),
    };
    let name = match path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name.to_string(),
            None => oops(format!("module has invalid filename: {}", spec), app),
        },
        None => oops(format!("cannot determine name of module: {}", spec), app),
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
                .conflicts_with("ide"),
        )
        .arg(
            Arg::with_name("use")
                .long("use")
                .value_name("MODULE")
                .help("Path to a module to use.")
                .takes_value(true)
                .multiple(true),
        )
        .arg(Arg::with_name("ide").long("ide").help("Runs the IDE"))
        .arg(Arg::with_name("verbose").long("verbose").help("Provides additional output"));
    let matches = app.clone().get_matches();
    let verbose = matches.is_present("verbose");
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
        let foo = Foolang::new(module_roots);
        // FIXME: pass in env and argv to run
        match foo.run(&program) {
            Ok(_) => std::process::exit(0),
            Err(err) => oops(err.to_string(), &app),
        }
    }
    if matches.is_present("ide") {
        println!("Starting server & browsing to http://127.0.0.1:8000/index.html");
        if webbrowser::open("http://127.0.0.1:8000/index.html").is_err() {
            println!("Could not open browser!");
        }
        // FIXME: Need to pass module_roots here.
        let server = Server::new();
        rouille::start_server("127.0.0.1:8000", move |request| {
            handle_request(request, server.clone(), verbose)
        });
    }
}
