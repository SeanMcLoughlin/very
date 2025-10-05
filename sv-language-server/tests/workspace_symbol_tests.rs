pub mod common;

use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn test_workspace_symbol_search() {
    let backend = common::create_test_backend();

    // Open multiple documents
    let uri1 = common::test_uri("/test/file1.sv");
    let content1 = r#"module test_module;
    logic test_signal;
endmodule"#;

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri1.clone(),
                language_id: "systemverilog".to_string(),
                version: 1,
                text: content1.to_string(),
            },
        })
        .await;

    let uri2 = common::test_uri("/test/file2.sv");
    let content2 = r#"module another_module;
    logic another_signal;
endmodule"#;

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri2.clone(),
                language_id: "systemverilog".to_string(),
                version: 1,
                text: content2.to_string(),
            },
        })
        .await;

    // Search for "test"
    let result = backend
        .symbol(WorkspaceSymbolParams {
            query: "test".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let symbols = result.unwrap();

    if let Some(syms) = symbols {
        // Should find symbols containing "test"
        assert!(!syms.is_empty(), "Should find symbols with 'test'");
        let has_test_module = syms.iter().any(|s| s.name.contains("test_module"));
        assert!(has_test_module, "Should find test_module");
    }
}

#[tokio::test]
async fn test_workspace_symbol_case_insensitive() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/file.sv");

    let content = r#"module TestModule;
endmodule"#;

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "systemverilog".to_string(),
                version: 1,
                text: content.to_string(),
            },
        })
        .await;

    // Search with lowercase
    let result = backend
        .symbol(WorkspaceSymbolParams {
            query: "testmodule".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let symbols = result.unwrap();

    if let Some(syms) = symbols {
        let has_module = syms
            .iter()
            .any(|s| s.name.to_lowercase().contains("testmodule"));
        assert!(has_module, "Search should be case-insensitive");
    }
}

#[tokio::test]
async fn test_workspace_symbol_empty_query() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/file.sv");

    let content = r#"module test;
endmodule"#;

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "systemverilog".to_string(),
                version: 1,
                text: content.to_string(),
            },
        })
        .await;

    // Empty query should match all symbols
    let result = backend
        .symbol(WorkspaceSymbolParams {
            query: "".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok());
    // Empty query matches everything, so we should get results
    let symbols = result.unwrap();
    if let Some(syms) = symbols {
        assert!(!syms.is_empty(), "Empty query should match all symbols");
    }
}
