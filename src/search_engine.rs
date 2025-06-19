use std::path::Path;

use crate::{index::TokensFreqWithinDoc, tokenizer::Token};

pub(crate) struct SearchEngine {
    index: crate::SearchEngineIndex,
}

impl SearchEngine {
    fn compute_tf_idf(&self, token: &Token, document_path: &Path) -> Result<f32, ()> {
        Ok(self.compute_tf(token, document_path)? * self.compute_idf(token))
    }

    fn compute_tf(&self, token: &Token, document_path: &Path) -> Result<f32, ()> {
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
