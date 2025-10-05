mod common;

use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn test_completion_system_functions() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/completion.sv");

    let content = r#"module test;
    initial begin
        $dis
    end
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

    // Try to get completions after "$dis"
    let result = backend
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 12), // After "$dis"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: Some("$".to_string()),
            }),
        })
        .await;

    assert!(result.is_ok());
    if let Ok(Some(CompletionResponse::Array(items))) = result {
        // $display needs to be in completions.
        let has_display = items.iter().any(|item| item.label.contains("display"));
        assert!(
            has_display,
            "display not found in completions of system functions starting with '$dis'"
        );
    }
}

#[tokio::test]
async fn test_completion_keywords() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/keywords.sv");
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

    // Get completions in the module body
    let result = backend
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(1, 4),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await;

    assert!(result.is_ok());

    if let Ok(Some(CompletionResponse::Array(items))) = result {
        assert!(
            !items.is_empty(),
            "Should return keyword completions in module body"
        );

        // Should include common SystemVerilog keywords like "logic", "wire", "always", etc.
        let keyword_labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();

        let has_sv_keywords = keyword_labels.iter().any(|label| {
            [
                "logic", "wire", "reg", "always", "assign", "initial", "begin",
            ]
            .contains(label)
        });

        assert!(
            has_sv_keywords,
            "Completions should include SystemVerilog keywords, got: {:?}",
            keyword_labels
        );
    } else {
        panic!("Should return keyword completions");
    }
}
