use crate::{Document, bm25::BM25, Tokenizer};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub score: f64,
    pub chunk_index: usize,
    pub content: String,
    pub matched_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

pub struct SearchEngine {
    documents: Vec<Document>,
    bm25: BM25,
    tokenizer: Tokenizer,
}

impl SearchEngine {
    pub fn new(documents: Vec<Document>, tokenizer: Tokenizer) -> Self {
        let bm25 = BM25::new(&documents, &tokenizer);
        Self {
            documents,
            bm25,
            tokenizer,
        }
    }
    
    /// Search for a query and return ranked chunk results with context
    pub fn search(&self, query: &str, n: usize, context: usize) -> Vec<SearchResult> {
        // Always use lowercase for search
        let query_terms = self.tokenizer.tokenize(&query.to_lowercase());
        if query_terms.is_empty() {
            return Vec::new();
        }
        
        // Score each chunk using BM25
        let mut chunk_results: Vec<(usize, usize, f64)> = Vec::new(); // (doc_idx, chunk_idx, score)
        
        for (doc_idx, document) in self.documents.iter().enumerate() {
            for (chunk_idx, chunk) in document.chunks.iter().enumerate() {
                let chunk_terms = self.tokenizer.tokenize(&chunk.content.to_lowercase());
                let score = self.bm25.score_chunk(&chunk_terms, &query_terms, doc_idx, chunk_idx);
                
                if score > 0.01 {
                    chunk_results.push((doc_idx, chunk_idx, score));
                }
            }
        }
        
        // Sort by score (highest first)
        chunk_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top N and expand with context
        let mut results = Vec::new();
        let top_chunks: Vec<_> = chunk_results.into_iter().take(n).collect();
        
        for (doc_idx, chunk_idx, score) in top_chunks {
            let document = &self.documents[doc_idx];
            
            // Get expanded chunks with context
            let expanded_chunks = document.expand_chunks(&[chunk_idx], context);
            
            for expanded_chunk in expanded_chunks {
                let matched_text = self.extract_matched_portion(&expanded_chunk.content, &query_terms);
                
                results.push(SearchResult {
                    file_path: document.file_path.clone(),
                    score,
                    chunk_index: chunk_idx,
                    content: expanded_chunk.content,
                    matched_text,
                    start_pos: expanded_chunk.start,
                    end_pos: expanded_chunk.end,
                });
            }
        }
        
        results
    }
    
    fn extract_matched_portion(&self, content: &str, query_terms: &[String]) -> String {
        // Find the first matching term in the content for highlighting
        for term in query_terms {
            if let Some(matched_word) = self.find_matching_word_in_content(content, term) {
                return matched_word;
            }
        }
        
        query_terms.first().unwrap_or(&String::new()).clone()
    }
    
    fn find_matching_word_in_content(&self, content: &str, term: &str) -> Option<String> {
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for word in words {
            // Clean the word of punctuation for comparison
            let clean_word: String = word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            
            // Check if this cleaned word contains our search term
            if clean_word.contains(term) {
                // Return the original word (with punctuation and original case)
                return Some(word.to_string());
            }
        }
        
        None
    }
    

}

#[cfg(test)]
mod tests {
    use crate::Tokenizer;

    #[test]
    fn test_tokenize() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Hello, world! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"this".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }
}