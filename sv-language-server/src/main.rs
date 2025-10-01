use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sv_parser::{Expression, ModuleItem, SourceUnit, SystemVerilogParser};
use tokio::io::{stdin, stdout};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    /// Include directories for SystemVerilog (+incdir+)
    #[serde(default)]
    include_directories: Vec<String>,

    /// Preprocessor defines (+define+)
    #[serde(default)]
    defines: HashMap<String, Option<String>>,

    /// Source directories to search for files
    #[serde(default)]
    source_directories: Vec<String>,

    /// Override config file location
    #[serde(skip_serializing_if = "Option::is_none")]
    config_file_path: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            include_directories: Vec::new(),
            defines: HashMap::new(),
            source_directories: Vec::new(),
            config_file_path: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    symbol_type: SymbolType,
    range: Range,
    uri: Url,
}

#[derive(Debug, Clone)]
enum SymbolType {
    Module,
    Variable,
    Port,
}

#[derive(Debug, Clone)]
struct DocumentState {
    content: String,
    ast: Option<SourceUnit>,
    symbols: Vec<Symbol>,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
    workspace_symbols: Arc<RwLock<HashMap<String, Vec<Symbol>>>>, // symbol_name -> all locations
    config: Arc<RwLock<ServerConfig>>,
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        // Store workspace root
        {
            let mut workspace_root = self.workspace_root.write().await;
            *workspace_root = params.root_uri.and_then(|uri| uri.to_file_path().ok());
        }

