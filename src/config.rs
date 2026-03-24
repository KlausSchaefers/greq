use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_extensions: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub max_file_size_mb: u64,
    pub case_sensitive: bool,
    pub context_lines: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_extensions: vec![
                "txt".to_string(),
                "md".to_string(), 
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "toml".to_string(),
            ],
            ignore_patterns: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
            ],
            max_file_size_mb: 10,
            case_sensitive: false,
            context_lines: 2,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the config file path (typically ~/.greq/config.json)
    pub fn default_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Could not find home directory"))?;
        
        let config_dir = PathBuf::from(home).join(".greq");
        std::fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("config.json"))
    }
    
    /// Check if a file extension should be included in search
    pub fn should_include_extension(&self, extension: &str) -> bool {
        self.default_extensions.iter().any(|ext| ext == extension)
    }
    
    /// Check if a path should be ignored
    pub fn should_ignore_path(&self, path: &str) -> bool {
        self.ignore_patterns.iter().any(|pattern| path.contains(pattern))
    }
}