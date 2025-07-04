use std::{
    env::{self, Args},
    fs::File,
    path::Path,
};
use tiny_http::{Method, Request, Response, Server, StatusCode};

use rust_search_engine::{Result, SearchEngine, SearchEngineIndex, SearchResult};

fn main() -> Result<()> {
    let mut args = env::args();
    let program_name = args.next().expect("program name is always provided");

    let command = args.next().ok_or_else(|| {
        print_usage(&program_name);
        eprintln!("ERROR: Expected a command");
        eprintln!("Example: {program_name} <COMMAND>");
    })?;

    match command.as_str() {
        "help" => {
            print_usage(&program_name);
        }
        "index" => {
            let docs_dir = args.next().ok_or_else(|| {
                print_usage(&program_name);
                eprintln!("ERROR: Expected path of the directory with documents to index");
                eprintln!("Example: {program_name} index path/to/dir/");
            })?;
            let index = SearchEngineIndex::new(docs_dir)?;

            let dest_path = args.next().unwrap_or_else(|| String::from("index.json"));
            index.save(dest_path)?;
        }
        "search" => {
            let query = args.next().ok_or_else(|| {
                print_usage(&program_name);
                eprintln!("ERROR: Expected search query");
                eprintln!("Example: {program_name} search 'my search query'");
            })?;
            let index_path = args.next().unwrap_or_else(|| String::from("index.json"));

            let search_engine = SearchEngine::new(index_path)?;
            let search_results = search_engine.search(&query);
            for SearchResult {
                doc_path,
                importance_score,
            } in search_results
            {
                println!("{path}: {importance_score}", path = doc_path.display());
            }
        }
        "serve" => {
            let ServeCommandArgs {
                address,
                index_path,
            } = ServeCommandArgs::new(args)?;

            let server = Server::http(&address)
                .map_err(|err| eprintln!("ERROR: Couldn't start server: {err}"))?;

            let search_engine = SearchEngine::new(index_path)?;

            println!("INFO: Listening at {address}");

            for request in server.incoming_requests() {
                print_incoming_request(&request);
                let _ = handle_request(request, &search_engine);
            }
        }
        _ => {
            print_usage(&program_name);
            eprintln!("ERROR: invalid command {command}");
        }
    }

    Ok(())
}

struct ServeCommandArgs {
    address: String,
    index_path: String,
}

impl ServeCommandArgs {
    fn new(args: Args) -> Result<Self> {
        #[derive(Default)]
        struct ServeCommandArgsBuilder {
            host: Option<String>,
            port: Option<String>,
            index_path: Option<String>,
        }

        let mut builder = ServeCommandArgsBuilder::default();

        for arg in args {
            let (flag, value) = arg.split_once('=').ok_or_else(|| {
                eprintln!("ERROR: Invalid argument format, expected: --flag=value")
            })?;

            let value = Some(String::from(value));

            match flag {
                "--host" => {
                    builder.host = value;
                }
                "--port" => {
                    builder.port = value;
                }
                "--index-path" => {
                    builder.index_path = value;
                }
                _ => {
                    eprintln!("ERROR: Invalid argument: {flag}");
                    return Err(());
                }
            }
        }

        let host = builder.host.unwrap_or_else(|| String::from("127.0.0.1"));
        let port = builder.port.unwrap_or_else(|| String::from("8000"));
        let index_path = builder
            .index_path
            .unwrap_or_else(|| String::from("index.json"));

        Ok(Self {
            address: format!("{host}:{port}"),
            index_path,
        })
    }
}

fn print_incoming_request(request: &Request) {
    println!(
        "{method} {url} from {remote_addr}",
        method = request.method(),
        url = request.url(),
        remote_addr = request
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| String::from("unknown address"))
    );
}

fn print_usage(program_name: &str) {
    println!("Usage:   {program_name} <COMMAND>");
    println!();
    println!("Commands:");
    println!("  help:    Show app usage and exit.");
    println!("  index <DOCS-DIR> [DEST-PATH]:");
    println!("           Create an index from a directory of documents and save it.");
    println!("           Default destination is `index.json`.");
    println!("  search <QUERY> [INDEX-PATH]:");
    println!("           Search for relevant documents using a search query.");
    println!("           Default index path is `index.json`.");
    println!("  serve [--host=HOST] [--port=PORT] [--index-path=INDEX-PATH]:");
    println!("           Create an HTTP server to search for documents using a search query.");
    println!("           Default address is 127.0.0.1:8000");
    println!();
}

fn serve_file(path: &Path, request: Request, status_code: StatusCode) -> Result<()> {
    let file = File::open(path).map_err(|err| eprintln!("ERROR: Couldn't open file: {err}"))?;
    let response = Response::from_file(file).with_status_code(status_code);
    request
        .respond(response)
        .map_err(|err| eprintln!("ERROR: Failed to send response: {err}"))?;

    Ok(())
}

fn handle_request(mut request: Request, search_engine: &SearchEngine) -> Result<()> {
    match (request.method(), request.url()) {
        (Method::Get, "/") => serve_file(Path::new("frontend/index.html"), request, 200.into())?,
        (Method::Get, "/styles.css") => {
            serve_file(Path::new("frontend/styles.css"), request, 200.into())?
        }
        (Method::Post, "/api/search") => {
            let mut query = String::with_capacity(request.body_length().unwrap_or(32));

            request
                .as_reader()
                .read_to_string(&mut query)
                .map_err(|err| eprintln!("ERROR: Failed to read request body: {err}"))?;

            let search_results = search_engine.search(&query);

            let docs_paths: Vec<_> = search_results
                .into_iter()
                .map(|search_result| search_result.doc_path)
                // TODO: implement paging somehow (maybe from the frontend side).
                // use a page size that makes the site look good (e.g. 5)
                .collect();

            let docs_paths = serde_json::to_vec(&docs_paths)
                .map_err(|err| eprintln!("ERROR: Failed to serialize response: {err}"))?;

            request
                .respond(Response::from_data(docs_paths))
                .map_err(|err| eprintln!("ERROR: Failed to send response: {err}"))?;
        }
        (method, url) => {
            eprintln!("ERROR: {method} {url} is not supported yet");
            serve_file(Path::new("frontend/404.html"), request, 200.into())?
        }
    };

    Ok(())
}