        // Load configuration from initialization options
        let mut config = ServerConfig::default();
        if let Some(init_options) = params.initialization_options {
            match serde_json::from_value::<ServerConfig>(init_options) {
                Ok(parsed_config) => {
                    config = parsed_config;
                    self.client
                        .log_message(
                            MessageType::INFO,
                            "Configuration loaded from initialization options",
                        )
                        .await;
                }
                Err(e) => {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Failed to parse initialization options: {}", e),
                        )
                        .await;
                }
            }
        }

        // Try to load from config file if not provided in initialization options
        if config.include_directories.is_empty()
            && config.defines.is_empty()
            && config.source_directories.is_empty()
        {
            if let Some(file_config) = self.load_config_file(&config).await {
                config = file_config;
            }
        }

        // Validate and store the configuration
        self.validate_config(&config).await;
        {
            let mut stored_config = self.config.write().await;
            *stored_config = config;
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "sv-language-server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("sv-language-server".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                rename_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "SystemVerilog Language Server initialized!",
            )
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            language_id: params.text_document.language_id,
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let new_text = std::mem::take(&mut params.content_changes[0].text);

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: new_text,
            language_id: "systemverilog".to_string(),
            version: params.text_document.version,
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("file saved: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Remove document from storage and workspace symbols
        {
            let mut docs = self.documents.write().await;
            if let Some(doc_state) = docs.remove(&params.text_document.uri) {
                // Remove symbols from workspace index
                let mut workspace_symbols = self.workspace_symbols.write().await;
                for symbol in doc_state.symbols {
                    if let Some(symbol_list) = workspace_symbols.get_mut(&symbol.name) {
                        symbol_list.retain(|s| s.uri != params.text_document.uri);
                        if symbol_list.is_empty() {
                            workspace_symbols.remove(&symbol.name);
                        }
                    }
                }
            }
        }

        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn diagnostic(
        &self,
        _params: DocumentDiagnosticParams,
    ) -> LspResult<DocumentDiagnosticReportResult> {
        self.client
            .log_message(MessageType::INFO, "diagnostics requested!")
            .await;

        // For now, return no diagnostics - we'll implement parsing-based diagnostics later
        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: vec![],
                },
            }),
        ))
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        self.client
            .log_message(MessageType::INFO, "rename requested!")
            .await;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Get the document state
        let doc_symbols = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => doc_state.symbols.clone(),
                None => {
                    self.client
                        .log_message(MessageType::ERROR, "Document not found")
                        .await;
                    return Ok(None);
                }
            }
        };

        // Find the symbol at the requested position
        let symbol_at_position = doc_symbols
            .iter()
            .find(|symbol| self.position_in_range(position, symbol.range));

        if let Some(symbol) = symbol_at_position {
            // Find all workspace references to this symbol
            let workspace_references = {
                let workspace_symbols = self.workspace_symbols.read().await;
                workspace_symbols
                    .get(&symbol.name)
                    .cloned()
                    .unwrap_or_default()
            };

            if workspace_references.is_empty() {
                return Ok(None);
            }

            // Group edits by file URI
            let mut changes = HashMap::new();
            for reference in workspace_references {
                // Only rename symbols of compatible types
                let should_rename = match (&symbol.symbol_type, &reference.symbol_type) {
                    (SymbolType::Module, SymbolType::Module) => true,
                    (SymbolType::Variable, _) | (SymbolType::Port, _) => true,
                    (_, SymbolType::Variable) | (_, SymbolType::Port) => true,
                };

                if should_rename {
                    changes
                        .entry(reference.uri.clone())
                        .or_insert_with(Vec::new)
                        .push(TextEdit {
                            range: reference.range,
                            new_text: new_name.clone(),
                        });
                }
            }

            if changes.is_empty() {
                return Ok(None);
            }

            return Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }));
        }

        Ok(None)
    }

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> LspResult<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;

        // Get the document AST
        let ast = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => doc_state.ast.clone(),
                None => {
                    self.client
                        .log_message(MessageType::WARNING, "Document not found for folding")
                        .await;
                    return Ok(None);
                }
            }
        };

        if let Some(ast) = ast {
            let content = {
                let docs = self.documents.read().await;
                docs.get(&uri)
                    .map(|doc| doc.content.clone())
                    .unwrap_or_default()
            };

            let folding_ranges = self.extract_folding_ranges(&ast, &content);
            Ok(Some(folding_ranges))
        } else {
            Ok(None)
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "Configuration changed")
            .await;

        // Try to extract configuration from the settings
        if let Ok(config) = serde_json::from_value::<ServerConfig>(params.settings) {
            let mut stored_config = self.config.write().await;
            *stored_config = config;

            self.client
                .log_message(MessageType::INFO, "Configuration updated successfully")
                .await;
        } else {
            self.client
                .log_message(MessageType::WARNING, "Failed to parse new configuration")
                .await;
        }
    }
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        // Parse and cache AST, extract symbols, and validate
        let (diagnostics, ast, symbols) = self
            .parse_and_analyze_document(&params.text, &params.uri)
            .await;

        // Update document state
        {
            let mut docs = self.documents.write().await;
            let old_doc = docs.insert(
                params.uri.clone(),
                DocumentState {
                    content: params.text.clone(),
                    ast: ast.clone(),
                    symbols: symbols.clone(),
                },
            );

            // Update workspace symbol index
            let mut workspace_symbols = self.workspace_symbols.write().await;

            // Remove old symbols from this document
            if let Some(old_state) = old_doc {
                for old_symbol in old_state.symbols {
                    if let Some(symbol_list) = workspace_symbols.get_mut(&old_symbol.name) {
                        symbol_list.retain(|s| s.uri != params.uri);
                        if symbol_list.is_empty() {
                            workspace_symbols.remove(&old_symbol.name);
                        }
                    }
                }
            }

            // Add new symbols
            for symbol in symbols {
                workspace_symbols
                    .entry(symbol.name.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol);
            }
        }

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;
    }

    async fn parse_and_analyze_document(
        &self,
        text: &str,
        uri: &Url,
    ) -> (Vec<Diagnostic>, Option<SourceUnit>, Vec<Symbol>) {
        let mut diagnostics = Vec::new();
        let mut ast = None;
        let mut symbols = Vec::new();

        // Get configuration for parser
        let (include_paths, defines) = {
            let config = self.config.read().await;
            let workspace_root = self.workspace_root.read().await;

            // Convert include directories to absolute paths
            let mut include_paths = Vec::new();
            if let Some(root) = workspace_root.as_ref() {
                for include_dir in &config.include_directories {
                    let path = if std::path::Path::new(include_dir).is_absolute() {
                        PathBuf::from(include_dir)
                    } else {
                        root.join(include_dir)
                    };
                    include_paths.push(path);
                }
            }

            // Convert defines to parser format
            let mut defines = HashMap::new();
            for (key, value) in &config.defines {
                defines.insert(key.clone(), value.clone().unwrap_or_default());
            }

            (include_paths, defines)
        };

        // Create parser with configuration
        let parser = SystemVerilogParser::new(include_paths, defines);

        match parser.parse_content(text) {
            Ok(parsed_ast) => {
                // Parse succeeded - extract symbols from AST
                self.client
                    .log_message(MessageType::INFO, "Parse succeeded!")
                    .await;

                symbols = self.extract_symbols_from_ast(&parsed_ast, text, uri);
                ast = Some(parsed_ast);
            }
            Err(parse_error) => {
                // Parse failed - convert each error to a diagnostic
                self.client
                    .log_message(MessageType::INFO, format!("Parse failed: {}", parse_error))
                    .await;

                for error in &parse_error.errors {
                    let range = if let Some(location) = &error.location {
                        // Debug log the location info
                        self.client
                            .log_message(
                                MessageType::INFO,
                                format!(
                                    "Error location: line={}, col={}, span={:?}",
                                    location.line, location.column, location.span
                                ),
                            )
                            .await;

                        if let Some((start_char, end_char)) = location.span {
                            // Use span positions to calculate precise range
                            let start_pos = self
                                .char_offset_to_position(text, start_char)
                                .unwrap_or_else(|| {
                                    Position::new(location.line as u32, location.column as u32)
                                });
                            let end_pos = self
                                .char_offset_to_position(text, end_char)
                                .unwrap_or_else(|| {
                                    Position::new(location.line as u32, location.column as u32 + 1)
                                });

                            Range::new(start_pos, end_pos)
                        } else {
                            // Use line/column directly
                            let start_pos =
                                Position::new(location.line as u32, location.column as u32);
                            let end_pos =
                                Position::new(location.line as u32, location.column as u32 + 1);

                            Range::new(start_pos, end_pos)
                        }
                    } else {
                        // Fallback to start of document
                        self.client
                            .log_message(MessageType::WARNING, "No location info for error")
                            .await;
                        Range::new(Position::new(0, 0), Position::new(0, 1))
                    };

                    let severity = match error.error_type {
                        sv_parser::ParseErrorType::UnsupportedFeature(_) => {
                            DiagnosticSeverity::WARNING
                        }
                        sv_parser::ParseErrorType::PreprocessorError => DiagnosticSeverity::ERROR,
                        _ => DiagnosticSeverity::ERROR,
                    };

                    let mut diagnostic = Diagnostic {
                        range,
                        severity: Some(severity),
                        code: None,
                        code_description: None,
                        source: Some("sv-parser".to_string()),
                        message: error.message.clone(),
                        related_information: None,
                        tags: None,
                        data: None,
                    };

                    // Add suggestions as related information if available
                    if !error.suggestions.is_empty() {
                        diagnostic.message = format!(
                            "{}\n\nSuggestions:\n{}",
                            error.message,
                            error
                                .suggestions
                                .iter()
                                .map(|s| format!("  • {}", s))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                    }

                    diagnostics.push(diagnostic);
                }
            }
        }

        (diagnostics, ast, symbols)
    }

    // Helper function to convert character offset to LSP Position
    fn char_offset_to_position(&self, text: &str, offset: usize) -> Option<Position> {
        if offset > text.len() {
            // Clamp to end of text
            let prefix = text;
            let line = prefix.matches('\n').count();
            let column = prefix.split('\n').last().unwrap_or("").len();
            return Some(Position::new(line as u32, column as u32));
        }

        let prefix = &text[..offset];
        let line = prefix.matches('\n').count();
        let column = prefix.split('\n').last().unwrap_or("").len();

        Some(Position::new(line as u32, column as u32))
    }

    // Extract symbols from AST recursively
    fn extract_symbols_from_ast(&self, ast: &SourceUnit, content: &str, uri: &Url) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        for item in &ast.items {
            self.extract_symbols_from_module_item(item, content, uri, &mut symbols);
        }
        symbols
    }

    // Extract symbols from a module item
    fn extract_symbols_from_module_item(
        &self,
        item: &ModuleItem,
        content: &str,
        uri: &Url,
        symbols: &mut Vec<Symbol>,
    ) {
        match item {
            ModuleItem::ModuleDeclaration { name, ports, items } => {
                // Add module name as a symbol
                if let Some(range) = self.find_identifier_range(content, name) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Module,
                        range,
                        uri: uri.clone(),
                    });
                }

                // Add port names as symbols
                for port in ports {
                    if let Some(range) = self.find_identifier_range(content, &port.name) {
                        symbols.push(Symbol {
                            name: port.name.clone(),
                            symbol_type: SymbolType::Port,
                            range,
                            uri: uri.clone(),
                        });
                    }
                }

                // Recursively process module items
                for sub_item in items {
                    self.extract_symbols_from_module_item(sub_item, content, uri, symbols);
                }
            }
            ModuleItem::PortDeclaration { name, .. } => {
                if let Some(range) = self.find_identifier_range(content, name) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Port,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            ModuleItem::Assignment { target, expr } => {
                // Add assignment target as variable
                if let Some(range) = self.find_identifier_range(content, target) {
                    symbols.push(Symbol {
                        name: target.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }

                // Extract identifiers from expression
                self.extract_symbols_from_expression(expr, content, uri, symbols);
            }
        }
    }

    // Extract symbols from expressions
    fn extract_symbols_from_expression(
        &self,
        expr: &Expression,
        content: &str,
        uri: &Url,
        symbols: &mut Vec<Symbol>,
    ) {
        match expr {
            Expression::Identifier(name) => {
                if let Some(range) = self.find_identifier_range(content, name) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            Expression::Binary { left, right, .. } => {
                self.extract_symbols_from_expression(left, content, uri, symbols);
                self.extract_symbols_from_expression(right, content, uri, symbols);
            }
            Expression::Unary { operand, .. } => {
                self.extract_symbols_from_expression(operand, content, uri, symbols);
            }
            Expression::Number(_) => {
                // Numbers are not identifiers we care about for renaming
            }
        }
    }

    // Find the range of an identifier in the source text
    fn find_identifier_range(&self, content: &str, identifier: &str) -> Option<Range> {
        // Simple implementation: find first occurrence
        // In a real implementation, you'd want to track locations during parsing
        if let Some(start) = content.find(identifier) {
            let start_pos = self.char_offset_to_position(content, start)?;
            let end_pos = self.char_offset_to_position(content, start + identifier.len())?;
            Some(Range::new(start_pos, end_pos))
        } else {
            None
        }
    }

    // Check if a position is within a range
    fn position_in_range(&self, position: Position, range: Range) -> bool {
        (position.line > range.start.line
            || (position.line == range.start.line && position.character >= range.start.character))
            && (position.line < range.end.line
                || (position.line == range.end.line && position.character <= range.end.character))
    }

    // Load configuration from TOML file
    async fn load_config_file(&self, config: &ServerConfig) -> Option<ServerConfig> {
        let workspace_root = self.workspace_root.read().await;
        let workspace_path = workspace_root.as_ref()?;

        // Use custom config file path if specified, otherwise use default
        let config_path = if let Some(custom_path) = &config.config_file_path {
            workspace_path.join(custom_path)
        } else {
            workspace_path.join(".sv-lsp.toml")
        };

        match tokio::fs::read_to_string(&config_path).await {
            Ok(content) => match toml::from_str::<ServerConfig>(&content) {
                Ok(file_config) => {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Configuration loaded from {}", config_path.display()),
                        )
                        .await;
                    Some(file_config)
                }
                Err(e) => {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!(
                                "Failed to parse config file {}: {}",
                                config_path.display(),
                                e
                            ),
                        )
                        .await;
                    None
                }
            },
            Err(_) => {
                // Config file not found, which is fine
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "No config file found at {}, using defaults",
                            config_path.display()
                        ),
                    )
                    .await;
                None
            }
        }
    }

    // Validate configuration and warn about issues
    async fn validate_config(&self, config: &ServerConfig) {
        let workspace_root = self.workspace_root.read().await;

        // Validate include directories exist
        if let Some(root) = workspace_root.as_ref() {
            for include_dir in &config.include_directories {
                let path = if std::path::Path::new(include_dir).is_absolute() {
                    PathBuf::from(include_dir)
                } else {
                    root.join(include_dir)
                };

                if !path.exists() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Include directory does not exist: {}", path.display()),
                        )
                        .await;
                } else if !path.is_dir() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Include path is not a directory: {}", path.display()),
                        )
                        .await;
                }
            }

            // Validate source directories exist
            for source_dir in &config.source_directories {
                let path = if std::path::Path::new(source_dir).is_absolute() {
                    PathBuf::from(source_dir)
                } else {
                    root.join(source_dir)
                };

                if !path.exists() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Source directory does not exist: {}", path.display()),
                        )
                        .await;
                } else if !path.is_dir() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Source path is not a directory: {}", path.display()),
                        )
                        .await;
                }
            }
        }

        // Validate custom config file path if specified
        if let Some(config_path) = &config.config_file_path {
            if let Some(root) = workspace_root.as_ref() {
                let full_path = root.join(config_path);
                if !full_path.exists() {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Custom config file does not exist: {}", full_path.display()),
                        )
                        .await;
                }
            }
        }

        // Log configuration summary
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Configuration: {} include dirs, {} defines, {} source dirs",
                    config.include_directories.len(),
                    config.defines.len(),
                    config.source_directories.len()
                ),
            )
            .await;
    }

    // Extract folding ranges from AST
    fn extract_folding_ranges(&self, ast: &SourceUnit, content: &str) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();

        for item in &ast.items {
            self.extract_folding_ranges_from_item(item, content, &mut ranges);
        }

        ranges
    }

    fn extract_folding_ranges_from_item(
        &self,
        item: &ModuleItem,
        content: &str,
        ranges: &mut Vec<FoldingRange>,
    ) {
        match item {
            ModuleItem::ModuleDeclaration { name, items, .. } => {
                // Find the module folding range
                if let Some(range) = self.find_module_folding_range(name, content) {
                    ranges.push(range);
                }

                // Recursively process nested items
                for sub_item in items {
                    self.extract_folding_ranges_from_item(sub_item, content, ranges);
                }
            }
            ModuleItem::Assignment { .. } => {
                // Assignments typically don't need folding unless they're complex
                // Could add support for complex multi-line assignments here
            }
            ModuleItem::PortDeclaration { .. } => {
                // Port declarations usually don't need folding
            }
        }
    }

    fn find_module_folding_range(&self, module_name: &str, content: &str) -> Option<FoldingRange> {
        // Find the module declaration and its corresponding endmodule
        let module_pattern = format!("module {}", module_name);
        let start_pos = content.find(&module_pattern)?;

        // Find the line number for the start
        let start_line = content[..start_pos].matches('\n').count();

        // Find the corresponding endmodule
        // This is a simplified approach - in a real implementation you'd want to handle nested modules
        let endmodule_pos = content.rfind("endmodule")?;
        let end_line = content[..endmodule_pos].matches('\n').count();

        // Make sure the endmodule is after the module
        if end_line > start_line {
            Some(FoldingRange {
                start_line: start_line as u32,
                start_character: None,
                end_line: end_line as u32,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: Some(format!("module {} ...", module_name)),
            })
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(RwLock::new(HashMap::new())),
        workspace_symbols: Arc::new(RwLock::new(HashMap::new())),
        config: Arc::new(RwLock::new(ServerConfig::default())),
        workspace_root: Arc::new(RwLock::new(None)),
    });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
