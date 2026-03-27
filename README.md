# Greq = Grep + Query

Greq is a CLI tool that searches files and returns the most relevant sections. It’s designed to provide concise, high-signal context for AI agents, helping reduce token usage while preserving the information that matters.

Unlike grep, which performs exact pattern matching, Greq applies lightweight linguistic ranking (BM25) to score and sort file sections based on how relevant they are to your query. In practice, it behaves like a small search engine in your shell.

Greq works best with natural-language or multi-word queries. For simple single-keyword searches, grep is usually the faster and more appropriate tool.

## 🚀 Quick Install (Recommended)

Install with a single command:
```bash
curl -sSL https://raw.githubusercontent.com/KlausSchaefers/greq/main/install.sh | bash
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


## Examples
```bash
# Basic search with metadata and highlighting
greq "machine learning" docs/ -m -l

# Search with top 5 results and context
greq "rust programming" . --n 5 -C 2

# JSON output for scripting
greq "error handling" src/ -f json

# Search specific file types
greq "function" . --extensions "rs,py,js"

# Different chunk sizes for better context
greq "algorithms" . -s 300

# Fuzzy matching with sub-tokens (great for partial words)
greq "capo" docs/ --sub-token 4     # Finds "capoeira", "capon", etc.
greq "config" . -t 5                # Short form: finds "configuration", "configure"

# Regular vs fuzzy search comparison
greq "capo" docs/                   # No matches (exact word search)
greq "capo" docs/ -t 4              # Finds "capoeira" (fuzzy sub-token search)
```

### Options
```
    <QUERY>                  Search query
    [PATH]                   Directory or file to search [default: .]

  -e, --extensions <EXT>     File extensions (e.g., "rs,py,js")
  -n, --n <N>               Number of results [default: 3]
  -C, --context <CONTEXT>   Context chunks around matches [default: 1]
  -s, --chunk-size <SIZE>   Chunk size in characters [default: 200]
  -f, --format <FORMAT>     Output format: text or json [default: text]
  -m, --show-meta           Show metadata (filename, score, position)
  -l, --highlight           Enable highlighting of search terms
  -t, --sub-token <LENGTH>  Sub-token length for fuzzy matching (>3 enables fuzzy search) [default: 0]
  -h, --help                Print help
  -V, --version             Print version
```

### Fuzzy Search with Sub-tokens

Greq supports fuzzy matching using overlapping sub-tokens. This is particularly useful when:
- You remember only part of a word
- Searching for compound words or technical terms
- Dealing with typos or variations


**Note**: This will be slower and use more memory. 

**How it works:**
- Words are split into overlapping sub-sequences of specified length
- Example: "capoeira" with `--sub-token 4` becomes ["capo", "apoe", "poei", "oeir", "eira"]
- Your search term "capo" will match because it appears as a sub-token

**Usage:**
```bash
# Enable fuzzy search with 4-character sub-tokens
greq "capo" docs/ --sub-token 4

# Works with any sub-token length > 3
greq "config" . -t 5    # Finds "configuration", "configure", etc.
```



### Manual Download

Alternatively, download manually from the [Releases page](https://github.com/klausschaefers/greq/releases):

- **Linux x86_64**: `greq` 
- **Linux ARM64**: `greq`
- **Windows**: `greq.exe`
- **macOS Intel**: `greq`
- **macOS Apple Silicon**: `greq`

After downloading, make the binary executable (Linux/macOS):
```bash
chmod +x greq
```


### Uninstall

To remove greq from your system:

**Quick uninstall (Recommended):**
```bash
curl -sSL https://raw.githubusercontent.com/KlausSchaefers/greq/main/uninstall.sh | bash
```

**Manual removal:**
```bash
# Remove the binary
sudo rm /usr/local/bin/greq

# For Windows (if applicable)
# rm /usr/local/bin/greq.exe
```

#### macOS Security Notice
On macOS, you may see "cannot be verified" error. This is because the binary isn't signed with an Apple Developer certificate. 

**Option 1: Remove quarantine attribute (Recommended)**
```bash
xattr -d com.apple.quarantine greq
```

**Option 2: Right-click method**
1. Right-click the `greq` binary in Finder
2. Select "Open" 
3. Click "Open" in the security dialog

**Option 3: System Settings**
1. Try to run `./greq` (it will fail)
2. Go to System Settings > Privacy & Security
3. Click "Allow Anyway" next to the greq message
4. Try running `./greq` again and confirm

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


# Thanks

We use the [fastembed-rs](https://github.com/Anush008/fastembed-rs) lib from Anush008. Please give him a star!