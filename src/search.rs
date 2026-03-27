use crate::{Document, bm25::BM25, TokenizerTrait, embeddings::Embeddings};
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
    embeddings: Option<Embeddings>,
    tokenizer: Box<dyn TokenizerTrait>,
    embedding_weight: f64,
}

impl SearchEngine {
    pub fn new(documents: Vec<Document>, tokenizer: Box<dyn TokenizerTrait>, embedding_weight: f64) -> Self {
        let bm25 = BM25::new(&documents, tokenizer.clone_tokenizer());
        if embedding_weight > 0.0 {
            let embeddings: Embeddings = Embeddings::new(&documents);
            Self {
                documents,
                bm25,
                embeddings: Some(embeddings),
                tokenizer,         
                embedding_weight,
            }
        } else  {
            Self {
                documents,
                bm25,
                embeddings: None,
                tokenizer,         
                embedding_weight,
            }
        }
    }
    
    /// Search for a query and return ranked chunk results with context
    pub fn search(&self, query: &str, n: usize, context: usize) -> Vec<SearchResult> {
        // Score each chunk using BM25
        let bm25_chunk_results = self.bm25.search(&self.documents, query, 0.001);
        let mut combined_results = Vec::new();
        
        if self.embedding_weight > 0.0 {
            if let Some(embeddings) = &self.embeddings {
                let embedding_chunk_results = embeddings.search(&self.documents, query, 0.001);
                print!("DEBUG: BM25 found {} chunks, Embeddings found {} chunks", bm25_chunk_results.len(), embedding_chunk_results.len());
                
                // Combine BM25 and embedding scores using efficient HashMap lookup
                for ((doc_idx, chunk_idx), embedding_score) in embedding_chunk_results {
                    let bm25_score = bm25_chunk_results.get(&(doc_idx, chunk_idx))
                        .copied()
                        .unwrap_or(0.0);

                    //println!("DEBUG: Doc {} Chunk {} BM25 score: {:.4}, Embedding score: {:.4}", doc_idx, chunk_idx, bm25_score, embedding_score);
                    
                    let combined_score = (1.0 - self.embedding_weight) * bm25_score + self.embedding_weight * embedding_score;
                    combined_results.push((doc_idx, chunk_idx, combined_score));
                }
            }
        } else {
            // Convert HashMap to Vec when no embeddings are used
            combined_results = bm25_chunk_results.into_iter()
                .map(|((doc_idx, chunk_idx), score)| (doc_idx, chunk_idx, score))
                .collect();
        }
        
        // Sort by score (highest first)
        combined_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N and expand with context
        let top_chunks: Vec<_> = combined_results.into_iter().take(n).collect();
        
        let mut results = Vec::new();
        for (doc_idx, chunk_idx, score) in top_chunks {
            let document = &self.documents[doc_idx];
            
            // Get expanded chunks with context
            let expanded_chunks = document.expand_chunks(&[chunk_idx], context);
            
            for expanded_chunk in expanded_chunks {
                let query_terms = self.tokenizer.tokenize(&query.to_lowercase());
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
    //use crate::{Tokenizer, TokenizerTrait};

    #[test]
    fn test_search() {
        // let tokenizer = Tokenizer::new();
        // let tokens = tokenizer.tokenize("Hello, world! This is a test.");
        // assert!(tokens.contains(&"hello".to_string()));
        // assert!(tokens.contains(&"world".to_string()));
        // assert!(tokens.contains(&"this".to_string()));
        // assert!(tokens.contains(&"test".to_string()));
    }
}