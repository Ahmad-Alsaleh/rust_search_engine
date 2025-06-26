# A local search engine in Rust

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

