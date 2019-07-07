use clap::{App, Arg};
use foolang::evaluator::GlobalEnv;
use foolang::time::TimeInfo;
use rouille::{match_assets, post_input, session, try_or_400, Request, Response};
use webbrowser;

fn handle_request(request: &Request, env: &GlobalEnv, verbose: bool) -> Response {
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
            Response::text(format!("{}", env.eval_str(input.source.as_str()))).with_no_cache()
        } else {
            Response::empty_404()
        }
    })
}

fn main() {
    TimeInfo::init();
    let mut env = GlobalEnv::new();
    let matches = App::new("Foolang")
        .version("0.1.0")
        .arg(
            Arg::with_name("expr")
                .long("eval")
                .value_name("EXPR")
                .help("Expression to evaluate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file")
                .long("load")
                .value_name("FILE")
                .help("File to load")
                .takes_value(true),
        )
        .arg(Arg::with_name("ide").long("ide").help("Runs the IDE"))
        .arg(Arg::with_name("verbose").long("verbose").help("Provides additional output"))
        .get_matches();
    if let Some(file) = matches.value_of("file") {
        env.load_file(file);
    }
    if let Some(expr) = matches.value_of("expr") {
        env.eval_str(expr);
    }
    let verbose = matches.is_present("verbose");
    if matches.is_present("ide") {
        println!("Starting server & browsing to http://127.0.0.1:8000/index.html");
        if webbrowser::open("http://127.0.0.1:8000/index.html").is_err() {
            println!("Could not open browser!");
        }
        rouille::start_server("127.0.0.1:8000", move |request| {
            handle_request(request, &env, verbose)
        });
    }
    //env.load_file("foo/playground.foo");
    //env.eval_str("Playground terminal run");
}
