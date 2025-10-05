use std::sync::Arc;
use sv_language_server::Backend;
use tower_lsp::lsp_types::*;
use tower_lsp::LspService;

/// Create a test backend for direct testing
/// Returns an Arc-wrapped backend so it can be shared across tests
pub fn create_test_backend() -> Arc<Backend> {
    // Create a service to get a valid Client
    let (service, _socket) = LspService::new(|client| sv_language_server::create_backend(client));

    // Get a reference to the inner backend and wrap it in Arc
    // We need to leak it to get a 'static reference, then wrap in Arc
    let backend_ref = service.inner() as *const Backend;

    // This is unsafe but necessary for testing
    // In production, the service owns the backend
    unsafe {
        // Clone the Backend's Arc fields to create a new Backend
        let original = &*backend_ref;

        Arc::new(Backend {
            client: original.client.clone(),
            documents: Arc::clone(&original.documents),
            workspace_symbols: Arc::clone(&original.workspace_symbols),
            config: Arc::clone(&original.config),
            workspace_root: Arc::clone(&original.workspace_root),
        })
    }
}

/// Helper to create a test URI
pub fn test_uri(path: &str) -> Url {
    Url::parse(&format!("file://{}", path)).unwrap()
}

/// Helper to create a test position (zero-based row and column)
pub fn test_position(row: u32, col: u32) -> Position {
    Position {
        line: row,
        character: col,
    }
}
