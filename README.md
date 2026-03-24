# Greq = Grep + Query

Greq is a CLI tool to search files and return relevant sections. It is different to `grep` in that it runs a simple linguistic analysis (BM25) on the text and ranks the file sections accoridng to the importance regarding the query. Its is like a mini search engine.

Greq will work better for longer queries, for single keyword queries `grep` is a better option.



# Build


Debug
```
cargo build
```

Release
```
cargo build --release
```


# Examples

```sh
# Plain text (no highlighting, no metadata) - default
cargo run -- "karate" tests/data/ --n 2

# With highlighting only
cargo run -- "karate" tests/data/ --n 1 -l

# With both metadata and highlighting
cargo run -- "karate" tests/data/ --n 1 -m -l

# JSON format (highlighting parameter ignored)
cargo run -- "karate" tests/data/ --n 1 --format json
```