pub mod search;
pub mod file_walker;
pub mod bm25;
pub mod config;

use std::path::PathBuf;

/// Document represents a searchable text document
#[derive(Debug, Clone)]
pub struct Document {
    pub file_path: PathBuf,
    pub content: String,
    pub lines: Vec<String>,
}

impl Document {
    pub fn new(file_path: PathBuf, content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            file_path,
            content,
            lines,
        }
    }
    
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
    
    pub fn get_line(&self, line_number: usize) -> Option<&String> {
        if line_number > 0 {
            self.lines.get(line_number - 1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_document_creation() {
        let content = "line 1\nline 2\nline 3".to_string();
        let doc = Document::new(PathBuf::from("test.txt"), content);
        
        assert_eq!(doc.line_count(), 3);
        assert_eq!(doc.get_line(1), Some(&"line 1".to_string()));
        assert_eq!(doc.get_line(2), Some(&"line 2".to_string()));
        assert_eq!(doc.get_line(3), Some(&"line 3".to_string()));
        assert_eq!(doc.get_line(0), None);
        assert_eq!(doc.get_line(4), None);
    }
}