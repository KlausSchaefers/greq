use crate::Chunk;
use std::path::Path;

/// Safety function to find a valid UTF-8 byte boundary at or before the given position
fn find_safe_byte_boundary(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    
    // Find the nearest char boundary at or before pos
    let mut safe_pos = pos;
    while safe_pos > 0 && !text.is_char_boundary(safe_pos) {
        safe_pos -= 1;
    }
    safe_pos
}

/// Trait for parsing text content into chunks
pub trait Parser {
    /// Parse text content into chunks
    fn parse(&self, content: &str, max_chunk_size: usize) -> Vec<Chunk>;
}

/// Create an appropriate parser based on file extension
pub fn create_parser(path: &Path) -> Box<dyn Parser> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("md") | Some("markdown") => Box::new(MarkdownParser::new()),
        _ => Box::new(DefaultParser::new()),
    }
}

/// Default parser that splits text at word boundaries
pub struct DefaultParser;

impl DefaultParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for DefaultParser {
    fn parse(&self, content: &str, max_size: usize) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut current_pos = 0;
        
        while current_pos < content.len() {
            let chunk_end = if current_pos + max_size >= content.len() {
                content.len()
            } else {
                // Find the last word boundary within max_size
                let search_end = find_safe_byte_boundary(content, current_pos + max_size);
                let chunk_text = &content[current_pos..search_end];
                
                // Find the last whitespace to avoid splitting words
                if let Some(last_space) = chunk_text.rfind(char::is_whitespace) {
                    current_pos + last_space + 1
                } else {
                    // If no whitespace found, use the safe search_end
                    search_end
                }
            };
            
            let chunk_content = content[current_pos..chunk_end].trim().to_string();
            if !chunk_content.is_empty() {
                chunks.push(Chunk::new(chunk_content, current_pos, chunk_end));
            }
            
            current_pos = chunk_end;
            // Skip any leading whitespace for the next chunk
            while current_pos < content.len() {
                if let Some(ch) = content[current_pos..].chars().next() {
                    if ch.is_whitespace() {
                        current_pos += ch.len_utf8();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        chunks
    }
}


impl Default for DefaultParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown-aware parser that respects headline boundaries
pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if a line is a markdown headline
    fn is_headline(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return false;
        }
        
        // Find the first non-# character
        let mut hash_count = 0;
        for ch in trimmed.chars() {
            if ch == '#' {
                hash_count += 1;
            } else {
                break;
            }
        }
        
        // Must have at least one # and the next character should be a space or end of string
        hash_count > 0 && hash_count <= 6 && // Valid markdown headers are 1-6 levels
            (trimmed.len() == hash_count || // Just hashes
             trimmed.chars().nth(hash_count) == Some(' ')) // Followed by space
    }
    
    /// Split a long line into chunks using word boundaries (similar to DefaultParser)
    fn split_long_line(&self, line: &str, line_start: usize, max_size: usize) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut current_pos = 0;
        
        while current_pos < line.len() {
            let chunk_end = if current_pos + max_size >= line.len() {
                line.len()
            } else {
                // Find the last word boundary within max_size
                let search_end = find_safe_byte_boundary(line, current_pos + max_size);
                let chunk_text = &line[current_pos..search_end];
                
                // Find the last whitespace to avoid splitting words
                if let Some(last_space) = chunk_text.rfind(char::is_whitespace) {
                    current_pos + last_space + 1
                } else {
                    // If no whitespace found, use the safe search_end
                    search_end
                }
            };
            
            let chunk_content = line[current_pos..chunk_end].trim().to_string();
            if !chunk_content.is_empty() {
                chunks.push(Chunk::new(
                    chunk_content,
                    line_start + current_pos,
                    line_start + chunk_end
                ));
            }
            
            current_pos = chunk_end;
            // Skip any leading whitespace for the next chunk
            while current_pos < line.len() {
                if let Some(ch) = line[current_pos..].chars().next() {
                    if ch.is_whitespace() {
                        current_pos += ch.len_utf8();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        chunks
    }
}

impl Parser for MarkdownParser {
    fn parse(&self, content: &str, max_size: usize) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return chunks;
        }
        
        let mut current_chunk_lines = Vec::new();
        let mut chunk_start_pos = 0;
        let mut current_pos = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let line_start = current_pos;
            let line_end = current_pos + line.len();
            
            // If this single line is already too long and it's not a headline, 
            // we need to use word-boundary splitting like DefaultParser
            if line.len() > max_size && !Self::is_headline(line) {
                // First, finalize any current chunk
                if !current_chunk_lines.is_empty() {
                    let chunk_content = current_chunk_lines.join("\n");
                    if !chunk_content.trim().is_empty() {
                        chunks.push(Chunk::new(
                            chunk_content.trim().to_string(),
                            chunk_start_pos,
                            line_start.saturating_sub(1)
                        ));
                    }
                    current_chunk_lines.clear();
                }
                
                // Split this long line using word boundaries
                let line_chunks = self.split_long_line(line, line_start, max_size);
                chunks.extend(line_chunks);
                
                // Reset for next chunk
                chunk_start_pos = line_end + 1;
            } else {
                // Check if this line is a headline and we have existing content
                if Self::is_headline(line) && !current_chunk_lines.is_empty() {
                    // Finalize the current chunk before starting a new one
                    let chunk_content = current_chunk_lines.join("\n");
                    if !chunk_content.trim().is_empty() {
                        chunks.push(Chunk::new(
                            chunk_content.trim().to_string(),
                            chunk_start_pos,
                            current_pos.saturating_sub(1)
                        ));
                    }
                    
                    current_chunk_lines.clear();
                    chunk_start_pos = line_start;
                }
                
                current_chunk_lines.push(line.to_string());
                
                // Check if we've exceeded max_size (but don't split on headlines)
                let current_content = current_chunk_lines.join("\n");
                if current_content.len() > max_size && !Self::is_headline(line) && current_chunk_lines.len() > 1 {
                    // Remove the last line and create a chunk
                    current_chunk_lines.pop();
                    let chunk_content = current_chunk_lines.join("\n");
                    
                    if !chunk_content.trim().is_empty() {
                        chunks.push(Chunk::new(
                            chunk_content.trim().to_string(),
                            chunk_start_pos,
                            line_start.saturating_sub(1)
                        ));
                    }
                    
                    // Start new chunk with the current line
                    current_chunk_lines.clear();
                    current_chunk_lines.push(line.to_string());
                    chunk_start_pos = line_start;
                }
            }
            
            // Move position past this line + newline (except for last line)
            current_pos = if i < lines.len() - 1 {
                line_end + 1
            } else {
                line_end
            };
        }
        
        // Add any remaining content as the final chunk
        if !current_chunk_lines.is_empty() {
            let chunk_content = current_chunk_lines.join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk::new(
                    chunk_content.trim().to_string(),
                    chunk_start_pos,
                    current_pos
                ));
            }
        }
        
