use crate::{Document, TokenizerTrait};
use std::collections::HashMap;

/// BM25 (Best Matching 25) ranking algorithm implementation
/// Used for ranking text chunks based on search query relevance
pub struct BM25 {
    /// Term frequencies for each chunk across all documents
    chunk_term_frequencies: Vec<HashMap<String, usize>>,
    /// Document frequencies (how many chunks contain each term)
    term_frequencies: HashMap<String, usize>,
    /// Average chunk length
    avg_chunk_length: f64,
    /// Chunk lengths
    chunk_lengths: Vec<usize>,
    /// Total number of chunks
    num_chunks: usize,
    /// Mapping from chunk index to (doc_index, chunk_index_in_doc)
    chunk_mapping: Vec<(usize, usize)>,
    /// Tokenizer used for text processing
    tokenizer: Box<dyn TokenizerTrait>,
    /// BM25 parameters
    k1: f64,
    b: f64,
}

impl BM25 {
    /// Create a new BM25 index from document chunks
    pub fn new(documents: &[Document], tokenizer: Box<dyn TokenizerTrait>) -> Self {
        let mut chunk_term_frequencies = Vec::new();
        let mut term_frequencies = HashMap::new();
        let mut chunk_lengths = Vec::new();
        let mut chunk_mapping = Vec::new();
        let mut total_length = 0;
        
        for (doc_idx, document) in documents.iter().enumerate() {
            for (chunk_idx, chunk) in document.chunks.iter().enumerate() {
                let terms = tokenizer.tokenize(&chunk.content);
                let term_counts = tokenizer.count_terms(&terms);
                
                
                chunk_lengths.push(terms.len());
                total_length += terms.len();
                chunk_mapping.push((doc_idx, chunk_idx));
                
                // Update term frequencies (how many chunks contain each term)
                for term in term_counts.keys() {
                    *term_frequencies.entry(term.clone()).or_insert(0) += 1;
                   // println!("Term '{}' : {}", term, term_counts[term]);
                }
                
                chunk_term_frequencies.push(term_counts);
            }
        }
        
        let avg_chunk_length = if chunk_term_frequencies.is_empty() {
            0.0
        } else {
            total_length as f64 / chunk_term_frequencies.len() as f64
        };
        
        let num_chunks = chunk_term_frequencies.len();
        
        Self {
            chunk_term_frequencies,
            term_frequencies,
            avg_chunk_length,
            chunk_lengths,
            num_chunks,
            chunk_mapping,
            tokenizer,
            k1: 1.5, // Term frequency saturation parameter
            b: 0.75, // Length normalization parameter
        }
    }
    
        /// Search for chunks matching the query using BM25 scoring
    pub fn search(
        &self, 
        documents: &[Document], 
        query: &str, 
        min_score: f64
    ) -> Vec<(usize, usize, f64)> {
        // Tokenize the query
        let query_terms = self.tokenizer.tokenize(&query.to_lowercase());
        if query_terms.is_empty() {
            return Vec::new();
        }
        
        let mut chunk_results = Vec::new();
        
        for (doc_idx, document) in documents.iter().enumerate() {
            for (chunk_idx, chunk) in document.chunks.iter().enumerate() {
                let chunk_terms = self.tokenizer.tokenize(&chunk.content.to_lowercase());
                let score = self.score_chunk(&chunk_terms, &query_terms, doc_idx, chunk_idx);            
                if score > min_score {
                    chunk_results.push((doc_idx, chunk_idx, score));
                }
            }
        }
        chunk_results
    }

    /// Score a chunk against a query using BM25
    fn score_chunk(&self, _chunk_terms: &[String], query_terms: &[String], doc_index: usize, chunk_index: usize) -> f64 {
        // Find the global chunk index
        let global_chunk_idx = self.chunk_mapping.iter()
            .position(|&(d_idx, c_idx)| d_idx == doc_index && c_idx == chunk_index);
            
        let global_chunk_idx = match global_chunk_idx {
            Some(idx) => idx,
            None => return 0.0,
        };
        
        if global_chunk_idx >= self.chunk_term_frequencies.len() {
            return 0.0;
        }

        
        let chunk_tf = &self.chunk_term_frequencies[global_chunk_idx];
        let chunk_length = self.chunk_lengths[global_chunk_idx] as f64;
        
        if chunk_length == 0.0 {
            return 0.0;
        }
        
        let mut score = 0.0;
        
        for term in query_terms {
            if let Some(&tf) = chunk_tf.get(term) {
                let tf = tf as f64;
                let df = self.term_frequencies.get(term).unwrap_or(&0);
                
                // IDF calculation: log(N / df) with smoothing
                let idf = if *df > 0 {
                    ((self.num_chunks as f64) / (*df as f64)).ln().max(0.01)
                } else {
                    (self.num_chunks as f64).ln()
                };
                
                // BM25 term score
                let term_score = idf * (tf * (self.k1 + 1.0)) / 
                    (tf + self.k1 * (1.0 - self.b + self.b * (chunk_length / self.avg_chunk_length)));
                
                score += term_score;
            }
        }
        
        // Boost score for chunks with multiple query term matches
        let matches = query_terms.iter()
            .filter(|&term| chunk_tf.contains_key(term))
            .count();
        
        if matches > 1 {
            score *= 1.0 + (matches as f64 - 1.0) * 0.2; // 20% boost per additional term
        }
        
        score
    }
    

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tokenizer;
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
    fn test_bm25_chunk_scoring() {
        let documents = create_test_documents();
        let tokenizer = Box::new(Tokenizer::new()) as Box<dyn TokenizerTrait>;
        let bm25 = BM25::new(&documents, tokenizer);
        
        let query = vec!["quick".to_string(), "fox".to_string()];
        
        // Test scoring first chunk of first document (should contain "quick" and "fox")
        if let Some(first_chunk) = documents[0].get_chunk(0) {
            let tokenizer = Tokenizer::new();
            let chunk_terms = tokenizer.tokenize(&first_chunk.content);
            let score = bm25.score_chunk(&chunk_terms, &query, 0, 0);
            assert!(score > 0.0, "First chunk should score > 0, got {}", score);
        }
    }
    
    #[test]
    fn test_tokenization() {
        let tokenizer = Tokenizer::new();
        let terms = tokenizer.tokenize("Hello, world! This is a test.");
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"this".to_string()));
        assert!(terms.contains(&"test".to_string()));
        assert!(!terms.contains(&"a".to_string())); // Single letter filtered out
    }
    
    #[test]
    fn test_term_counting() {
        let tokenizer = Tokenizer::new();
        let terms = vec!["hello".to_string(), "world".to_string(), "hello".to_string()];
        let counts = tokenizer.count_terms(&terms);
        
        assert_eq!(counts.get("hello"), Some(&2));
        assert_eq!(counts.get("world"), Some(&1));
    }
}