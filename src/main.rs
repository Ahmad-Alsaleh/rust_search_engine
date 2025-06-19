// Steps:
// read all docs.
// parse xml
// tokenize all text
// find # of times every token/term appears in each doc and in how many docs it appears
//      (compute and cache tf-idf index to disk).
// create a simple http server on localhost

// TODO: be consistent when logging ERROR vs. WARN. (if it too much of headache to decide which to
// use, just use ERROR all the time. It is no big deal).

// TODO (next steps):
// 1. save the index to a local file (as json, using serde)
//      add two methods for SearchEngineIndex: save and load
// 2. remove compute_tf_idf to a strcut called SearchEngine (wich contains an index: SearchEngineIndex)
//      create a method called search that takes a prompt and loops over all docs to rank them. create
//      a similar func that takes a k to return the top k results (use DSs to optimize the complexity).
// 3. use uper case in the lexer and in compute_tf_idf
// 4. make a simple http server
// 5. maybe add stemming (later)
// 6. create a simple cli tool but create a lib.rs to do that

use std::env::{self};

use RustSearchEngine::SearchEngineIndex;

fn main() -> Result<(), ()> {
    let mut args = env::args();
    let program_name = args.next().expect("program name is always provided");

    let command = args.next().ok_or_else(|| {
        print_usage(&program_name);
        eprintln!("ERROR: expected a command");
        eprintln!("Example: {program_name} <COMMAND>");
    })?;

    if command == "help" {
        print_usage(&program_name);
    } else if command == "index" {
        let docs_dir = args.next().ok_or_else(|| {
            print_usage(&program_name);
            eprintln!("ERROR: Expected path of the directory with documents to index");
            eprintln!("Example: {program_name} index path/to/dir/");
        })?;
        let index = SearchEngineIndex::new(docs_dir)?;

        let dest_path = args.next().unwrap_or_else(|| String::from("index.json"));
        index.save(dest_path)?;
    } else if command == "search" {
        todo!();
        // let index_path = "index path";
        // let prompt = "search prompt";
        // let search_engine = SearchEngine::new(index_path);
        // let docs = search_engine.search(prompt);
        // print_docs(docs);

        // start http server which internally creates a search_engine { index } and calls search_engine.search(prompt);
    } else if command == "serve" {
        todo!();
        // let index_path = args.next()
        // let index_path = "index path";
        // let search_engine = SearchEngine::new(index_path);
        // let server = Server::new(search_engine);
        // server.handle_requests();

        // create a search_engine { index } object then start an http server which internally
        // calls search_engine.search(prompt).
        // the search_engine object will load the index from a provided path (default path should
        // work too)
        // search_engine.search should tokenize the prompt and loop over all docs computing the
        // tfi-df of each.
    } else {
        todo!();
        // print_usage();
        // eprintln!("ERROR: invalid command {command}");
    }

    Ok(())
}

fn print_usage(program_name: &str) {
    eprintln!("USAGE:   {program_name} <COMMAND>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("help:    Show app usage and exit");
    eprintln!("index <DOCS-DIR> [<DEST-PATH>]:");
    eprintln!("     Create an index from a directory of documents and save it.");
    eprintln!("     Default destination is `index.json`");
    eprintln!();
}