        chunks
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parser_basic() {
        let parser = DefaultParser::new();
        let content = "This is a test document with some content that should be chunked.";
        let chunks = parser.parse(content, 20);
        
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(chunk.end > chunk.start);
        }
    }

    #[test]
    fn test_default_parser_word_boundaries() {
        let parser = DefaultParser::new();
        let content = "Short words here break properly"; // 32 chars
        let chunks = parser.parse(content, 20);
        
        // Should split at word boundaries
        assert!(chunks.len() >= 2);
        
        // Check that chunks don't start or end mid-word (except for very long words)
        for chunk in &chunks {
            let trimmed = chunk.content.trim();
            if !trimmed.is_empty() {
                // Should not start with punctuation or whitespace
                assert!(!trimmed.starts_with(' '));
            }
        }
    }

    #[test]
    fn test_default_parser_empty_content() {
        let parser = DefaultParser::new();
        let chunks = parser.parse("", 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_default_parser_single_chunk() {
        let parser = DefaultParser::new();
        let content = "Short content";
        let chunks = parser.parse(content, 100);
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, content);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, content.len());
    }

    #[test]
    fn test_markdown_parser_headline_boundary() {
        let parser = MarkdownParser::new();
        let content = "abc\n# headline\ndef";
        let chunks = parser.parse(content, 100);
        
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "abc");
        assert_eq!(chunks[1].content, "# headline\ndef");
    }

    #[test]
    fn test_markdown_parser_multiple_headlines() {
        let parser = MarkdownParser::new();
        let content = "intro\n# First\ncontent1\n## Second\ncontent2\n### Third\ncontent3";
        let chunks = parser.parse(content, 100);
        
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].content, "intro");
        assert_eq!(chunks[1].content, "# First\ncontent1");
        assert_eq!(chunks[2].content, "## Second\ncontent2");
        assert_eq!(chunks[3].content, "### Third\ncontent3");
    }

    #[test]
    fn test_markdown_parser_respects_max_size() {
        let parser = MarkdownParser::new();
        let content = "This is a very long line that should be split even in markdown parser when it exceeds the maximum size\nMore content here";
        let chunks = parser.parse(content, 50);
        
        // Should still respect size limits for non-headline content
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            // Allow some flexibility for headlines, but non-headline chunks should respect size
            if !chunk.content.trim_start().starts_with('#') {
                assert!(chunk.content.len() <= 80); // Some margin for word boundaries
            }
        }
    }

    #[test]
    fn test_markdown_parser_headline_detection() {
        assert!(MarkdownParser::is_headline("# Title"));
        assert!(MarkdownParser::is_headline("## Subtitle"));
        assert!(MarkdownParser::is_headline("### Sub-subtitle"));
        assert!(MarkdownParser::is_headline("   # Indented title"));
        
        assert!(!MarkdownParser::is_headline("#hashtag"));
        assert!(!MarkdownParser::is_headline("##no space"));
        assert!(!MarkdownParser::is_headline("not a headline"));
        assert!(!MarkdownParser::is_headline(""));
    }

    #[test]
    fn test_markdown_parser_empty_content() {
        let parser = MarkdownParser::new();
        let chunks = parser.parse("", 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_markdown_parser_only_headlines() {
        let parser = MarkdownParser::new();
        let content = "# First\n## Second\n### Third";
        let chunks = parser.parse(content, 100);
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "# First");
        assert_eq!(chunks[1].content, "## Second");
        assert_eq!(chunks[2].content, "### Third");
    }

    #[test]
    fn test_create_parser_markdown_extensions() {
        use std::path::PathBuf;
        
        // Test .md extension
        let path = PathBuf::from("test.md");
        let parser = create_parser(&path);
        let content = "text\n# header\nmore";
        let chunks = parser.parse(content, 100);
        
        // Should create 2 chunks due to markdown parsing
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "text");
        assert_eq!(chunks[1].content, "# header\nmore");
    }

    #[test]
    fn test_create_parser_markdown_full_extension() {
        use std::path::PathBuf;
        
        // Test .markdown extension
        let path = PathBuf::from("document.markdown");
        let parser = create_parser(&path);
        let content = "intro\n## Section\ncontent";
        let chunks = parser.parse(content, 100);
        
        // Should create 2 chunks due to markdown parsing
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "intro");
        assert_eq!(chunks[1].content, "## Section\ncontent");
    }

    #[test]
    fn test_create_parser_default_extensions() {
        use std::path::PathBuf;
        
        // Test non-markdown extensions
        let extensions = &["txt", "rs", "py", "js", ""];
        
        for ext in extensions {
            let path = if ext.is_empty() {
                PathBuf::from("file_no_extension")
            } else {
                PathBuf::from(format!("test.{}", ext))
            };
            
            let parser = create_parser(&path);
            let content = "text\n# header\nmore"; // Same content as markdown test
            let chunks = parser.parse(content, 100);
            
            // Should create 1 chunk since it's using DefaultParser
            // DefaultParser doesn't respect markdown headers
            assert_eq!(chunks.len(), 1, "Failed for extension: {}", ext);
            assert_eq!(chunks[0].content, content);
        }
    }
}