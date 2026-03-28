pub mod search;
pub mod file_walker;
pub mod bm25;
pub mod config;
pub mod tokenizer;
pub mod embeddings;
pub mod parser;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

// Re-export main types
pub use search::{SearchEngine, SearchResult};
pub use file_walker::FileWalker;
pub use tokenizer::{Tokenizer, SubTokenizer, TokenizerTrait};
pub use embeddings::Embeddings;
pub use parser::{Parser, DefaultParser, MarkdownParser, create_parser};

/// A chunk of text from a document with position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub content: String,
    pub start: usize,
    pub end: usize,
}

impl Chunk {
    pub fn new(content: String, start: usize, end: usize) -> Self {
        Self { content, start, end }
    }
    
    /// Check if this chunk overlaps with another chunk
    pub fn overlaps_with(&self, other: &Chunk) -> bool {
        !(self.end <= other.start || other.end <= self.start)
    }
    
    /// Merge this chunk with another overlapping chunk
    pub fn merge_with(&self, other: &Chunk) -> Chunk {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        // We'll need the original content to reconstruct the merged chunk
        // For now, concatenate the content
        let content = if self.start <= other.start {
            format!("{} {}", self.content.trim(), other.content.trim())
        } else {
            format!("{} {}", other.content.trim(), self.content.trim())
        };
        Chunk::new(content, start, end)
    }
}

/// Document represents a searchable text document
#[derive(Debug, Clone)]
pub struct Document {
    pub file_path: PathBuf,
    pub content: String,
    pub chunks: Vec<Chunk>,
}

impl Document {
    pub fn new(file_path: PathBuf, content: String) -> Self {
        Self::new_with_chunk_size(file_path, content, 200)
    }
    
    pub fn new_with_chunk_size(file_path: PathBuf, content: String, max_chunk_size: usize) -> Self {
        let parser = create_parser(&file_path);
        Self::new_with_parser_boxed(file_path, content, max_chunk_size, parser)
    }
    
    pub fn new_with_parser<P: Parser>(
        file_path: PathBuf, 
        content: String, 
        max_chunk_size: usize, 
        parser: &P
    ) -> Self {
        let chunks = parser.parse(&content, max_chunk_size);
        Self {
            file_path,
            content,
            chunks,
        }
    }
    
    pub fn new_with_parser_boxed(
        file_path: PathBuf, 
        content: String, 
        max_chunk_size: usize, 
        parser: Box<dyn Parser>
    ) -> Self {
        let chunks = parser.parse(&content, max_chunk_size);
        Self {
            file_path,
            content,
            chunks,
        }
    }
    
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
    
    pub fn get_chunk(&self, index: usize) -> Option<&Chunk> {
        self.chunks.get(index)
    }
    
    /// Expand chunks with context, merging overlapping results
    pub fn expand_chunks(&self, chunk_indices: &[usize], context: usize) -> Vec<Chunk> {
        let mut expanded = Vec::new();
        
        for &index in chunk_indices {
            let start_idx = index.saturating_sub(context);
            let end_idx = (index + context + 1).min(self.chunks.len());
            
            // Collect chunks with context
            let mut context_chunks = Vec::new();
            for i in start_idx..end_idx {
                if let Some(chunk) = self.get_chunk(i) {
                    context_chunks.push(chunk.clone());
                }
            }
            
            // Merge context chunks into one
            if let Some(merged) = self.merge_adjacent_chunks(&context_chunks) {
                expanded.push(merged);
            }
        }
        
        // Remove overlapping results
        self.merge_overlapping_chunks(&mut expanded);
        expanded
    }
    
    fn merge_adjacent_chunks(&self, chunks: &[Chunk]) -> Option<Chunk> {
        if chunks.is_empty() {
            return None;
        }
        
        if chunks.len() == 1 {
            return Some(chunks[0].clone());
        }
        
        let start = chunks.first().unwrap().start;
        let end = chunks.last().unwrap().end;
        
        // Extract the content from the original text
        let content = self.content[start..end].to_string();
        
        Some(Chunk::new(content, start, end))
    }
    
    fn merge_overlapping_chunks(&self, chunks: &mut Vec<Chunk>) {
        if chunks.len() <= 1 {
            return;
        }
        
        // Sort by start position
        chunks.sort_by_key(|c| c.start);
        
        let mut merged = Vec::new();
        let mut current = chunks[0].clone();
        
        for chunk in chunks.iter().skip(1) {
            if current.overlaps_with(chunk) {
                current = self.merge_chunks_from_content(&current, chunk);
            } else {
                merged.push(current);
                current = chunk.clone();
            }
        }
        merged.push(current);
        
        chunks.clear();
        chunks.extend(merged);
    }
    
    fn merge_chunks_from_content(&self, chunk1: &Chunk, chunk2: &Chunk) -> Chunk {
        let start = chunk1.start.min(chunk2.start);
        let end = chunk1.end.max(chunk2.end);
        let content = self.content[start..end].to_string();
        Chunk::new(content, start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_document_creation() {
        let content = "This is a test document with multiple words that should be chunked properly".to_string();
        let doc = Document::new_with_chunk_size(PathBuf::from("test.txt"), content, 20);
        
        assert!(doc.chunk_count() > 0);
        assert!(doc.get_chunk(0).is_some());
    }
    
    #[test]
    fn test_chunking_no_word_split() {
        let content = "ab cd de fe".to_string();
        let doc = Document::new_with_chunk_size(PathBuf::from("test.txt"), content, 9);
        
        let chunks = &doc.chunks;
        assert!(chunks.len() >= 1);
        
        // First chunk should be "ab cd de" (8 chars, within limit, doesn't split "fe")
        assert_eq!(chunks[0].content, "ab cd de");
        
        if chunks.len() > 1 {
            assert_eq!(chunks[1].content, "fe");
        }
    }
}