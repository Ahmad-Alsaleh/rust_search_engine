// Steps:
// read all docs.
// parse xml
// tokenize all text
// find # of times every token/term appears in each doc and in how many docs it appears
//      (compute and cache tf-idf index to disk).
// create a simple http server on localhost

use std::{fs::File, io::BufReader};
use xml::{reader::XmlEvent, EventReader};

use crate::tokenizer::Tokenizer;

mod tokenizer;

fn main() -> Result<(), ()> {
    let file = File::open("documents/gl2/glFog.xhtml").unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    for event in parser {
        let event = event.map_err(|err| {
            println!("ERROR: Couldn't parse next XML event: {err}");
        })?;

        if let XmlEvent::Characters(text) = event {
            let text = text.chars().collect::<Vec<_>>();
            for token in Tokenizer::new(&text) {
                println!("{token}");
            }
        }
    }

    Ok(())
}
