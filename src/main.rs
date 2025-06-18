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
// 6. create a simple cli tool

use std::path::Path;

use crate::index::SearchEngineIndex;

mod index;
mod tokenizer;

fn main() -> Result<(), ()> {
    let index = SearchEngineIndex::new("documents/")?;

    // for (path, tokens_freq) in index.tokens_freq_within_docs.iter().take(3) {
    //     println!("{path}:", path = path.display());
    //     for (token, freq) in tokens_freq.iter().take(5) {
    //         println!("    {token}: {freq}")
    //     }
    // }

    let tf_idf = index.compute_tf_idf(&"FOG".into(), Path::new("documents/es1/glFog.xhtml"))?;
    println!("{tf_idf}");

    Ok(())
}
