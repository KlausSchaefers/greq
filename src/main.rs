use clap::Parser;
use anyhow::Result;
use greq::search::{SearchEngine, SearchResult};
use greq::file_walker::FileWalker;
use colored::*;
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
    
    /// Maximum number of results to show
    #[arg(short = 'n', long, default_value = "20")]
    max_results: usize,
    
    /// Show line numbers
    #[arg(short = 'l', long)]
    line_numbers: bool,
    
    /// Context lines around matches
    #[arg(short = 'C', long, default_value = "2")]
    context: usize,
    
    /// Case insensitive search
    #[arg(short = 'i', long)]
    ignore_case: bool,
    
    /// Show only file names (no content)
    #[arg(short = 'f', long)]
    files_only: bool,
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

    for doc in &documents {
        println!("Indexed: {}", doc.file_path.to_string_lossy());
    }
    
    // Create search engine and perform search
    let mut search_engine = SearchEngine::new(documents);
    search_engine.set_case_sensitive(!cli.ignore_case);
    
    let results = search_engine.search(&cli.query, cli.max_results);
    
    if results.is_empty() {
        println!("No matches found for: {}", cli.query.yellow());
        return Ok(());
    }
    
    // Display results
    display_results(&results, &cli);
    
    Ok(())
}

fn display_results(results: &[SearchResult], cli: &Cli) {
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            println!();
        }
        
        // File header with score
        println!(
            "{}{}  {}",
            result.file_path.to_string_lossy().blue().bold(),
            if cli.line_numbers { format!(":{}", result.line_number) } else { String::new() },
            format!("(score: {:.3})", result.score).dimmed()
        );
        
        if !cli.files_only {
            // Show context with highlighted matches
            display_match_context(result, cli);
        }
    }
}

fn display_match_context(result: &SearchResult, cli: &Cli) {
    let lines: Vec<&str> = result.content.lines().collect();
    let line_idx = result.line_number.saturating_sub(1);
    
    let start = line_idx.saturating_sub(cli.context);
    let end = (line_idx + cli.context + 1).min(lines.len());
    
    for i in start..end {
        let line = lines.get(i).unwrap_or(&"");
        let line_num = i + 1;
        
        let prefix = if cli.line_numbers {
            format!("{:4}: ", line_num)
        } else {
            "      ".to_string()
        };
        
        if i == line_idx {
            // Highlight the matching line
            println!("{}{}", prefix.green(), highlight_query_in_line(line, &result.matched_text));
        } else {
            println!("{}{}", prefix.dimmed(), line.dimmed());
        }
    }
}

fn highlight_query_in_line(line: &str, query: &str) -> String {
    // Simple highlighting - replace with more sophisticated matching later
    line.replace(query, &query.yellow().to_string())
}