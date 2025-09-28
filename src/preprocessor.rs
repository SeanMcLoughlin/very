use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ParseError;

#[derive(Debug, Clone)]
pub struct Preprocessor {
    include_dirs: Vec<PathBuf>,
    defines: HashMap<String, String>,
}

impl Preprocessor {
    pub fn new(include_dirs: Vec<PathBuf>, defines: HashMap<String, String>) -> Self {
        Self {
            include_dirs,
            defines,
        }
    }

    pub fn preprocess_file(&mut self, file_path: &Path) -> Result<String, ParseError> {
        let content = fs::read_to_string(file_path).map_err(|e| ParseError {
            message: format!("Failed to read file {}: {}", file_path.display(), e),
            location: None,
        })?;

        self.preprocess_content(&content, Some(file_path))
    }

    pub fn preprocess_content(
        &mut self,
        content: &str,
        current_file: Option<&Path>,
    ) -> Result<String, ParseError> {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();

            if line.starts_with('`') {
                // Handle preprocessor directives
                if let Some(directive) = line.strip_prefix('`') {
                    if let Some(define_content) = directive.strip_prefix("define ") {
                        self.handle_define(define_content)?;
                        continue; // Don't add the define line to output
                    } else if let Some(include_content) = directive.strip_prefix("include ") {
                        let included_content =
                            self.handle_include(include_content, current_file, line_num + 1)?;
                        result.push_str(&included_content);
                        result.push('\n');
                        continue;
                    } else if directive.starts_with("ifdef ")
                        || directive.starts_with("ifndef ")
                        || directive == "else"
                        || directive == "endif"
                    {
                        // For now, just ignore conditional compilation directives
                        // TODO: Implement proper conditional compilation
                        continue;
                    }
                }
            }

            // Expand macros in the line
            let expanded_line = self.expand_macros(line);
            result.push_str(&expanded_line);
            result.push('\n');
        }

        Ok(result)
    }

    fn handle_define(&mut self, define_content: &str) -> Result<(), ParseError> {
        // Parse `define MACRO_NAME value
        let parts: Vec<&str> = define_content.splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(ParseError {
                message: "Empty define directive".to_string(),
                location: None,
            });
        }

        let macro_name = parts[0].to_string();
        let macro_value = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };

        self.defines.insert(macro_name, macro_value);
        Ok(())
    }

    fn handle_include(
        &mut self,
        include_content: &str,
        current_file: Option<&Path>,
        line_num: usize,
    ) -> Result<String, ParseError> {
        // Parse `include "filename" or `include <filename>
        let filename = include_content.trim();
        let filename = if filename.starts_with('"') && filename.ends_with('"') {
            &filename[1..filename.len() - 1]
        } else if filename.starts_with('<') && filename.ends_with('>') {
            &filename[1..filename.len() - 1]
        } else {
            filename
        };

        // Try to find the file in include directories
        let mut found_path = None;

        // First try relative to current file
        if let Some(current) = current_file {
            if let Some(parent) = current.parent() {
                let candidate = parent.join(filename);
                if candidate.exists() {
                    found_path = Some(candidate);
                }
            }
        }

        // Then try include directories
        if found_path.is_none() {
            for inc_dir in &self.include_dirs {
                let candidate = inc_dir.join(filename);
                if candidate.exists() {
                    found_path = Some(candidate);
                    break;
                }
            }
        }

        let include_path = found_path.ok_or_else(|| ParseError {
            message: format!("Include file '{}' not found", filename),
            location: Some((line_num, 1)),
        })?;

        // Recursively preprocess the included file
        self.preprocess_file(&include_path)
    }

    fn expand_macros(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Simple macro expansion - replace all occurrences
        for (macro_name, macro_value) in &self.defines {
            // Replace macro with backtick prefix
            let macro_with_backtick = format!("`{}", macro_name);
            result = result.replace(&macro_with_backtick, macro_value);

            // Also replace bare macro names (without backtick) in some contexts
            // This is a simplified approach - real SystemVerilog has more complex rules
            if result.contains(macro_name) {
                // Only replace if it's a whole word (not part of another identifier)
                let words: Vec<&str> = result.split_whitespace().collect();
                let expanded_words: Vec<String> = words
                    .iter()
                    .map(|word| {
                        if word == &macro_name {
                            macro_value.clone()
                        } else {
                            word.to_string()
                        }
                    })
                    .collect();
                result = expanded_words.join(" ");
            }
        }

        result
    }
}
