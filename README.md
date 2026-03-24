# Greq = Grep + Query

Greq is a CLI tool to search files and return relevant sections. It is different to `grep` in that it runs a simple linguistic analysis (BM25) on the text and ranks the file sections according to the importance regarding the query. It's like a mini search engine.

Greq will work better for longer queries, for single keyword queries `grep` is a better option.

## 🚀 Installation

### Download Pre-built Binaries (Recommended)

Pre-built binaries are automatically created for every commit to master. Download the latest release from the [Releases page](https://github.com/klausschaefers/greq/releases):


After downloading, make the binary executable (Linux/macOS):
```bash
chmod +x greq
```

### Build from Source

**Debug**
```bash
cargo build
```

**Release**
```bash
cargo build --release
```


## Usage
```bash
greq "search query" [path] [options]
```

### Examples
```bash
# Basic search with metadata and highlighting
greq "machine learning" docs/ -m -l

# Search with top 5 results and context
greq "rust programming" . --n 5 -C 2

# JSON output for scripting
greq "error handling" src/ --format json

# Search specific file types
greq "function" . --extensions "rs,py,js"

# Files only (no content)
greq "config" . -f

# Different chunk sizes for better context
greq "algorithms" . --chunk-size 300
```

### Options
```
    <QUERY>                  Search query
    [PATH]                   Directory or file to search [default: .]

  -e, --extensions <EXT>     File extensions (e.g., "rs,py,js")
  -n, --n <N>               Number of results [default: 3]
  -C, --context <CONTEXT>   Context chunks around matches [default: 1]
      --chunk-size <SIZE>   Chunk size in characters [default: 200]
      --format <FORMAT>     Output format: text or json [default: text]
  -f, --files-only          Show only file names
  -m, --show-meta           Show metadata (filename, score, position)
  -l, --highlight           Enable highlighting of search terms
  -h, --help                Print help
  -V, --version             Print version
```

## 🔍 How it Works

Greq uses the BM25 (Best Matching 25) ranking algorithm to score text relevance:

1. **Document Chunking**: Files are split into overlapping chunks for better context
2. **BM25 Scoring**: Each chunk is scored based on term frequency and document frequency
3. **Context Expansion**: Results include surrounding chunks for better context
4. **Ranking**: Results are sorted by relevance score

This makes Greq particularly effective for:
- ✅ Multi-word queries
- ✅ Concept-based searches  
- ✅ Finding relevant passages in large codebases
- ✅ Research and documentation search

Use `grep` for:
- ❌ Single keyword searches
- ❌ Exact pattern matching
- ❌ Regular expressions

