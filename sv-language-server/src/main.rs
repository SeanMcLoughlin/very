use anyhow::Result;
use std::collections::HashMap;
use sv_parser::SystemVerilogParser;
use tokio::io::{stdin, stdout};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
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
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
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

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
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
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let diagnostics = self.validate_document(&params.text).await;

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;
    }

    async fn validate_document(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Create parser and try to parse the document
        let parser = SystemVerilogParser::new(vec![], HashMap::new());

        match parser.parse_content(text) {
            Ok(_ast) => {
                // Parse succeeded - no diagnostics
                self.client
                    .log_message(MessageType::INFO, "Parse succeeded!")
                    .await;
            }
            Err(parse_error) => {
                // Parse failed - convert each error to a diagnostic
                self.client
                    .log_message(MessageType::INFO, format!("Parse failed: {}", parse_error))
                    .await;

                for error in &parse_error.errors {
                    let range = if let Some(location) = &error.location {
                        // Use actual error location
                        let start_pos = Position::new(location.line as u32, location.column as u32);
                        let end_pos = if let Some((_start_char, end_char)) = location.span {
                            // Try to find end position from span
                            self.char_offset_to_position(text, end_char)
                                .unwrap_or(start_pos)
                        } else {
                            // Default to single character
                            Position::new(location.line as u32, location.column as u32 + 1)
                        };
                        Range::new(start_pos, end_pos)
                    } else {
                        // Fallback to start of document
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

        diagnostics
    }

    // Helper function to convert character offset to LSP Position
    fn char_offset_to_position(&self, text: &str, offset: usize) -> Option<Position> {
        if offset > text.len() {
            return None;
        }

        let prefix = &text[..offset];
        let line = prefix.matches('\n').count();
        let column = prefix.split('\n').last().unwrap_or("").len();

        Some(Position::new(line as u32, column as u32))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
