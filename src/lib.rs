mod index;
mod search_engine;
mod tokenizer;

pub type Result<T> = std::result::Result<T, ()>;

pub use index::SearchEngineIndex;
pub use search_engine::{SearchEngine, SearchResult};
