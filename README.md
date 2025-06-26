# A local search engine in Rust

## Technical Overview

This search engine implements a classic information retrieval system using TF-IDF (Term Frequency-Inverse Document Frequency) scoring. Here's how it works:

**Document Processing & Indexing:**
- Parses XHTML documents using XML event streaming to extract text content
- Tokenizes text by splitting on whitespace and categorizing tokens as alphabetic, numeric, or symbolic
- Normalizes tokens to uppercase for case-insensitive matching
- Builds an inverted index that tracks token frequencies both within individual documents and across the entire document collection

**Search Algorithm:**
- Query terms are tokenized using the same process as document indexing
- For each document, calculates a relevance score by summing TF-IDF values for each query term
- Results are ranked by descending relevance score and filtered to exclude documents with zero relevance

**Architecture:**
- **Indexer**: Recursively traverses directories, processes XHTML files, and serializes the index to JSON
- **Search Engine**: Loads the pre-built index and performs TF-IDF scoring for queries
- **Web Interface**: HTTP server with a clean frontend for interactive searching
- **CLI Interface**: Command-line tools for indexing and searching

The system is designed for fast search performance by pre-computing document statistics during indexing, allowing queries to be processed efficiently without re-parsing documents.

## Usage

NOTE: only files with the extension `xhtml` are currently supported. More file extensions can be added easily in the future.

First, create the index for the search engine to use:

```bash
cargo run -- index <DOCS-DIR>
```

Replace `<DOCS-DIR>` with the actual path of the direcotory containing the documents you want to index.

Then, to search from the command line, use the following:

```bash
cargo run -- search 'you search query!'
```

Or, start a local http server for a better UI:

```bash
# you can change the port and host, or use the defaults
cargo run -- serve --port=5512 --host=127.0.0.1
```

For more details, use the help command:

```bash
cargo run -- help
```
