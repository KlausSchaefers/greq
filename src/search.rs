use crate::{Document, bm25::BM25};
use std::path::PathBuf;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub score: f64,
    pub line_number: usize,
    pub content: String,
    pub matched_text: String,
}

pub struct SearchEngine {
    documents: Vec<Document>,
    bm25: BM25,
    case_sensitive: bool,
}

impl SearchEngine {
    pub fn new(documents: Vec<Document>) -> Self {
        let bm25 = BM25::new(&documents);
        Self {
            documents,
            bm25,
            case_sensitive: false, // Default to case-insensitive
        }
    }
    
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }
    
    /// Search for a query and return ranked results
    pub fn search(&self, query: &str, max_results: usize) -> Vec<SearchResult> {
        let query_terms = self.tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }
        
        // Score each document using BM25
        let mut results: Vec<SearchResult> = self.documents
            .par_iter()
            .enumerate()
            .filter_map(|(doc_idx, doc)| {
                self.search_document(doc, doc_idx, &query_terms, query)
            })
            .collect();
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        results.truncate(max_results);
        results
    }
    
    fn search_document(
        &self, 
        document: &Document, 
        doc_idx: usize, 
        query_terms: &[String], 
        _original_query: &str
    ) -> Option<SearchResult> {
        // Find the best matching line in the document
        let mut best_score = 0.0;
        let mut best_line = 1;
        let mut matched_text = String::new();
        
        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_terms = self.tokenize(line);
            let line_score = self.bm25.score_line(&line_terms, query_terms, doc_idx);
            
            if line_score > best_score {
                best_score = line_score;
                best_line = line_idx + 1;
                matched_text = self.extract_matched_portion(line, query_terms);
            }
        }
        
        // Only return results with meaningful scores
        if best_score > 0.01 {
            Some(SearchResult {
                file_path: document.file_path.clone(),
                score: best_score,
                line_number: best_line,
                content: document.content.clone(),
                matched_text,
            })
        } else {
            None
        }
    }
    
    fn extract_matched_portion(&self, line: &str, query_terms: &[String]) -> String {
        // Find the first matching term in the line for highlighting
        for term in query_terms {
            let search_line = line.to_lowercase();
            
            if search_line.contains(term) {
                // Find the actual case in the original line
                if let Some(start) = search_line.find(term) {
                    return line.chars()
                        .skip(start)
                        .take(term.len())
                        .collect();
                }
            }
        }
        
        query_terms.first().unwrap_or(&String::new()).clone()
    }
    
    fn tokenize(&self, text: &str) -> Vec<String> {
        // Always use lowercase for consistency with BM25
        let processed_text = text.to_lowercase();
        
        // Simple tokenization - split on whitespace and common punctuation
        processed_text
            .split_whitespace()
            .flat_map(|word| {
                word.split(&[',', '.', ';', ':', '!', '?', '(', ')', '[', ']', '{', '}', '"', '\''])
            })
            .filter(|token| !token.is_empty() && token.len() > 1)
            .map(|token| token.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let engine = SearchEngine::new(vec![]);
        let tokens = engine.tokenize("Hello, world! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"this".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }
}