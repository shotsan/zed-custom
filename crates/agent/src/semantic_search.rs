use anyhow::{Context as _, Result};
use collections::HashMap;
use gpui::{App, Entity, Task};
use project::Project;
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Class,
    Struct,
    Enum,
    Variable,
    Namespace,
}

#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub file_path: PathBuf,
    pub line_range: Range<u32>,
    pub symbol_type: SymbolType,
    pub name: String,
    pub signature: Option<String>,
}

pub struct SemanticIndex {
    symbols: HashMap<String, Vec<SymbolLocation>>,
    symbols_by_file: HashMap<PathBuf, Vec<SymbolLocation>>,
}

impl SemanticIndex {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::default(),
            symbols_by_file: HashMap::default(),
        }
    }

    pub fn overwrite(&mut self, other: Self) {
        self.symbols = other.symbols;
        self.symbols_by_file = other.symbols_by_file;
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn build(project: Entity<Project>, cx: &mut App) -> Task<Result<Self>> {
        let worktrees = project.read(cx).visible_worktrees(cx).collect::<Vec<_>>();
        
        cx.spawn(async move |cx| {
            let mut index = SemanticIndex::new();
            
            for worktree in worktrees {
                let entries = worktree.read_with(cx, |tree, _| {
                    tree.entries(false, 0)
                        .filter(|entry| {
                            entry.is_file() && 
                            entry.path.extension()
                                .map(|ext: &str| ext)
                                .map(|ext| matches!(ext, 
                                    // C/C++
                                    "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" | "hxx" |
                                    // Python
                                    "py" | "pyi" |
                                    // JavaScript/TypeScript
                                    "js" | "jsx" | "ts" | "tsx" |
                                    // Rust
                                    "rs" |
                                    // Go
                                    "go" |
                                    // Java
                                    "java" |
                                    // C#
                                    "cs"
                                ))
                                .unwrap_or(false)
                        })
                        .map(|entry| {
                            let abs_path = tree.abs_path().join(entry.path.as_unix_str());
                            (abs_path, entry.path.to_rel_path_buf())
                        })
                        .collect::<Vec<_>>()
                });

                   for (abs_path, _rel_path) in entries {
                        if let Ok(symbols) = Self::parse_file(&abs_path).await {
                            for symbol in symbols {
                                index.symbols
                                    .entry(symbol.name.clone())
                                    .or_insert_with(Vec::new)
                                    .push(symbol.clone());
                                
                                index.symbols_by_file
                                    .entry(symbol.file_path.clone())
                                    .or_insert_with(Vec::new)
                                    .push(symbol);
                            }
                        }
                    }
            }
            
            Ok(index)
        })
    }

    async fn parse_file(path: &PathBuf) -> Result<Vec<SymbolLocation>> {
        let content = smol::fs::read_to_string(path)
            .await
            .context("Failed to read file")?;
        
        let mut symbols = Vec::new();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        // Language-specific parsing patterns
        match ext {
            "py" | "pyi" => {
                // Python: def function_name(...) and class ClassName
                let function_pattern = regex::Regex::new(
                    r"(?m)^\s*def\s+(\w+)\s*\("
                )?;
                let class_pattern = regex::Regex::new(
                    r"(?m)^\s*class\s+(\w+)"
                )?;
                
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(caps) = function_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Function,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                    
                    if let Some(caps) = class_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Class,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                }
            }
            "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" | "hxx" => {
                // C/C++: function definitions and class/struct declarations
                let function_pattern = regex::Regex::new(
                    r"(?m)^(?:(?:inline|static|virtual|explicit|constexpr)\s+)*(?:\w+(?:::\w+)*(?:<[^>]+>)?)\s+(\w+)\s*\([^)]*\)\s*(?:const)?\s*(?:override)?\s*(?:final)?\s*\{"
                )?;
                let class_pattern = regex::Regex::new(
                    r"(?m)^(?:class|struct)\s+(\w+)"
                )?;
                
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(caps) = function_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Function,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                    
                    if let Some(caps) = class_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Class,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                }
            }
            "js" | "jsx" | "ts" | "tsx" => {
                // JavaScript/TypeScript: function declarations and class declarations
                let function_pattern = regex::Regex::new(
                    r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\("
                )?;
                let class_pattern = regex::Regex::new(
                    r"(?m)^\s*(?:export\s+)?class\s+(\w+)"
                )?;
                
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(caps) = function_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Function,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                    
                    if let Some(caps) = class_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Class,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                }
            }
            _ => {
                // Generic fallback: look for common patterns
                let function_pattern = regex::Regex::new(
                    r"(?m)^\s*(?:pub\s+)?(?:fn|func|def|function)\s+(\w+)"
                )?;
                
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(caps) = function_pattern.captures(line) {
                        if let Some(name) = caps.get(1) {
                            symbols.push(SymbolLocation {
                                file_path: path.clone(),
                                line_range: line_num as u32..(line_num as u32 + 1),
                                symbol_type: SymbolType::Function,
                                name: name.as_str().to_string(),
                                signature: Some(line.trim().to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(symbols)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SymbolLocation> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        // Exact matches first
        if let Some(symbols) = self.symbols.get(query) {
            results.extend(symbols.iter().cloned());
        }
        
        // Prefix matches
        for (name, symbols) in &self.symbols {
            if name.to_lowercase().starts_with(&query_lower) && name != query {
                results.extend(symbols.iter().cloned());
            }
        }
        
        // Substring matches
        for (name, symbols) in &self.symbols {
            if name.to_lowercase().contains(&query_lower) 
                && !name.to_lowercase().starts_with(&query_lower) 
            {
                results.extend(symbols.iter().cloned());
            }
        }
        
        results.truncate(limit);
        results
    }

    pub fn get_file_symbols(&self, path: &PathBuf) -> Option<&Vec<SymbolLocation>> {
        self.symbols_by_file.get(path)
    }

    pub fn has_cpp_files(&self) -> bool {
        self.symbols_by_file.keys().any(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e, "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" | "hxx"))
                .unwrap_or(false)
        })
    }

    pub fn has_python_files(&self) -> bool {
        self.symbols_by_file.keys().any(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e, "py" | "pyi"))
                .unwrap_or(false)
        })
    }

    pub fn update_file(&mut self, path: PathBuf, symbols: Vec<SymbolLocation>) {
        // Remove old symbols for this file
        if let Some(old_symbols) = self.symbols_by_file.remove(&path) {
            for old_symbol in old_symbols {
                if let Some(symbol_list) = self.symbols.get_mut(&old_symbol.name) {
                    symbol_list.retain(|s| s.file_path != path);
                    if symbol_list.is_empty() {
                        self.symbols.remove(&old_symbol.name);
                    }
                }
            }
        }
        
        // Add new symbols
        for symbol in &symbols {
            self.symbols
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol.clone());
        }
        
        self.symbols_by_file.insert(path, symbols);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_search() {
        let mut index = SemanticIndex::new();
        
        let symbol = SymbolLocation {
            file_path: PathBuf::from("/test/foo.cpp"),
            line_range: 10..15,
            symbol_type: SymbolType::Function,
            name: "calculateSum".to_string(),
            signature: Some("int calculateSum(int a, int b)".to_string()),
        };
        
        index.symbols.insert("calculateSum".to_string(), vec![symbol]);
        
        let results = index.search("calculate", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "calculateSum");
    }

    #[test]
    fn test_update_file() {
        let mut index = SemanticIndex::new();
        let path = PathBuf::from("/test/foo.cpp");
        
        let symbol1 = SymbolLocation {
            file_path: path.clone(),
            line_range: 10..15,
            symbol_type: SymbolType::Function,
            name: "oldFunction".to_string(),
            signature: None,
        };
        
        index.update_file(path.clone(), vec![symbol1]);
        assert!(index.symbols.contains_key("oldFunction"));
        
        let symbol2 = SymbolLocation {
            file_path: path.clone(),
            line_range: 20..25,
            symbol_type: SymbolType::Function,
            name: "newFunction".to_string(),
            signature: None,
        };
        
        index.update_file(path, vec![symbol2]);
        assert!(!index.symbols.contains_key("oldFunction"));
        assert!(index.symbols.contains_key("newFunction"));
    }
}
