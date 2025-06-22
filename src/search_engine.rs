use std::path::Path;

use crate::{
    index::TokensFreqWithinDoc,
    tokenizer::{Token, Tokenizer},
    Result, SearchEngineIndex,
};

pub struct SearchResult<'a> {
    pub doc_path: &'a Path,
    pub importance_score: f32,
}

pub struct SearchEngine {
    pub(crate) index: SearchEngineIndex,
}

// public methods
impl SearchEngine {
    pub fn new(index_path: impl AsRef<Path>) -> Result<Self> {
        let index_path = index_path.as_ref();
        let index = SearchEngineIndex::load(index_path)?;

        Ok(Self { index })
    }

    pub fn search(&self, prompt: &str) -> Vec<SearchResult> {
        let prompt = prompt.chars().collect::<Vec<_>>();

        let mut search_results: Vec<_> = self
            .index
            .tokens_freq_within_docs
            .keys()
            .map(|doc_path| {
                let importance_score = Tokenizer::new(&prompt)
                    .map(|token| {
                        self.compute_tf_idf(&token, doc_path)
                            .expect("we are passing valid file paths by calling `.keys()`")
                    })
                    .sum();
                SearchResult {
                    doc_path,
                    importance_score,
                }
            })
            .filter(|search_result| search_result.importance_score > 0.)
            .collect();

        search_results.sort_unstable_by(|a, b| b.importance_score.total_cmp(&a.importance_score));

        search_results
    }
}

// private methods
impl SearchEngine {
    fn compute_tf_idf(&self, token: &Token, document_path: &Path) -> Result<f32> {
        Ok(self.compute_tf(token, document_path)? * self.compute_idf(token))
    }

    fn compute_tf(&self, token: &Token, document_path: &Path) -> Result<f32> {
        let TokensFreqWithinDoc {
            tokens_freq,
            num_of_tokens,
        } = self
            .index
            .tokens_freq_within_docs
            .get(document_path)
            .ok_or_else(|| {
                eprintln!(
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

    fn compute_idf(&self, token: &Token) -> f32 {
        let num_of_docs = self.index.tokens_freq_within_docs.len();
        let token_freq = self.index.tokens_freq_across_docs.get(token).unwrap_or(&1);
        f32::log10(num_of_docs as f32 / *token_freq as f32)
    }
}
