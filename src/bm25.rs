use crate::Document;
use std::collections::HashMap;

/// BM25 (Best Matching 25) ranking algorithm implementation
/// Used for ranking text documents based on search query relevance
pub struct BM25 {
    /// Term frequencies for each document
    term_frequencies: Vec<HashMap<String, usize>>,
    /// Document frequencies (how many documents contain each term)
    document_frequencies: HashMap<String, usize>,
    /// Average document length
    avg_doc_length: f64,
    /// Document lengths
    doc_lengths: Vec<usize>,
    /// Total number of documents
    num_documents: usize,
    /// BM25 parameters
    k1: f64,
    b: f64,
}

impl BM25 {
    /// Create a new BM25 index from documents
    pub fn new(documents: &[Document]) -> Self {
        let mut term_frequencies = Vec::new();
        let mut document_frequencies = HashMap::new();
        let mut doc_lengths = Vec::new();
        let mut total_length = 0;
        
        for document in documents {
            let terms = Self::tokenize(&document.content);
            let term_counts = Self::count_terms(&terms);
            
            doc_lengths.push(terms.len());
            total_length += terms.len();
            
            // Update document frequencies
            for term in term_counts.keys() {
                *document_frequencies.entry(term.clone()).or_insert(0) += 1;
            }
            
            term_frequencies.push(term_counts);
        }
        
        let avg_doc_length = if documents.is_empty() {
            0.0
        } else {
            total_length as f64 / documents.len() as f64
        };
        
        Self {
            term_frequencies,
            document_frequencies,
            avg_doc_length,
            doc_lengths,
            num_documents: documents.len(),
            k1: 1.5, // Term frequency saturation parameter
            b: 0.75, // Length normalization parameter
        }
    }
    
    /// Score a document against a query using BM25
    pub fn score(&self, query_terms: &[String], doc_index: usize) -> f64 {
        if doc_index >= self.term_frequencies.len() {
            return 0.0;
        }
        
        let doc_tf = &self.term_frequencies[doc_index];
        let doc_length = self.doc_lengths[doc_index] as f64;
        
        let mut score = 0.0;
        
        for term in query_terms {
            if let Some(&tf) = doc_tf.get(term) {
                let tf = tf as f64;
                let df = self.document_frequencies.get(term).unwrap_or(&0);
                
                // IDF calculation: log(N / df) with smoothing
                let idf = if *df > 0 {
                    ((self.num_documents as f64) / (*df as f64)).ln().max(0.01)
                } else {
                    (self.num_documents as f64).ln()
                };
                
                // BM25 term score
                let term_score = idf * (tf * (self.k1 + 1.0)) / 
                    (tf + self.k1 * (1.0 - self.b + self.b * (doc_length / self.avg_doc_length)));
                
                score += term_score;
            }
        }
        
        score
    }
    
    /// Score a line of text against a query (for line-level matching)
    pub fn score_line(&self, line_terms: &[String], query_terms: &[String], _doc_index: usize) -> f64 {
        let line_tf = Self::count_terms(line_terms);
        let line_length = line_terms.len() as f64;
        
        if line_length == 0.0 {
            return 0.0;
        }
        
        let mut score = 0.0;
        
        for term in query_terms {
            if let Some(&tf) = line_tf.get(term) {
                let tf = tf as f64;
                let df = self.document_frequencies.get(term).unwrap_or(&0);
                
                // IDF calculation with smoothing
                let idf = if *df > 0 {
                    ((self.num_documents as f64) / (*df as f64)).ln().max(0.01)
                } else {
                    // If term not in corpus, give it a small positive IDF
                    (self.num_documents as f64).ln()
                };
                
                // Modified BM25 for line scoring
                let term_score = idf * (tf * (self.k1 + 1.0)) / 
                    (tf + self.k1 * (1.0 - self.b + self.b * (line_length / self.avg_doc_length.max(1.0))));
                
                score += term_score;
            }
        }
        
        // Boost score for lines with multiple query term matches
        let matches = query_terms.iter()
            .filter(|&term| line_tf.contains_key(term))
            .count() as f64;
        let query_coverage = matches / query_terms.len() as f64;
        
        score * (1.0 + query_coverage * 0.5)
    }
    
    /// Tokenize text into terms
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .flat_map(|word| {
                word.split(&[',', '.', ';', ':', '!', '?', '(', ')', '[', ']', '{', '}', '"', '\'', '-', '_'])
            })
            .filter(|token| !token.is_empty() && token.len() > 1)
            .map(|token| token.to_string())
            .collect()
    }
    
    /// Count term frequencies in a list of terms
    fn count_terms(terms: &[String]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for term in terms {
            *counts.entry(term.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_documents() -> Vec<Document> {
        vec![
            Document::new(
                PathBuf::from("doc1.txt"),
                "The quick brown fox jumps over the lazy dog".to_string()
            ),
            Document::new(
                PathBuf::from("doc2.txt"), 
                "A quick brown fox is very fast".to_string()
            ),
            Document::new(
                PathBuf::from("doc3.txt"),
                "The lazy dog sleeps all day".to_string()
            ),
        ]
    }
    
    #[test]
    fn test_bm25_scoring() {
        let documents = create_test_documents();
        let bm25 = BM25::new(&documents);
        
        let query = vec!["quick".to_string(), "fox".to_string()];
        
        // Score each document
        let score1 = bm25.score(&query, 0);
        let score2 = bm25.score(&query, 1);
        let score3 = bm25.score(&query, 2);
        
        // Documents 1 and 2 contain both "quick" and "fox", document 3 contains neither
        assert!(score1 > 0.0, "Document 1 should score > 0, got {}", score1);
        assert!(score2 > 0.0, "Document 2 should score > 0, got {}", score2);
        assert!(score3 == 0.0, "Document 3 should score 0, got {}", score3);
        
        // Documents with query terms should score better than documents without
        assert!(score1 > score3);
        assert!(score2 > score3);
    }
    
    #[test]
    fn test_tokenization() {
        let terms = BM25::tokenize("Hello, world! This is a test.");
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"this".to_string()));
        assert!(terms.contains(&"test".to_string()));
        assert!(!terms.contains(&"a".to_string())); // Single letter filtered out
    }
    
    #[test]
    fn test_term_counting() {
        let terms = vec!["hello".to_string(), "world".to_string(), "hello".to_string()];
        let counts = BM25::count_terms(&terms);
        
        assert_eq!(counts.get("hello"), Some(&2));
        assert_eq!(counts.get("world"), Some(&1));
    }
}