use anyhow::Result;
use crate::{Document, config::Config};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;

pub struct FileWalker {
    root_path: PathBuf,
    extensions: Option<Vec<String>>,
    config: Config,
}

impl FileWalker {
    pub fn new(root_path: PathBuf, extensions: Option<Vec<String>>) -> Self {
        let config = Config::default();
        Self {
            root_path,
            extensions,
            config,
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
        
        if self.root_path.is_file() {
            // If root_path is a single file, just return it
            files.push(self.root_path.clone());
            return Ok(files);
        }
        
        for entry in WalkDir::new(&self.root_path)
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
        
        // Check extension
        if let Some(ref allowed_extensions) = self.extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                return allowed_extensions.iter().any(|allowed| allowed.to_lowercase() == ext_str);
            }
            return false;
        } else {
            // Use config defaults
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                return self.config.should_include_extension(&ext_str);
            }
        }
        
        // Include files without extensions if no specific extensions were requested
        self.extensions.is_none()
    }
    
    fn read_file_as_document(&self, path: &PathBuf) -> Result<Document> {
        let content = std::fs::read_to_string(path)?;
        Ok(Document::new(path.clone(), content))
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