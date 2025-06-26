use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use xml::{reader::XmlEvent, EventReader};

use crate::{
    tokenizer::{Token, Tokenizer},
    Result,
};

type TokensFreq = HashMap<Token, usize>;
type TokensFreqWithinDocs = HashMap<PathBuf, TokensFreqWithinDoc>;

#[derive(Serialize, Deserialize)]
pub(crate) struct TokensFreqWithinDoc {
    pub(crate) tokens_freq: TokensFreq,
    pub(crate) num_of_tokens: usize,
}

// TODO: consider saving the tf-idf value of each token instead of the token frequency
// within and between docs. This might make searching more effecient and potentially
// reduce the index size since each token will be indexed exactly once
#[derive(Default, Serialize, Deserialize)]
pub struct SearchEngineIndex {
    pub(crate) tokens_freq_within_docs: TokensFreqWithinDocs,
    pub(crate) tokens_freq_across_docs: TokensFreq,
}

// public methods
impl SearchEngineIndex {
    pub fn new(docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref();

        let mut search_engine_index = Self::default();
        search_engine_index.index_dir(docs_dir)?;

        Ok(search_engine_index)
    }

    pub fn save(&self, dest: impl AsRef<Path>) -> Result<()> {
        let file = File::create(dest)
            .map_err(|err| eprintln!("ERROR: Failed to create file while saving index: {err}"))?;
        let file = BufWriter::new(file);

        serde_json::to_writer(file, self)
            .map_err(|err| eprintln!("ERROR: Failed to serialize index: {err}"))?;

        Ok(())
    }
}

// private methods
impl SearchEngineIndex {
    pub(crate) fn load(index_path: &Path) -> Result<Self> {
        let file = File::open(index_path)
            .map_err(|err| eprintln!("ERROR: Failed to open the index file: {err}"))?;
        let file = BufReader::new(file);

        let index = serde_json::from_reader(file)
            .map_err(|err| eprintln!("ERROR: Failed to deserialize index: {err}"))?;

        Ok(index)
    }

    // TODO: handle symbolic links. consider using https://github.com/BurntSushi/walkdir for that
    fn index_dir(&mut self, dir_path: &Path) -> Result<()> {
        let dir_entries = std::fs::read_dir(dir_path).map_err(|err| {
            eprintln!(
                "ERROR: Couldn't open directory {path}: {err}",
                path = dir_path.display()
            );
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
                if self.index_dir(&dir_entry_path).is_err() {
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
                    // NOTE: only files with the extension `xhtml` are currently supported
                    .is_some_and(|extension| extension == "xhtml")
            {
                eprintln!("INFO: Parsing {path}", path = dir_entry_path.display());
                if let Ok(tokens_freq_within_doc) = get_tokens_freq_within_doc(&dir_entry_path) {
                    self.update_token_freq_across_docs(tokens_freq_within_doc.tokens_freq.keys());
                    self.tokens_freq_within_docs
                        .insert(dir_entry_path, tokens_freq_within_doc);
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

    fn update_token_freq_across_docs<'a>(
        &mut self,
        tokens_to_update: impl Iterator<Item = &'a Token>,
    ) {
        for token in tokens_to_update {
            // NOTE: the entry API is not used here to avoid unneeded `clone` when the token
            // already exists in the hashmap. `.entry()` takes ownership over the token,
            // which we don't have here since we are given &Token, so we'll need to clone
            // every time we call `.entry()`, even if we are not inserting the token. However,
            // when `.get_mut`, we are cloning only when inserting the token to the hashmap.
            if let Some(count) = self.tokens_freq_across_docs.get_mut(token) {
                *count += 1;
            } else {
                self.tokens_freq_across_docs.insert(token.clone(), 1);
            }
        }
    }
}

fn get_tokens_freq_within_doc(path: &Path) -> Result<TokensFreqWithinDoc> {
    let file = File::open(path).map_err(|err| {
        eprintln!(
            "ERROR: Couldn't open file {path} to parse it: {err}",
            path = path.display()
        );
    })?;
    let file = BufReader::new(file);

    let mut tokens_freq = TokensFreq::new();

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

    let num_of_tokens = tokens_freq.values().sum();

    Ok(TokensFreqWithinDoc {
        tokens_freq,
        num_of_tokens,
    })
}
