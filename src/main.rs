use clap::Parser;
use anyhow::Result;
use greq::{SearchEngine, SearchResult, FileWalker};
use colored::*;
use serde_json;
use std::path::PathBuf;

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
    #[arg(long, default_value = "200")]
    chunk_size: usize,
    
    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    format: String,
    
    /// Show only file names (no content)
    #[arg(short = 'f', long)]
    files_only: bool,
    
    /// Show metadata (filename, score, position)
    #[arg(short = 'm', long, default_value = "false")]
    show_meta: bool,
    
    /// Enable highlighting of search terms
    #[arg(short = 'l', long, default_value = "false")]
    highlight: bool,

    
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Parse file extensions if provided
    let extensions: Option<Vec<String>> = cli.extensions.clone().map(|ext_str| {
        ext_str.split(',').map(|s| s.trim().to_string()).collect()
    });
    
    // Walk files and collect content
    let file_walker = FileWalker::new(cli.path.clone(), extensions);
    let documents = file_walker.collect_documents()?;
    
    if documents.is_empty() {
        println!("No files found to search");
        return Ok(());
    }
    
    // Create search engine and perform search
    let search_engine = SearchEngine::new(documents);
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
            // File header with score
            println!(
                "{}:{}  {}",
                result.file_path.to_string_lossy().blue().bold(),
                result.start_pos,
                format!("(score: {:.3})", result.score).dimmed()
            );
        }
        
        if !cli.files_only {
            // Show content with or without highlighting
            if cli.highlight {
                let highlighted = highlight_query_in_text(&result.content, &result.matched_text);
                println!("{}", highlighted);
            } else {
                println!("{}", result.content);
            }
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