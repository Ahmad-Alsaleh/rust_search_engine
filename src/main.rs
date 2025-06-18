// Steps:
// read all docs.
// parse xml
// tokenize all text
// find # of times every token/term appears in each doc and in how many docs it appears
//      (compute and cache tf-idf index to disk).
// create a simple http server on localhost

// TODO: be consistent when logging ERROR vs. WARN. (if it too much of headache to decide which to
// use, just use ERROR all the time. It is no big deal).

use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use xml::{reader::XmlEvent, EventReader};

use crate::tokenizer::{Token, Tokenizer};

mod tokenizer;

type TokensFreq = HashMap<Token, usize>;
type TokensFreqWithinDocs = HashMap<PathBuf, TokensFreq>;

fn main() -> Result<(), ()> {
    let mut tokens_freq_within_docs = TokensFreqWithinDocs::new();
    let mut tokens_freq_across_docs = TokensFreq::new();

    index_dir(
        Path::new("./documents-small"),
        &mut tokens_freq_within_docs,
        &mut tokens_freq_across_docs,
    )?;

    Ok(())
}

// TODO: consider using https://github.com/BurntSushi/walkdir if you wanna handle symbolic links.
fn index_dir(
    dir_path: &Path,
    tokens_freq_within_docs: &mut TokensFreqWithinDocs,
    tokens_freq_across_docs: &mut TokensFreq,
) -> Result<(), ()> {
    let dir_entries = std::fs::read_dir(dir_path).map_err(|err| {
        eprintln!(
            "ERROR: Couldn't open directory {path}: {err}",
            path = dir_path.display()
        )
    })?;

    'next_dir_entry: for dir_entry in dir_entries {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                eprintln!("ERROR: Couldn't get directory entry: {err}");
                continue 'next_dir_entry;
            }
        };

        let dir_entry_path = dir_entry.path();

        let dir_entry_type = match dir_entry.file_type() {
            Ok(dir_entry_type) => dir_entry_type,
            Err(err) => {
                eprintln!(
                    "ERROR: Couldn't extract file type of {path}: {err}",
                    path = dir_entry_path.display()
                );
                continue 'next_dir_entry;
            }
        };

        if dir_entry_type.is_dir() {
            if index_dir(
                &dir_entry_path,
                tokens_freq_within_docs,
                tokens_freq_across_docs,
            )
            .is_err()
            {
                eprintln!(
                    "WARN: Skipping directory {path} as it couldn't be parsed",
                    path = dir_entry_path.display()
                );
            };
            continue 'next_dir_entry;
        }

        if dir_entry_type.is_file()
            && dir_entry_path
                .extension()
                // NOTE: only files with the extension `xthml` are currently supported
                .is_some_and(|extension| extension == "xhtml")
        {
            eprintln!("INFO: Parsing {path}", path = dir_entry_path.display());
            if let Ok(tokens_freq_within_doc) = get_tokens_freq_within_doc(&dir_entry_path) {
                update_token_freq_accross_docs(
                    tokens_freq_across_docs,
                    tokens_freq_within_doc.keys(),
                );
                tokens_freq_within_docs.insert(dir_entry_path, tokens_freq_within_doc);
            } else {
                eprintln!(
                    "WARN: Skipping file {path} as it couldn't be parsed",
                    path = dir_entry_path.display()
                );
            }
        } else {
            eprintln!("INFO: Skipping {path}", path = dir_entry_path.display());
        }
    }

    Ok(())
}

fn update_token_freq_accross_docs<'a>(
    tokens_freq_across_docs: &mut TokensFreq,
    tokens_to_update: impl Iterator<Item = &'a Token>,
) {
    for token in tokens_to_update {
        // NOTE: the entry API is not used here to avoid unneeded `clone` when the token
        // already exists in the hashmap. `.entry()` takes ownernship over the token,
        // which we don't have here since we are given &Token, so we'll need to clone
        // everytime we call `.entry()`, even if we are not inserting the token. However,
        // when `.get_mut`, we are cloning only when inserting the token to the hashmap.
        if let Some(count) = tokens_freq_across_docs.get_mut(token) {
            *count += 1;
        } else {
            tokens_freq_across_docs.insert(token.clone(), 1);
        }
    }
}

fn get_tokens_freq_within_doc(path: &Path) -> Result<TokensFreq, ()> {
    let file = File::open(path).map_err(|err| {
        eprintln!(
            "ERROR: Couldn't open file {path} to parse it: {err}",
            path = path.display()
        );
    })?;
    let file = BufReader::new(file);

    let mut tokens_freq = TokensFreq::new();

    // TODO: consider using `xml::ParserConfig` to trim whitespance and disable unneeded events
    // (such as comments)
    'next_xml_event: for xml_event in EventReader::new(file) {
        let xml_event = match xml_event {
            Ok(xml_event) => xml_event,
            Err(err) => {
                eprintln!("WARN: Couldn't parse next XML event: {err}");
                continue 'next_xml_event;
            }
        };

        if let XmlEvent::Characters(text) = xml_event {
            let text = text.chars().collect::<Vec<_>>();
            for token in Tokenizer::new(&text) {
                tokens_freq
                    .entry(token)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
    }

    Ok(tokens_freq)
}
