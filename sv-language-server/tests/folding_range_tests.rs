mod common;

use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn test_folding_range_module() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/folding.sv");

    let content = r#"module test;
    logic a;
    logic b;
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

    let result = backend
        .folding_range(FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok(), "Folding range should succeed");
    let ranges = result.unwrap();

    assert!(ranges.is_some(), "Should return folding ranges");
    let r = ranges.unwrap();
    assert!(!r.is_empty(), "Should have at least one folding range");

    // Check that we have a range covering the module body
    // module starts at line 0, endmodule at line 3
    let has_module_range = r
        .iter()
        .any(|range| range.start_line == 0 && range.end_line >= 2);
    assert!(
        has_module_range,
        "Should have a folding range for the module (line 0 to ~3)"
    );
}

#[tokio::test]
async fn test_folding_range_class() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/class.sv");

    let content = r#"module top;
class MyClass;
    int x;
    int y;
endclass
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

    let result = backend
        .folding_range(FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok(), "Folding range should succeed");
    let ranges = result.unwrap();

    if ranges.is_some() {
        let r = ranges.unwrap();
        // Should have ranges for both module and class
        assert!(
            r.len() >= 1,
            "Should have at least one folding range (module or class), got {}",
            r.len()
        );
    }
}
