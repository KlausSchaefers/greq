use anyhow::Result;
use crate::{Document, config::Config};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;
use glob::Pattern;

pub struct FileWalker {
    root_path: PathBuf,
    glob_pattern: Option<Pattern>,
    base_dir: PathBuf,
    extensions: Option<Vec<String>>,
    config: Config,
    chunk_size: usize,
}

impl FileWalker {
    pub fn new(root_path: PathBuf, extensions: Option<Vec<String>>) -> Self {
        Self::new_with_chunk_size(root_path, extensions, 200)
    }
    
    pub fn new_with_chunk_size(root_path: PathBuf, extensions: Option<Vec<String>>, chunk_size: usize) -> Self {
        let config = Config::default();
        let (base_dir, glob_pattern) = Self::parse_glob_pattern(&root_path);
        
        Self {
            root_path: root_path.clone(),
            glob_pattern,
            base_dir,
            extensions,
            config,
            chunk_size,
        }
    }
    
    /// Parse a path that might contain glob patterns
    /// Returns (base_directory, pattern) where base_directory is the directory to walk
    /// and pattern is the glob pattern to match files against
    fn parse_glob_pattern(path: &PathBuf) -> (PathBuf, Option<Pattern>) {
        let path_str = path.to_string_lossy();
        
        // Check if the path contains glob pattern characters
        if path_str.contains('*') || path_str.contains('?') || path_str.contains('[') {
            // Try to find the last directory separator before any glob patterns
            let mut base_path = PathBuf::new();
            let mut pattern_part = String::new();
            let mut found_glob = false;
            
            for component in path.components() {
                let component_str = component.as_os_str().to_string_lossy();
                if !found_glob && !component_str.contains('*') && !component_str.contains('?') && !component_str.contains('[') {
                    base_path.push(component);
                } else {
                    found_glob = true;
                    if !pattern_part.is_empty() {
                        pattern_part.push_str("/");
                    }
                    pattern_part.push_str(&component_str);
                }
            }
            
            // If base_path is empty, use current directory
            if base_path.as_os_str().is_empty() {
                base_path = PathBuf::from(".");
            }
            
            // Use the original path as the pattern
            // The glob crate expects the full pattern as it was provided
            match Pattern::new(&path_str) {
                Ok(pattern) => (base_path, Some(pattern)),
                Err(_) => (path.clone(), None), // Fall back to normal behavior if pattern is invalid
            }
        } else {
            (path.clone(), None)
        }
    }
    
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    
    /// Collect all searchable documents from the file system
    pub fn collect_documents(&self) -> Result<Vec<Document>> {
        let file_paths = self.find_files()?;
        
        // Read files in parallel and convert to documents
        let documents: Result<Vec<Document>, _> = file_paths
            .par_iter()
            .map(|path| self.read_file_as_document(path))
            .collect();
            
        documents
    }
    
    fn find_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if self.root_path.is_file() && self.glob_pattern.is_none() {
            // If root_path is a single file and no glob pattern, just return it
            files.push(self.root_path.clone());
            return Ok(files);
        }
        
        // Use base_dir for walking when we have a glob pattern, otherwise use root_path
        let walk_dir = if self.glob_pattern.is_some() {
            &self.base_dir
        } else {
            &self.root_path
        };
        
        for entry in WalkDir::new(walk_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() && self.should_include_file(path) {
                files.push(path.to_path_buf());
            }
        }
        
        Ok(files)
    }
    
    fn should_include_file(&self, path: &Path) -> bool {
        // Check file size
        if let Ok(metadata) = path.metadata() {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.config.max_file_size_mb {
                return false;
            }
        }
        
        // Check ignore patterns
        let path_str = path.to_string_lossy();
        if self.config.should_ignore_path(&path_str) {
            return false;
        }
        
        // Check glob pattern first (highest priority)
        if let Some(ref pattern) = self.glob_pattern {
            if !pattern.matches_path(path) {
                return false;
            }
        }
        
        // Check extension
        if let Some(ref allowed_extensions) = self.extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                return allowed_extensions.iter().any(|allowed| allowed.to_lowercase() == ext_str);
            }
            return false;
        } else {
            // Use config defaults only if no glob pattern is specified
            if self.glob_pattern.is_none() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    return self.config.should_include_extension(&ext_str);
                }
            }
        }
        
        // Include files without extensions if no specific extensions were requested
        self.extensions.is_none()
    }
    
    fn read_file_as_document(&self, path: &PathBuf) -> Result<Document> {
        let content = std::fs::read_to_string(path)?;
        Ok(Document::new_with_chunk_size(path.clone(), content, self.chunk_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_should_include_file() {
        let walker = FileWalker::new(PathBuf::from("."), Some(vec!["rs".to_string()]));
        
        // Should include .rs files
        assert!(walker.should_include_file(Path::new("test.rs")));
        
        // Should not include other extensions
        assert!(!walker.should_include_file(Path::new("test.txt")));
    }
    
    #[test]
    fn test_parse_glob_pattern() {
        // Test simple glob pattern
        let (base, pattern) = FileWalker::parse_glob_pattern(&PathBuf::from("./data/*.txt"));
        assert_eq!(base, PathBuf::from("./data"));
        assert!(pattern.is_some());
        
        // Test regular path without globs
        let (base, pattern) = FileWalker::parse_glob_pattern(&PathBuf::from("./data/test.txt"));
        assert_eq!(base, PathBuf::from("./data/test.txt"));
        assert!(pattern.is_none());
        
        // Test pattern in middle of path
        let (base, pattern) = FileWalker::parse_glob_pattern(&PathBuf::from("./data/*/files/*.txt"));
        assert_eq!(base, PathBuf::from("./data"));
        assert!(pattern.is_some());
    }
    
    #[test]
    fn test_glob_pattern_matching() {
        // Test with pattern that should match in the right format
        let walker = FileWalker::new(PathBuf::from("*.txt"), None);
        
        // Test the glob pattern exists
        assert!(walker.glob_pattern.is_some());
        
        if let Some(ref pattern) = walker.glob_pattern {
            // Pattern "*.txt" should match "test.txt" and "subdir/test.txt" (glob * is greedy)
            assert!(pattern.matches("test.txt"));
            assert!(!pattern.matches("test.rs"));
            // Note: *.txt actually matches paths with directories too in glob
            assert!(pattern.matches("subdir/test.txt")); 
        }
        
        // Test with directory pattern that's more restrictive
        let walker2 = FileWalker::new(PathBuf::from("data/*.txt"), None);
        if let Some(ref pattern) = walker2.glob_pattern {
            assert!(pattern.matches("data/test.txt"));
            assert!(!pattern.matches("data/test.rs"));
            assert!(!pattern.matches("test.txt")); // Doesn't match root level
        }
    }
    
    #[test] 
    fn test_collect_documents() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello world\nThis is a test")?;
        
        let walker = FileWalker::new(test_file, None);
        let documents = walker.collect_documents()?;
        
        assert_eq!(documents.len(), 1);
        assert!(documents[0].content.contains("Hello world"));
        
        Ok(())
    }
}