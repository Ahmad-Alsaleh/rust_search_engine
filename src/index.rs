use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use xml::{reader::XmlEvent, EventReader};

use crate::tokenizer::{Token, Tokenizer};

type TokensFreq = HashMap<Token, usize>;
type TokensFreqWithinDocs = HashMap<PathBuf, TokensFreqWithinDoc>;

struct TokensFreqWithinDoc {
    tokens_freq: TokensFreq,
    num_of_tokens: usize,
}

#[derive(Default)]
pub struct SearchEngineIndex {
    tokens_freq_within_docs: TokensFreqWithinDocs,
    tokens_freq_across_docs: TokensFreq,
}

impl SearchEngineIndex {
    pub fn new(dir_path: impl AsRef<Path>) -> Result<Self, ()> {
        let dir_path = dir_path.as_ref();

        let mut search_engine_index: Self = Default::default();
        search_engine_index.index_dir(dir_path)?;

        Ok(search_engine_index)
    }

    // TODO: try to change token: &String to token: &str
    pub fn compute_tf_idf(&self, token: &Token, document_path: &Path) -> Result<f32, ()> {
        Ok(self.compute_tf(token, document_path)? * self.compute_idf(token))
    }

    fn compute_tf(&self, token: &Token, document_path: &Path) -> Result<f32, ()> {
        let TokensFreqWithinDoc {
            tokens_freq,
            num_of_tokens,
        } = self
            .tokens_freq_within_docs
            .get(document_path)
            .ok_or_else(|| {
                println!(
                    "ERROR: Invalid document path {path}",
                    path = document_path.display()
                )
            })?;

        let tf = if let Some(&token_freq) = tokens_freq.get(token) {
            token_freq as f32 / *num_of_tokens as f32
        } else {
            0.
        };

        Ok(tf)
    }

    pub fn compute_idf(&self, token: &Token) -> f32 {
        let num_of_docs = self.tokens_freq_within_docs.len();
        let token_freq = self.tokens_freq_across_docs.get(token).unwrap_or(&1);
        f32::log10(num_of_docs as f32 / *token_freq as f32)
    }

    // TODO: handle symbolic links. consider using https://github.com/BurntSushi/walkdir for that
    fn index_dir(&mut self, dir_path: &Path) -> Result<(), ()> {
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
                    // NOTE: only files with the extension `xthml` are currently supported
                    .is_some_and(|extension| extension == "xhtml")
            {
                eprintln!("INFO: Parsing {path}", path = dir_entry_path.display());
                if let Ok(tokens_freq_within_doc) = get_tokens_freq_within_doc(&dir_entry_path) {
                    self.update_token_freq_accross_docs(tokens_freq_within_doc.tokens_freq.keys());
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

    fn update_token_freq_accross_docs<'a>(
        &mut self,
        tokens_to_update: impl Iterator<Item = &'a Token>,
    ) {
        for token in tokens_to_update {
            // NOTE: the entry API is not used here to avoid unneeded `clone` when the token
            // already exists in the hashmap. `.entry()` takes ownernship over the token,
            // which we don't have here since we are given &Token, so we'll need to clone
            // everytime we call `.entry()`, even if we are not inserting the token. However,
            // when `.get_mut`, we are cloning only when inserting the token to the hashmap.
            if let Some(count) = self.tokens_freq_across_docs.get_mut(token) {
                *count += 1;
            } else {
                self.tokens_freq_across_docs.insert(token.clone(), 1);
            }
        }
    }
}

fn get_tokens_freq_within_doc(path: &Path) -> Result<TokensFreqWithinDoc, ()> {
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

    let num_of_tokens = tokens_freq.values().sum();

    Ok(TokensFreqWithinDoc {
        tokens_freq,
        num_of_tokens,
    })
}
