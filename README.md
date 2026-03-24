# Greq = Grep + Query

Greq is a CLI tool to search files and return relevant sections. It is different to `grep` in that it runs a simple linguistic analysis (BM25) on the text and ranks the file sections accoridng to the importance regarding the query. Its is like a mini search engine.

Greq will work better for longer queries, for single keyword queries `grep` is a better option.

#Examples

```sh
# Search for "BM25" in Rust files
cargo run -- "BM25" src/ --extensions rs

# Case-insensitive search with context
cargo run -- "search" --ignore-case --context 3

# Show only filenames
cargo run -- "query" --files-only
```