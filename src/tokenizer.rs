use std::collections::HashMap;

/// Text tokenizer for search operations
/// Provides consistent tokenization across the search engine components
#[derive(Clone)]
pub struct Tokenizer {
    /// Characters to split on when tokenizing
    split_chars: Vec<char>,
    /// Minimum token length to include
    min_token_length: usize,
}

impl Tokenizer {
    /// Create a new tokenizer with default settings
    pub fn new() -> Self {
        Self {
            split_chars: vec![
                ',', '.', ';', ':', '!', '?', '(', ')', '[', ']', 
                '{', '}', '"', '\'', '-', '_'
            ],
            min_token_length: 2,
        }
    }
    
    /// Create a new tokenizer with custom split characters
    pub fn with_split_chars(split_chars: Vec<char>) -> Self {
        Self {
            split_chars,
            min_token_length: 2,
        }
    }
    
    /// Create a new tokenizer with custom minimum token length
    pub fn with_min_token_length(min_token_length: usize) -> Self {
        Self {
            split_chars: vec![
                ',', '.', ';', ':', '!', '?', '(', ')', '[', ']', 
                '{', '}', '"', '\'', '-', '_'
            ],
            min_token_length,
        }
    }
    
    /// Tokenize text into terms
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .flat_map(|word| {
                word.split(&self.split_chars[..])
            })
            .filter(|token| !token.is_empty() && token.len() >= self.min_token_length)
            .map(|token| token.to_string())
            .collect()
    }
    
    /// Count term frequencies in a list of terms
    pub fn count_terms(&self, terms: &[String]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for term in terms {
            *counts.entry(term.clone()).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    
    #[test]
    fn test_hyphen_and_underscore_splitting() {
        let tokenizer = Tokenizer::new();
        let terms = tokenizer.tokenize("hello-world_test");
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"test".to_string()));
    }
    
    #[test]
    fn test_custom_split_chars() {
        let tokenizer = Tokenizer::with_split_chars(vec![',', '.']);
        let terms = tokenizer.tokenize("hello,world.test-this");
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"test-this".to_string())); // hyphen not split since not in custom chars
    }
    
    #[test]
    fn test_min_token_length() {
        let tokenizer = Tokenizer::with_min_token_length(3);
        let terms = tokenizer.tokenize("a bb ccc dddd");
        assert!(!terms.contains(&"a".to_string()));
        assert!(!terms.contains(&"bb".to_string()));
        assert!(terms.contains(&"ccc".to_string()));
        assert!(terms.contains(&"dddd".to_string()));
    }
}