use clap::Parser;
use anyhow::Result;
use greq::{SearchEngine, SearchResult, FileWalker, Tokenizer, SubTokenizer, TokenizerTrait, Document};
use colored::*;
use serde_json;
use std::path::PathBuf;
use std::io::{self, IsTerminal, Read};

#[derive(Parser)]
#[command(name = "greq")]
#[command(about = "Grep + Query: A file search tool with BM25 ranking")]
#[command(version)]
struct Cli {
    /// Search query
    query: String,
    
    /// Directory or file to search in
    #[arg(default_value = ".")]
    path: PathBuf,
    
    /// File extensions to include (e.g., "rs,py,js")
    #[arg(short, long)]
    extensions: Option<String>,
    
    /// Number of top results to show
    #[arg(short = 'n', long, default_value = "3")]
    n: usize,
    
    /// Context chunks around matches
    #[arg(short = 'C', long, default_value = "1")]
    context: usize,
    
    /// Maximum chunk size in characters
    #[arg(short = 's', long, default_value = "200")]
    chunk_size: usize,
    
    /// Output format (text or json)
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
    
    /// Show metadata (filename, score, position)
    #[arg(short = 'm', long, default_value = "false")]
    show_meta: bool,
    
    /// Enable highlighting of search terms
    #[arg(short = 'l', long, default_value = "false")]
    highlight: bool,
    
    /// Sub-token length for fuzzy matching (use SubTokenizer if > 3)
    #[arg(long = "sub-token", short = 't', default_value = "0")]
    sub_token: usize,

    /// Embedding weight for combining BM25 and embedding scores
    #[arg(long = "embedding-weight", short = 'w', default_value = "0")]
    embedding_weight: f64,
    
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Parse file extensions if provided
    let extensions: Option<Vec<String>> = cli.extensions.clone().map(|ext_str| {
        ext_str.split(',').map(|s| s.trim().to_string()).collect()
    });
    
    // Get documents from either stdin or file walker
    let documents = get_documents(cli.path.clone(), extensions, cli.chunk_size)?;
    
    if documents.is_empty() {
        println!("No files found to search");
        return Ok(());
    }
    
    // Create tokenizer and search engine
    let tokenizer = create_tokenizer(cli.sub_token);
    let search_engine = SearchEngine::new(documents, tokenizer, cli.embedding_weight);
    let results = search_engine.search(&cli.query, cli.n, cli.context);
    
    if results.is_empty() {
        if cli.format == "json" {
            println!("[]");
        } else {
            println!("No matches found for: {}", cli.query.yellow());
        }
        return Ok(());
    }
    
    // Display results
    if cli.format == "json" {
        display_json_results(&results)?;
    } else {
        display_text_results(&results, &cli);
    }
    
    Ok(())
}

fn get_documents(
    path: PathBuf,
    extensions: Option<Vec<String>>,
    chunk_size: usize,
) -> Result<Vec<Document>> {
    // Check if we should read from stdin (piped input)
    // Only read from stdin if it's not a terminal AND there's actually data available
    if !io::stdin().is_terminal() {
        // Try to read from stdin non-blockingly to see if there's data
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) if !buffer.trim().is_empty() => {
                // We have actual data from stdin
                Ok(vec![Document::new_with_chunk_size(
                    PathBuf::from("<stdin>"),
                    buffer,
                    chunk_size
                )])
            }
            Ok(_) => {
                // stdin is available but empty, treat as no stdin input
                let file_walker = FileWalker::new_with_chunk_size(path, extensions, chunk_size);
                file_walker.collect_documents()
            }
            Err(_) => {
                // Error reading stdin, fall back to file walker
                let file_walker = FileWalker::new_with_chunk_size(path, extensions, chunk_size);
                file_walker.collect_documents()
            }
        }
    } else {
        // Walk files and collect content
        let file_walker = FileWalker::new_with_chunk_size(path, extensions, chunk_size);
        file_walker.collect_documents()
    }
}

fn display_json_results(results: &[SearchResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{}", json);
    Ok(())
}

fn display_text_results(results: &[SearchResult], cli: &Cli) {
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            println!();
        }
        
        if cli.show_meta {
            // File header with score and position
            println!(
                "{}:{}  {}",
                result.file_path.to_string_lossy().blue().bold(),
                result.start_pos,
                format!("(score: {:.3})", result.score).dimmed()
            );
        } 
        
        // Show content with or without highlighting
        if cli.highlight {
            let highlighted = highlight_query_in_text(&result.content, &result.matched_text);
            println!("{}", highlighted);
        } else {
            println!("{}", result.content);
        }
    }
}

fn highlight_query_in_text(content: &str, matched_word: &str) -> String {
    // Case-insensitive highlighting of the matched word
    let content_lower = content.to_lowercase();
    let matched_word_lower = matched_word.to_lowercase();
    
    // If the matched word appears in the content, highlight it
    if content_lower.contains(&matched_word_lower) {
        // Find all words in content and highlight matches
        let words: Vec<&str> = content.split_whitespace().collect();
        let highlighted_words: Vec<String> = words.into_iter().map(|word| {
            let clean_word = word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            
            // If this word contains our search term, highlight it
            if clean_word.contains(&matched_word_lower) {
                word.yellow().to_string()
            } else {
                word.to_string()
            }
        }).collect();
        
        highlighted_words.join(" ")
    } else {
        // Fallback to simple replacement if word-based highlighting fails
        content.to_string()
    }
}

fn create_tokenizer(sub_token: usize) -> Box<dyn TokenizerTrait> {
    if sub_token > 3 {
        Box::new(SubTokenizer::new(sub_token))
    } else {
        Box::new(Tokenizer::new())
    }
}