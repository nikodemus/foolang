use clap::{App, Arg};
use foolang::eval::eval_all;
use foolang::objects::Foolang;
use foolang::time::TimeInfo;
use foolang::unwind::Unwind;
use rouille::{match_assets, post_input, session, try_or_400, Request, Response};
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
    fn serve(&self, foo: &Foolang) -> bool {
        match self.receiver.try_recv() {
            Ok(msg) => {
                let response = match eval_all(foo, msg.as_str()) {
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
            let foo = Foolang::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                connections0.lock().unwrap().retain(|conn| conn.serve(&foo));
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

fn main() {
    TimeInfo::init();
    let matches = App::new("Foolang")
        .version("0.1.0")
        .arg(
            Arg::with_name("program")
                .index(1)
                .value_name("PROGRAM")
                .help("Foolang program to execute, must contain a main.")
                .takes_value(true)
                .conflicts_with("ide"),
        )
        .arg(Arg::with_name("ide").long("ide").help("Runs the IDE"))
        .arg(Arg::with_name("verbose").long("verbose").help("Provides additional output"))
        .get_matches();
    let verbose = matches.is_present("verbose");
    if let Some(fname) = matches.value_of("program") {
        let program = match std::fs::read_to_string(fname) {
            Ok(prog) => prog,
            Err(err) => panic!("ERROR: Could not load program: {} ({})", fname, err),
        };
        let foo = Foolang::new();
        // FIXME: pass in env and argv to run
        match foo.run(&program) {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }
    }
    if matches.is_present("ide") {
        println!("Starting server & browsing to http://127.0.0.1:8000/index.html");
        if webbrowser::open("http://127.0.0.1:8000/index.html").is_err() {
            println!("Could not open browser!");
        }
        let server = Server::new();
        rouille::start_server("127.0.0.1:8000", move |request| {
            handle_request(request, server.clone(), verbose)
        });
    }
}
