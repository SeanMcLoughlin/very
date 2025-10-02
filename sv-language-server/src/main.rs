use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sv_parser::{Expression, ModuleItem, SourceUnit, SystemVerilogParser};
use tokio::io::{stdin, stdout};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::request::{
    GotoDeclarationParams, GotoDeclarationResponse, GotoImplementationParams,
    GotoImplementationResponse, GotoTypeDefinitionParams, GotoTypeDefinitionResponse,
};
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
    Class,
    Function,
    #[allow(dead_code)]
    Task,
    Variable,
    Port,
    #[allow(dead_code)]
    Parameter,
    Define,
    Include,
}

#[derive(Debug, Clone)]
struct SystemFunctionInfo {
    signature: String,
    description: String,
}

fn get_system_function_info(name: &str) -> Option<SystemFunctionInfo> {
    match name {
        // Math functions (Chapter 20.8)
        "sin" => Some(SystemFunctionInfo {
            signature: "function real $sin(real x)".to_string(),
            description: "Returns the sine of x (x in radians)".to_string(),
        }),
        "cos" => Some(SystemFunctionInfo {
            signature: "function real $cos(real x)".to_string(),
            description: "Returns the cosine of x (x in radians)".to_string(),
        }),
        "tan" => Some(SystemFunctionInfo {
            signature: "function real $tan(real x)".to_string(),
            description: "Returns the tangent of x (x in radians)".to_string(),
        }),
        "asin" => Some(SystemFunctionInfo {
            signature: "function real $asin(real x)".to_string(),
            description: "Returns the arc sine of x".to_string(),
        }),
        "acos" => Some(SystemFunctionInfo {
            signature: "function real $acos(real x)".to_string(),
            description: "Returns the arc cosine of x".to_string(),
        }),
        "atan" => Some(SystemFunctionInfo {
            signature: "function real $atan(real x)".to_string(),
            description: "Returns the arc tangent of x".to_string(),
        }),
        "atan2" => Some(SystemFunctionInfo {
            signature: "function real $atan2(real y, real x)".to_string(),
            description: "Returns the arc tangent of y/x".to_string(),
        }),
        "sinh" => Some(SystemFunctionInfo {
            signature: "function real $sinh(real x)".to_string(),
            description: "Returns the hyperbolic sine of x".to_string(),
        }),
        "cosh" => Some(SystemFunctionInfo {
            signature: "function real $cosh(real x)".to_string(),
            description: "Returns the hyperbolic cosine of x".to_string(),
        }),
        "tanh" => Some(SystemFunctionInfo {
            signature: "function real $tanh(real x)".to_string(),
            description: "Returns the hyperbolic tangent of x".to_string(),
        }),
        "asinh" => Some(SystemFunctionInfo {
            signature: "function real $asinh(real x)".to_string(),
            description: "Returns the inverse hyperbolic sine of x".to_string(),
        }),
        "acosh" => Some(SystemFunctionInfo {
            signature: "function real $acosh(real x)".to_string(),
            description: "Returns the inverse hyperbolic cosine of x".to_string(),
        }),
        "atanh" => Some(SystemFunctionInfo {
            signature: "function real $atanh(real x)".to_string(),
            description: "Returns the inverse hyperbolic tangent of x".to_string(),
        }),
        "exp" => Some(SystemFunctionInfo {
            signature: "function real $exp(real x)".to_string(),
            description: "Returns e to the power of x".to_string(),
        }),
        "ln" => Some(SystemFunctionInfo {
            signature: "function real $ln(real x)".to_string(),
            description: "Returns the natural logarithm of x".to_string(),
        }),
        "log10" => Some(SystemFunctionInfo {
            signature: "function real $log10(real x)".to_string(),
            description: "Returns the base-10 logarithm of x".to_string(),
        }),
        "sqrt" => Some(SystemFunctionInfo {
            signature: "function real $sqrt(real x)".to_string(),
            description: "Returns the square root of x".to_string(),
        }),
        "pow" => Some(SystemFunctionInfo {
            signature: "function real $pow(real x, real y)".to_string(),
            description: "Returns x to the power of y".to_string(),
        }),
        "hypot" => Some(SystemFunctionInfo {
            signature: "function real $hypot(real x, real y)".to_string(),
            description: "Returns sqrt(x^2 + y^2)".to_string(),
        }),
        "floor" => Some(SystemFunctionInfo {
            signature: "function real $floor(real x)".to_string(),
            description: "Returns the largest integer not greater than x".to_string(),
        }),
        "ceil" => Some(SystemFunctionInfo {
            signature: "function real $ceil(real x)".to_string(),
            description: "Returns the smallest integer not less than x".to_string(),
        }),
        "clog2" => Some(SystemFunctionInfo {
            signature: "function integer $clog2(integer value)".to_string(),
            description: "Returns the ceiling of log base 2 of value".to_string(),
        }),

        // Display tasks (Chapter 20)
        "display" => Some(SystemFunctionInfo {
            signature: "task $display([list_of_arguments])".to_string(),
            description: "Displays the argument list and adds a newline".to_string(),
        }),
        "write" => Some(SystemFunctionInfo {
            signature: "task $write([list_of_arguments])".to_string(),
            description: "Displays the argument list without adding a newline".to_string(),
        }),
        "monitor" => Some(SystemFunctionInfo {
            signature: "task $monitor([list_of_arguments])".to_string(),
            description: "Continuously monitors and displays values when they change".to_string(),
        }),

        // Simulation control (Chapter 20.2)
        "finish" => Some(SystemFunctionInfo {
            signature: "task $finish[(n)]".to_string(),
            description: "Terminates the simulation".to_string(),
        }),
        "stop" => Some(SystemFunctionInfo {
            signature: "task $stop[(n)]".to_string(),
            description: "Suspends the simulation".to_string(),
        }),
        "exit" => Some(SystemFunctionInfo {
            signature: "task $exit".to_string(),
            description: "Terminates the simulation".to_string(),
        }),

        // Time functions (Chapter 20.3)
        "time" => Some(SystemFunctionInfo {
            signature: "function time $time".to_string(),
            description: "Returns the current simulation time as a 64-bit integer".to_string(),
        }),
        "stime" => Some(SystemFunctionInfo {
            signature: "function int $stime".to_string(),
            description: "Returns the current simulation time as a 32-bit integer".to_string(),
        }),
        "realtime" => Some(SystemFunctionInfo {
            signature: "function realtime $realtime".to_string(),
            description: "Returns the current simulation time as a real number".to_string(),
        }),

        // Random number generation (Chapter 20.15)
        "random" => Some(SystemFunctionInfo {
            signature: "function int $random[(seed)]".to_string(),
            description: "Returns a random 32-bit signed integer".to_string(),
        }),
        "urandom" => Some(SystemFunctionInfo {
            signature: "function int $urandom[(seed)]".to_string(),
            description: "Returns a random 32-bit unsigned integer".to_string(),
        }),
        "urandom_range" => Some(SystemFunctionInfo {
            signature: "function int $urandom_range(int maxval, int minval = 0)".to_string(),
            description: "Returns a random integer within the specified range".to_string(),
        }),

        _ => None,
    }
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
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
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

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Rename at position {}:{}",
                    position.line, position.character
                ),
            )
            .await;

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

        self.client
            .log_message(
                MessageType::INFO,
                format!("Found {} symbols in document", doc_symbols.len()),
            )
            .await;

        // Log all symbols with their ranges
        for symbol in &doc_symbols {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "Symbol '{}' at {}:{}-{}:{}",
                        symbol.name,
                        symbol.range.start.line,
                        symbol.range.start.character,
                        symbol.range.end.line,
                        symbol.range.end.character
                    ),
                )
                .await;
        }

        // Find the symbol at the requested position
        let symbol_at_position = doc_symbols
            .iter()
            .find(|symbol| self.position_in_range(position, symbol.range));

        if let Some(symbol) = symbol_at_position {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Found symbol '{}' at position", symbol.name),
                )
                .await;
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
                    (SymbolType::Class, SymbolType::Class) => true,
                    (SymbolType::Function, SymbolType::Function) => true,
                    (SymbolType::Task, SymbolType::Task) => true,
                    (SymbolType::Variable, _)
                    | (SymbolType::Port, _)
                    | (SymbolType::Parameter, _) => true,
                    (_, SymbolType::Variable)
                    | (_, SymbolType::Port)
                    | (_, SymbolType::Parameter) => true,
                    (SymbolType::Define, SymbolType::Define) => true,
                    (SymbolType::Include, SymbolType::Include) => true,
                    _ => false,
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

        self.client
            .log_message(MessageType::WARNING, "No symbol found at position")
            .await;
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "goto_definition called at {}:{}",
                    position.line, position.character
                ),
            )
            .await;

        // Find the symbol at the cursor position
        let symbol_info = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => {
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Document has {} symbols", doc_state.symbols.len()),
                        )
                        .await;

                    let found_symbol = doc_state
                        .symbols
                        .iter()
                        .find(|symbol| self.position_in_range(position, symbol.range))
                        .map(|s| (s.name.clone(), s.symbol_type.clone()));

                    if let Some((ref name, ref stype)) = found_symbol {
                        self.client
                            .log_message(
                                MessageType::INFO,
                                format!("Found symbol: name='{}', type={:?}", name, stype),
                            )
                            .await;
                    } else {
                        self.client
                            .log_message(MessageType::INFO, "No symbol found at position")
                            .await;
                    }

                    found_symbol
                }
                None => {
                    self.client
                        .log_message(MessageType::WARNING, "Document not found")
                        .await;
                    None
                }
            }
        };

        if let Some((name, symbol_type)) = symbol_info {
            // Special handling for include directives
            if matches!(symbol_type, SymbolType::Include) {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Processing include: path='{}'", name),
                    )
                    .await;

                // Try to resolve the include path
                let include_path = std::path::Path::new(&name);

                // If it's already an absolute path (resolved), use it directly
                let resolved = if include_path.is_absolute() {
                    Some(include_path.to_path_buf())
                } else {
                    // Try to resolve relative to the current file's directory
                    uri.to_file_path().ok().and_then(|current_file| {
                        let current_dir = current_file.parent()?;
                        let candidate = current_dir.join(&name);
                        if candidate.exists() {
                            return Some(candidate);
                        }

                        // Try looking in common include directories relative to current file
                        for include_dir in &["include", "../include", "../../include"] {
                            let candidate = current_dir.join(include_dir).join(&name);
                            if candidate.exists() {
                                return Some(candidate);
                            }
                        }
                        None
                    })
                };

                if let Some(resolved_path) = resolved {
                    if let Ok(file_uri) = Url::from_file_path(&resolved_path) {
                        self.client
                            .log_message(MessageType::INFO, format!("Navigating to: {}", file_uri))
                            .await;

                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: file_uri,
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 0,
                                },
                            },
                        })));
                    }
                }

                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Could not resolve include file: {}", name),
                    )
                    .await;
                return Ok(None);
            }

            // Look up all occurrences of this symbol
            let workspace_symbols = self.workspace_symbols.read().await;
            if let Some(symbol_list) = workspace_symbols.get(&name) {
                // For definition, we want the first declaration (typically the module/class/function declaration)
                // We'll prioritize declaration-type symbols (Module, Class, Function, Task, Port) as definitions
                let definition = symbol_list
                    .iter()
                    .find(|s| {
                        matches!(
                            s.symbol_type,
                            SymbolType::Module
                                | SymbolType::Class
                                | SymbolType::Function
                                | SymbolType::Task
                                | SymbolType::Port
                        )
                    })
                    .or_else(|| symbol_list.first());

                if let Some(def_symbol) = definition {
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: def_symbol.uri.clone(),
                        range: def_symbol.range,
                    })));
                }
            }
        }

        Ok(None)
    }

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> LspResult<Option<GotoDeclarationResponse>> {
        // For SystemVerilog, declaration is the same as definition
        let def_params = GotoDefinitionParams {
            text_document_position_params: params.text_document_position_params,
            work_done_progress_params: params.work_done_progress_params,
            partial_result_params: params.partial_result_params,
        };

        match self.goto_definition(def_params).await? {
            Some(GotoDefinitionResponse::Scalar(loc)) => {
                Ok(Some(GotoDeclarationResponse::Scalar(loc)))
            }
            Some(GotoDefinitionResponse::Array(locs)) => {
                Ok(Some(GotoDeclarationResponse::Array(locs)))
            }
            Some(GotoDefinitionResponse::Link(links)) => {
                Ok(Some(GotoDeclarationResponse::Link(links)))
            }
            None => Ok(None),
        }
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> LspResult<Option<GotoTypeDefinitionResponse>> {
        // For SystemVerilog, we'll look for module type definitions
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Find the symbol at the cursor position
        let symbol_name = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => doc_state
                    .symbols
                    .iter()
                    .find(|symbol| self.position_in_range(position, symbol.range))
                    .map(|s| s.name.clone()),
                None => None,
            }
        };

        if let Some(name) = symbol_name {
            // Look for module and class type definitions
            let workspace_symbols = self.workspace_symbols.read().await;
            if let Some(symbol_list) = workspace_symbols.get(&name) {
                let type_def = symbol_list
                    .iter()
                    .find(|s| matches!(s.symbol_type, SymbolType::Module | SymbolType::Class));

                if let Some(def_symbol) = type_def {
                    return Ok(Some(GotoTypeDefinitionResponse::Scalar(Location {
                        uri: def_symbol.uri.clone(),
                        range: def_symbol.range,
                    })));
                }
            }
        }

        Ok(None)
    }

    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> LspResult<Option<GotoImplementationResponse>> {
        // For SystemVerilog, implementation is similar to definition
        // We look for module instantiations
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Find the symbol at the cursor position
        let symbol_name = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => doc_state
                    .symbols
                    .iter()
                    .find(|symbol| self.position_in_range(position, symbol.range))
                    .map(|s| s.name.clone()),
                None => None,
            }
        };

        if let Some(name) = symbol_name {
            // Look for all module and class implementations with this name
            let workspace_symbols = self.workspace_symbols.read().await;
            if let Some(symbol_list) = workspace_symbols.get(&name) {
                let implementations: Vec<Location> = symbol_list
                    .iter()
                    .filter(|s| matches!(s.symbol_type, SymbolType::Module | SymbolType::Class))
                    .map(|s| Location {
                        uri: s.uri.clone(),
                        range: s.range,
                    })
                    .collect();

                if !implementations.is_empty() {
                    return Ok(Some(GotoImplementationResponse::Array(implementations)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Find the symbol at the cursor position
        let symbol_name = {
            let docs = self.documents.read().await;
            match docs.get(&uri) {
                Some(doc_state) => doc_state
                    .symbols
                    .iter()
                    .find(|symbol| self.position_in_range(position, symbol.range))
                    .map(|s| s.name.clone()),
                None => None,
            }
        };

        if let Some(name) = symbol_name {
            // Get all references to this symbol from the workspace
            let workspace_symbols = self.workspace_symbols.read().await;
            if let Some(symbol_list) = workspace_symbols.get(&name) {
                let references: Vec<Location> = symbol_list
                    .iter()
                    .map(|s| Location {
                        uri: s.uri.clone(),
                        range: s.range,
                    })
                    .collect();

                return Ok(Some(references));
            }
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> LspResult<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();

        let workspace_symbols = self.workspace_symbols.read().await;
        let mut results = Vec::new();

        // Search through all symbols
        for (name, symbols) in workspace_symbols.iter() {
            // Case-insensitive substring match
            if name.to_lowercase().contains(&query) {
                for symbol in symbols {
                    // Convert SymbolType to LSP SymbolKind and get display prefix
                    let (kind, type_prefix) = match symbol.symbol_type {
                        SymbolType::Module => (SymbolKind::MODULE, "module"),
                        SymbolType::Class => (SymbolKind::CLASS, "class"),
                        SymbolType::Function => (SymbolKind::FUNCTION, "function"),
                        SymbolType::Task => (SymbolKind::FUNCTION, "task"),
                        SymbolType::Variable => (SymbolKind::VARIABLE, "variable"),
                        SymbolType::Port => (SymbolKind::PROPERTY, "port"),
                        SymbolType::Parameter => (SymbolKind::CONSTANT, "parameter"),
                        SymbolType::Define => (SymbolKind::CONSTANT, "`define"),
                        SymbolType::Include => (SymbolKind::FILE, "`include"),
                    };

                    // Display name with type prefix (e.g., "module top")
                    let display_name = format!("{} {}", type_prefix, symbol.name);

                    #[allow(deprecated)]
                    results.push(SymbolInformation {
                        name: display_name,
                        kind,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: symbol.uri.clone(),
                            range: symbol.range,
                        },
                        container_name: None,
                    });
                }
            }
        }

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
        }
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        // Check if hovering over a system function call
        if let Some(ast) = &doc_state.ast {
            if let Some(hover_info) =
                self.find_hover_at_position(&ast.items, &doc_state.content, position)
            {
                return Ok(Some(hover_info));
            }
        }

        // Check if hovering over a symbol (module, variable, etc.)
        if let Some(symbol) = doc_state
            .symbols
            .iter()
            .find(|s| self.position_in_range(position, s.range))
        {
            let hover_text = match symbol.symbol_type {
                SymbolType::Module => format!("```systemverilog\nmodule {}\n```", symbol.name),
                SymbolType::Class => format!("```systemverilog\nclass {}\n```", symbol.name),
                SymbolType::Function => format!("```systemverilog\nfunction {}\n```", symbol.name),
                SymbolType::Task => format!("```systemverilog\ntask {}\n```", symbol.name),
                SymbolType::Variable => format!("```systemverilog\n{}\n```", symbol.name),
                SymbolType::Port => format!("```systemverilog\nport {}\n```", symbol.name),
                SymbolType::Parameter => {
                    format!("```systemverilog\nparameter {}\n```", symbol.name)
                }
                SymbolType::Define => format!("```systemverilog\n`define {}\n```", symbol.name),
                SymbolType::Include => {
                    let include_path = std::path::Path::new(&symbol.name);

                    // Try to resolve the include path
                    let resolved = if include_path.is_absolute() {
                        Some(include_path.to_path_buf())
                    } else {
                        // Try to resolve relative to the current file's directory
                        uri.to_file_path().ok().and_then(|current_file| {
                            let current_dir = current_file.parent()?;
                            let candidate = current_dir.join(&symbol.name);
                            if candidate.exists() {
                                return Some(candidate);
                            }

                            // Try looking in common include directories relative to current file
                            for include_dir in &["include", "../include", "../../include"] {
                                let candidate = current_dir.join(include_dir).join(&symbol.name);
                                if candidate.exists() {
                                    return Some(candidate);
                                }
                            }
                            None
                        })
                    };

                    // Format the path for display (relative to workspace root if possible)
                    let display_path = if let Some(resolved_path) = resolved {
                        // Canonicalize to resolve .. and . components
                        let canonical = resolved_path.canonicalize().unwrap_or(resolved_path);

                        // Try to make it relative to workspace root
                        if let Ok(current_file) = uri.to_file_path() {
                            if let Some(current_dir) = current_file.parent() {
                                // Find workspace root by looking for .git or .sv-lsp.toml
                                let mut workspace_root = current_dir;
                                while let Some(parent) = workspace_root.parent() {
                                    if parent.join(".git").exists()
                                        || parent.join(".sv-lsp.toml").exists()
                                    {
                                        workspace_root = parent;
                                        break;
                                    }
                                    workspace_root = parent;
                                }

                                // Make path relative to workspace root
                                if let Ok(rel) = canonical.strip_prefix(workspace_root) {
                                    rel.display().to_string()
                                } else {
                                    canonical.display().to_string()
                                }
                            } else {
                                canonical.display().to_string()
                            }
                        } else {
                            canonical.display().to_string()
                        }
                    } else {
                        // Couldn't resolve, just show the original path
                        symbol.name.clone()
                    };

                    format!("```systemverilog\n`include \"{}\"\n```", display_path)
                }
            };

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: Some(symbol.range),
            }));
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let docs = self.documents.read().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        // Convert our symbols to LSP DocumentSymbol format
        for symbol in &doc_state.symbols {
            let kind = match symbol.symbol_type {
                SymbolType::Module => SymbolKind::MODULE,
                SymbolType::Class => SymbolKind::CLASS,
                SymbolType::Function => SymbolKind::FUNCTION,
                SymbolType::Task => SymbolKind::FUNCTION,
                SymbolType::Variable => SymbolKind::VARIABLE,
                SymbolType::Port => SymbolKind::PROPERTY,
                SymbolType::Parameter => SymbolKind::CONSTANT,
                SymbolType::Define => SymbolKind::CONSTANT,
                SymbolType::Include => SymbolKind::FILE,
            };

            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: symbol.name.clone(),
                detail: None,
                kind,
                tags: None,
                deprecated: None,
                range: symbol.range,
                selection_range: symbol.range,
                children: None,
            });
        }

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> LspResult<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        // Find the symbol at the cursor position
        let symbol_at_position = doc_state
            .symbols
            .iter()
            .find(|symbol| self.position_in_range(position, symbol.range));

        if let Some(symbol) = symbol_at_position {
            let mut highlights = Vec::new();

            // Find all occurrences of this symbol in the current document
            for other_symbol in &doc_state.symbols {
                if other_symbol.name == symbol.name {
                    highlights.push(DocumentHighlight {
                        range: other_symbol.range,
                        kind: Some(DocumentHighlightKind::TEXT),
                    });
                }
            }

            if highlights.is_empty() {
                Ok(None)
            } else {
                Ok(Some(highlights))
            }
        } else {
            Ok(None)
        }
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> LspResult<Option<Vec<SelectionRange>>> {
        let uri = params.text_document.uri;
        let positions = params.positions;

        let docs = self.documents.read().await;
        let doc_state = match docs.get(&uri) {
            Some(state) => state,
            None => return Ok(None),
        };

        if doc_state.ast.is_none() {
            return Ok(None);
        }

        let ast = doc_state.ast.as_ref().unwrap();
        let content = &doc_state.content;

        let mut results = Vec::new();

        for position in positions {
            if let Some(selection) = self.find_selection_range_at_position(ast, content, position) {
                results.push(selection);
            }
        }

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
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

            // Add new symbols (skip Include symbols as they're file-specific)
            for symbol in symbols {
                if !matches!(symbol.symbol_type, SymbolType::Include) {
                    workspace_symbols
                        .entry(symbol.name.clone())
                        .or_insert_with(Vec::new)
                        .push(symbol);
                }
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

        // Use parse_content_recovery to get partial AST even with errors
        let result = parser.parse_content_recovery(text);

        // Extract symbols from AST (even if partial)
        if let Some(parsed_ast) = &result.ast {
            symbols = self.extract_symbols_from_ast(parsed_ast, text, uri);
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Extracted {} symbols from AST", symbols.len()),
                )
                .await;
            ast = Some(parsed_ast.clone());
        }

        // Process errors as diagnostics
        if !result.errors.is_empty() {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Parse completed with {} errors", result.errors.len()),
                )
                .await;

            for error in &result.errors {
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
                        let end_pos =
                            self.char_offset_to_position(text, end_char)
                                .unwrap_or_else(|| {
                                    Position::new(location.line as u32, location.column as u32 + 1)
                                });

                        Range::new(start_pos, end_pos)
                    } else {
                        // Use line/column directly
                        let start_pos = Position::new(location.line as u32, location.column as u32);
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
                    sv_parser::ParseErrorType::UnsupportedFeature(_) => DiagnosticSeverity::WARNING,
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

    // Helper function to convert a span to LSP Range
    fn span_to_range(&self, text: &str, span: sv_parser::Span) -> Option<Range> {
        let start_pos = self.char_offset_to_position(text, span.0)?;
        let end_pos = self.char_offset_to_position(text, span.1)?;
        Some(Range::new(start_pos, end_pos))
    }

    // Find hover information at a specific position
    fn find_hover_at_position(
        &self,
        items: &[ModuleItem],
        content: &str,
        position: Position,
    ) -> Option<Hover> {
        for item in items {
            if let Some(hover) = self.find_hover_in_item(item, content, position) {
                return Some(hover);
            }
        }
        None
    }

    // Recursively search for hover information in a module item
    fn find_hover_in_item(
        &self,
        item: &ModuleItem,
        content: &str,
        position: Position,
    ) -> Option<Hover> {
        match item {
            ModuleItem::ModuleDeclaration {
                name,
                name_span,
                items,
                ..
            } => {
                // Check if hovering over module name
                if let Some(range) = self.span_to_range(content, *name_span) {
                    if self.position_in_range(position, range) {
                        return Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("```systemverilog\nmodule {}\n```", name),
                            }),
                            range: Some(range),
                        });
                    }
                }

                // Recursively search in module items
                for sub_item in items {
                    if let Some(hover) = self.find_hover_in_item(sub_item, content, position) {
                        return Some(hover);
                    }
                }
            }
            ModuleItem::ProceduralBlock { statements, .. } => {
                // Check for system function calls in statements
                for stmt in statements {
                    if let Some(hover) = self.find_hover_in_statement(stmt, content, position) {
                        return Some(hover);
                    }
                }
            }
            _ => {}
        }
        None
    }

    // Find hover information in a statement
    fn find_hover_in_statement(
        &self,
        stmt: &sv_parser::Statement,
        content: &str,
        position: Position,
    ) -> Option<Hover> {
        match stmt {
            sv_parser::Statement::SystemCall { name, span, args } => {
                // First, check if we're hovering over any nested system function calls in the arguments
                for arg in args {
                    if let Some(hover) = self.find_hover_in_expression(arg, content, position) {
                        return Some(hover);
                    }
                }

                // If not hovering over arguments, check if hovering over the system call name itself
                if let Some(range) = self.span_to_range(content, *span) {
                    if self.position_in_range(position, range) {
                        if let Some(info) = get_system_function_info(name) {
                            return Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```systemverilog\n{}\n```\n\n{}",
                                        info.signature, info.description
                                    ),
                                }),
                                range: Some(range),
                            });
                        }
                    }
                }
            }
            sv_parser::Statement::Assignment { expr, .. } => {
                // Check if there's a system function call in the expression
                if let Some(hover) = self.find_hover_in_expression(expr, content, position) {
                    return Some(hover);
                }
            }
        }
        None
    }

    // Find hover information in an expression
    fn find_hover_in_expression(
        &self,
        expr: &Expression,
        content: &str,
        position: Position,
    ) -> Option<Hover> {
        match expr {
            Expression::SystemFunctionCall {
                name,
                span,
                arguments,
                ..
            } => {
                if let Some(range) = self.span_to_range(content, *span) {
                    if self.position_in_range(position, range) {
                        if let Some(info) = get_system_function_info(name) {
                            return Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```systemverilog\n{}\n```\n\n{}",
                                        info.signature, info.description
                                    ),
                                }),
                                range: Some(range),
                            });
                        }
                    }
                }
                // Also check arguments
                for arg in arguments {
                    if let Some(hover) = self.find_hover_in_expression(arg, content, position) {
                        return Some(hover);
                    }
                }
            }
            Expression::Binary { left, right, .. } => {
                if let Some(hover) = self.find_hover_in_expression(left, content, position) {
                    return Some(hover);
                }
                if let Some(hover) = self.find_hover_in_expression(right, content, position) {
                    return Some(hover);
                }
            }
            Expression::Unary { operand, .. } => {
                if let Some(hover) = self.find_hover_in_expression(operand, content, position) {
                    return Some(hover);
                }
            }
            Expression::MacroUsage { arguments, .. } => {
                for arg in arguments {
                    if let Some(hover) = self.find_hover_in_expression(arg, content, position) {
                        return Some(hover);
                    }
                }
            }
            Expression::MemberAccess { object, .. } => {
                // Check hover in the object expression
                if let Some(hover) = self.find_hover_in_expression(object, content, position) {
                    return Some(hover);
                }
            }
            _ => {}
        }
        None
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
            ModuleItem::ModuleDeclaration {
                name,
                name_span,
                ports,
                items,
                ..
            } => {
                // Add module name as a symbol using span from AST
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Module,
                        range,
                        uri: uri.clone(),
                    });
                }

                // Add port names as symbols
                for port in ports {
                    if let Some(range) = self.span_to_range(content, port.name_span) {
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
            ModuleItem::PortDeclaration {
                name, name_span, ..
            } => {
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Port,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            ModuleItem::VariableDeclaration {
                name,
                name_span,
                initial_value,
                ..
            } => {
                // Add variable declaration using span from AST
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }

                // Extract identifiers from initial value expression if present
                if let Some(expr) = initial_value {
                    self.extract_symbols_from_expression(expr, content, uri, symbols);
                }
            }
            ModuleItem::Assignment {
                target,
                target_span,
                expr,
                ..
            } => {
                // Add assignment target as variable using span from AST
                if let Some(range) = self.span_to_range(content, *target_span) {
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
            ModuleItem::ProceduralBlock { statements, .. } => {
                // Extract symbols from statements in procedural block
                for statement in statements {
                    self.extract_symbols_from_statement(statement, content, uri, symbols);
                }
            }
            ModuleItem::DefineDirective {
                name, name_span, ..
            } => {
                // Add define directive as a symbol
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Define,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            ModuleItem::IncludeDirective {
                path,
                path_span,
                resolved_path,
                ..
            } => {
                // Add include directive as a symbol
                if let Some(range) = self.span_to_range(content, *path_span) {
                    let symbol_name = resolved_path
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .unwrap_or(path)
                        .to_string();

                    symbols.push(Symbol {
                        name: symbol_name,
                        symbol_type: SymbolType::Include,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            ModuleItem::ClassDeclaration {
                name,
                name_span,
                items,
                ..
            } => {
                // Add class declaration as a symbol
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Class,
                        range,
                        uri: uri.clone(),
                    });
                }
                // Extract class members (properties and methods) as symbols
                for class_item in items {
                    self.extract_symbols_from_class_item(class_item, content, uri, symbols);
                }
            }
        }
    }

    // Extract symbols from statements
    fn extract_symbols_from_statement(
        &self,
        statement: &sv_parser::Statement,
        content: &str,
        uri: &Url,
        symbols: &mut Vec<Symbol>,
    ) {
        use sv_parser::Statement;
        match statement {
            Statement::Assignment {
                target,
                target_span,
                expr,
                ..
            } => {
                if let Some(range) = self.span_to_range(content, *target_span) {
                    symbols.push(Symbol {
                        name: target.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }
                self.extract_symbols_from_expression(expr, content, uri, symbols);
            }
            Statement::SystemCall { args, .. } => {
                for arg in args {
                    self.extract_symbols_from_expression(arg, content, uri, symbols);
                }
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
            Expression::Identifier(name, span) => {
                if let Some(range) = self.span_to_range(content, *span) {
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
            Expression::MacroUsage {
                name,
                name_span,
                arguments,
                ..
            } => {
                // Add macro usage as a symbol reference
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Define,
                        range,
                        uri: uri.clone(),
                    });
                }
                // Extract symbols from macro arguments
                for arg in arguments {
                    self.extract_symbols_from_expression(arg, content, uri, symbols);
                }
            }
            Expression::SystemFunctionCall { arguments, .. } => {
                // Extract symbols from system function call arguments
                for arg in arguments {
                    self.extract_symbols_from_expression(arg, content, uri, symbols);
                }
            }
            Expression::New { arguments, .. } => {
                // Extract symbols from new expression arguments
                for arg in arguments {
                    self.extract_symbols_from_expression(arg, content, uri, symbols);
                }
            }
            Expression::MemberAccess {
                object,
                member,
                member_span,
                ..
            } => {
                // Extract the object expression
                self.extract_symbols_from_expression(object, content, uri, symbols);
                // Extract the member as a symbol
                if let Some(range) = self.span_to_range(content, *member_span) {
                    symbols.push(Symbol {
                        name: member.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }
            }
            Expression::Number(_, _) | Expression::StringLiteral(_, _) => {
                // Numbers and string literals are not identifiers we care about for renaming
            }
        }
    }

    // Extract symbols from class items (properties and methods)
    fn extract_symbols_from_class_item(
        &self,
        class_item: &sv_parser::ClassItem,
        content: &str,
        uri: &Url,
        symbols: &mut Vec<Symbol>,
    ) {
        use sv_parser::ClassItem;
        match class_item {
            ClassItem::Property {
                name,
                name_span,
                initial_value,
                ..
            } => {
                // Add property as a symbol
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Variable,
                        range,
                        uri: uri.clone(),
                    });
                }
                // Extract symbols from initial value if present
                if let Some(expr) = initial_value {
                    self.extract_symbols_from_expression(expr, content, uri, symbols);
                }
            }
            ClassItem::Method {
                name,
                name_span,
                body,
                ..
            } => {
                // Add method as a symbol
                if let Some(range) = self.span_to_range(content, *name_span) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        symbol_type: SymbolType::Function,
                        range,
                        uri: uri.clone(),
                    });
                }
                // Extract symbols from method body statements
                for statement in body {
                    self.extract_symbols_from_statement(statement, content, uri, symbols);
                }
            }
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
            ModuleItem::ModuleDeclaration {
                name, items, span, ..
            } => {
                // Create folding range from span
                if let Some(range) = self.span_to_folding_range(content, *span) {
                    ranges.push(FoldingRange {
                        collapsed_text: Some(format!("module {} ...", name)),
                        kind: Some(FoldingRangeKind::Region),
                        ..range
                    });
                }

                // Recursively process nested items
                for sub_item in items {
                    self.extract_folding_ranges_from_item(sub_item, content, ranges);
                }
            }
            ModuleItem::ProceduralBlock { span, .. } => {
                // Add folding range for procedural blocks (always, initial, etc.)
                if let Some(range) = self.span_to_folding_range(content, *span) {
                    ranges.push(FoldingRange {
                        kind: Some(FoldingRangeKind::Region),
                        ..range
                    });
                }
            }
            ModuleItem::ClassDeclaration {
                name, items, span, ..
            } => {
                // Add folding range for class
                if let Some(range) = self.span_to_folding_range(content, *span) {
                    ranges.push(FoldingRange {
                        collapsed_text: Some(format!("class {} ...", name)),
                        kind: Some(FoldingRangeKind::Region),
                        ..range
                    });
                }

                // Also add folding ranges for class methods
                for class_item in items {
                    if let sv_parser::ClassItem::Method { name, span, .. } = class_item {
                        if let Some(range) = self.span_to_folding_range(content, *span) {
                            ranges.push(FoldingRange {
                                collapsed_text: Some(format!("function {} ...", name)),
                                kind: Some(FoldingRangeKind::Region),
                                ..range
                            });
                        }
                    }
                }
            }
            ModuleItem::VariableDeclaration { .. }
            | ModuleItem::Assignment { .. }
            | ModuleItem::PortDeclaration { .. }
            | ModuleItem::DefineDirective { .. }
            | ModuleItem::IncludeDirective { .. } => {
                // These items typically don't need folding
            }
        }
    }

    fn span_to_folding_range(&self, content: &str, span: (usize, usize)) -> Option<FoldingRange> {
        // Convert byte offsets to line numbers
        let start_line = content[..span.0].matches('\n').count();
        let end_line = content[..span.1].matches('\n').count();

        // Create folding range if it spans multiple lines (at least 2)
        // Some editors require at least 1 line of difference to show fold indicators
        if end_line >= start_line + 1 {
            Some(FoldingRange {
                start_line: start_line as u32,
                start_character: None,
                end_line: end_line as u32,
                end_character: None,
                kind: None,           // Will be set by caller
                collapsed_text: None, // Will be set by caller
            })
        } else {
            None
        }
    }

    fn find_selection_range_at_position(
        &self,
        ast: &SourceUnit,
        content: &str,
        position: Position,
    ) -> Option<SelectionRange> {
        // Build a hierarchy of selection ranges from the AST
        let mut ranges: Vec<(usize, usize)> = Vec::new();

        // Find all AST nodes that contain the position
        for item in &ast.items {
            self.collect_ranges_containing_position(item, content, position, &mut ranges);
        }

        if ranges.is_empty() {
            return None;
        }

        // Sort ranges by size (smallest first)
        ranges.sort_by_key(|(start, end)| end - start);

        // Build the selection range hierarchy (parent contains child)
        let mut selection_range: Option<SelectionRange> = None;

        for span in ranges {
            if let Some(range) = self.span_to_range(content, span) {
                selection_range = Some(SelectionRange {
                    range,
                    parent: selection_range.map(Box::new),
                });
            }
        }

        selection_range
    }

    fn collect_ranges_containing_position(
        &self,
        item: &ModuleItem,
        content: &str,
        position: Position,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        // Helper to check if a span contains the position
        let contains = |span: (usize, usize)| -> bool {
            if let Some(range) = self.span_to_range(content, span) {
                self.position_in_range(position, range)
            } else {
                false
            }
        };

        match item {
            ModuleItem::ModuleDeclaration {
                span,
                items,
                name_span,
                ..
            } => {
                if contains(*span) {
                    ranges.push(*span);
                    if contains(*name_span) {
                        ranges.push(*name_span);
                    }
                    // Recursively check nested items
                    for sub_item in items {
                        self.collect_ranges_containing_position(
                            sub_item, content, position, ranges,
                        );
                    }
                }
            }
            ModuleItem::ClassDeclaration {
                span,
                name_span,
                items: class_items,
                ..
            } => {
                if contains(*span) {
                    ranges.push(*span);
                    if contains(*name_span) {
                        ranges.push(*name_span);
                    }
                    // Check class methods
                    for class_item in class_items {
                        if let sv_parser::ClassItem::Method {
                            span, name_span, ..
                        } = class_item
                        {
                            if contains(*span) {
                                ranges.push(*span);
                            }
                            if contains(*name_span) {
                                ranges.push(*name_span);
                            }
                        } else if let sv_parser::ClassItem::Property {
                            span, name_span, ..
                        } = class_item
                        {
                            if contains(*span) {
                                ranges.push(*span);
                            }
                            if contains(*name_span) {
                                ranges.push(*name_span);
                            }
                        }
                    }
                }
            }
            ModuleItem::ProceduralBlock { span, .. } => {
                if contains(*span) {
                    ranges.push(*span);
                }
            }
            ModuleItem::VariableDeclaration {
                span, name_span, ..
            }
            | ModuleItem::Assignment {
                span,
                target_span: name_span,
                ..
            }
            | ModuleItem::PortDeclaration {
                span, name_span, ..
            } => {
                if contains(*span) {
                    ranges.push(*span);
                }
                if contains(*name_span) {
                    ranges.push(*name_span);
                }
            }
            ModuleItem::DefineDirective {
                span, name_span, ..
            }
            | ModuleItem::IncludeDirective {
                span,
                path_span: name_span,
                ..
            } => {
                if contains(*span) {
                    ranges.push(*span);
                }
                if contains(*name_span) {
                    ranges.push(*name_span);
                }
            }
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
