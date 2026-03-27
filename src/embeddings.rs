/// Placeholder for future embeddings functionality
use crate::{Document};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::cell::RefCell;

pub struct Embeddings {
    /// Term frequencies for each chunk across all documents
    chunk_embeddings: Vec<Vec<f32>>,
    model: Option<RefCell<TextEmbedding>>,
}

impl Embeddings {
    pub fn new(documents: &[Document]) -> Self {
        // Handle model creation with error handling (like try-catch)
        let mut model = match TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        ) {
            Ok(model) => model,
            Err(e) => {
                eprintln!("Failed to create embedding model: {}", e);
                return Self {
                    chunk_embeddings: Vec::new(),
                    model: None,
                };
            }
        };

        // Extract text content from the actual documents for embedding
        let mut text_chunks = Vec::new();
        for (doc_idx, document) in documents.iter().enumerate() {
            for (chunk_idx, chunk) in document.chunks.iter().enumerate() {
                println!("Document {} Chunk {}: {}", doc_idx, chunk_idx, chunk.content);
                text_chunks.push(format!("passage: {}", chunk.content));
            }
        }

        // Generate embeddings with error handling (like try-catch)
        let chunk_embeddings = match model.embed(text_chunks, None) {
            Ok(embeddings) => {
                println!("Successfully generated {} embeddings", embeddings.len());
                embeddings // Return embeddings from the Ok branch
            }
            Err(e) => {
                eprintln!("Failed to generate embeddings: {}", e);
                Vec::new() // Return empty vec on error
            }
        };
        
                
        Self {
            chunk_embeddings,
            model: Some(RefCell::new(model)),
        }
    }

    /// Get the generated embeddings
    pub fn get_embeddings(&self) -> &Vec<Vec<f32>> {
        &self.chunk_embeddings
    }

    /// Get the number of embeddings
    pub fn len(&self) -> usize {
        self.chunk_embeddings.len()
    }

    /// Check if embeddings are empty
    pub fn is_empty(&self) -> bool {
        self.chunk_embeddings.is_empty()
    }

    /// Get a reference to the model if available
    pub fn get_model(&self) -> bool {
        self.model.is_some()
    }

    /// Check if model is available
    pub fn has_model(&self) -> bool {
        self.model.is_some()
    }
    
    /// Compute cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        (dot_product / (norm_a * norm_b)) as f64
    }
    
    /// Search for chunks matching the query using cosine similarity
    pub fn search(
        &self, 
        documents: &[Document], 
        query: &str, 
        min_score: f64
    ) -> Vec<(usize, usize, f64)> {
        // Check if model is available
        let model_cell = match &self.model {
            Some(model_cell) => model_cell,
            None => {
                eprintln!("No embedding model available for search");
                return Vec::new();
            }
        };
        
        // Generate query embedding
        let query_text = format!("query: {}", query);
        let query_embedding = match model_cell.borrow_mut().embed(vec![query_text], None) {
            Ok(mut embeddings) => {
                if embeddings.is_empty() {
                    eprintln!("Failed to generate query embedding");
                    return Vec::new();
                }
                embeddings.remove(0)
            }
            Err(e) => {
                eprintln!("Error generating query embedding: {}", e);
                return Vec::new();
            }
        };
        
        let mut results = Vec::new();
        let mut chunk_idx = 0;
        
        // Iterate through documents and chunks to compute similarities
        for (doc_idx, document) in documents.iter().enumerate() {
            for (doc_chunk_idx, _chunk) in document.chunks.iter().enumerate() {
                if chunk_idx < self.chunk_embeddings.len() {
                    let chunk_embedding = &self.chunk_embeddings[chunk_idx];
                    let similarity = self.cosine_similarity(&query_embedding, chunk_embedding);
                    
                    if similarity >= min_score {
                        results.push((doc_idx, doc_chunk_idx, similarity));
                    }
                    
                    chunk_idx += 1;
                } else {
                    eprintln!("Warning: chunk index {} exceeds embedding count {}", chunk_idx, self.chunk_embeddings.len());
                    break;
                }
            }
        }
        
        results
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
    fn test_embeddings() {
        let documents = create_test_documents();
        println!("Documents: {:?}", documents);

        let embeddings = Embeddings::new(&documents);
        
        // Test the new optional model functionality
        println!("Has model: {}", embeddings.has_model());
        if embeddings.get_model() {
            println!("Model is available for further embedding generation");
        } else {
            println!("No model available - this instance only contains cached embeddings");
        }
        
        // Test embeddings access
        println!("Generated {} embeddings", embeddings.len());
        if !embeddings.is_empty() {
            let sample_embeddings = embeddings.get_embeddings();
            println!("First embedding has {} dimensions", sample_embeddings[0].len());
            println!("Sample values: {:?}", &sample_embeddings[0][0..3]); // First 3 values only
        }
    }
    
    #[test]
    fn test_embeddings_search() {
        let documents = create_test_documents();
        let embeddings = Embeddings::new(&documents);
        
        if embeddings.has_model() {
            // Test search functionality
            let search_results = embeddings.search(&documents, "quick brown fox", 0.1);
            println!("Search results for 'quick brown fox': {:?}", search_results);
            
            // Should find at least one result (documents contain "quick brown fox")
            assert!(!search_results.is_empty(), "Should find matching chunks");
            
            // Test with threshold too high
            let no_results = embeddings.search(&documents, "nonexistent query", 0.99);
            assert!(no_results.is_empty() || no_results.len() < search_results.len(), 
                   "High threshold should return fewer results");
        }
    }
}