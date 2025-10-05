pub mod common;

use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

#[tokio::test]
/// Test that there is markup content when hovering the mouse over the system function $display()
async fn test_hover_system_task_display() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_display.sv");

    let content = r#"module test;
    initial begin
        $display("Hello");
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

    let result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 9), // row[2] col[9] is the 'd' in $display,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let hover = result.unwrap();
    assert!(
        hover.is_some(),
        "Hover should return information for $display"
    );

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = hover.contents {
        assert!(
            content.value.contains("display"),
            "Hover should contain information about $display, got: {}",
            content.value
        );
        assert!(
            content.value.contains("task"),
            "Hover should indicate $display is a task, got: {}",
            content.value
        );
    } else {
        panic!("Hover should return markup content");
    }
}

#[tokio::test]
/// Test that there is markup content when hovering the mouse over system functions that call each
/// other.
async fn test_hover_nested_system_functions_and_tasks() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_display.sv");

    let content = r#"module test;
    initial begin
        $display($acos(4));
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

    // Display
    let display_result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 9), // row[2] col[9] is the 'd' in $display,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(display_result.is_ok());
    let display_hover = display_result.unwrap();
    assert!(
        display_hover.is_some(),
        "Hover should return information for $display"
    );

    let display_hover = display_hover.unwrap();
    if let HoverContents::Markup(display_content) = display_hover.contents {
        assert!(
            display_content.value.contains("display"),
            "Hover should contain information about $display, got: {}",
            display_content.value
        );
        assert!(
            display_content.value.contains("task"),
            "Hover should indicate $display is a task, got: {}",
            display_content.value
        );
    } else {
        panic!("Hover should return markup content");
    }

    // acos
    let acos_result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 18), // row[2] col[18] is the 'a' in $acos,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(acos_result.is_ok());
    let acos_hover = acos_result.unwrap();
    assert!(
        acos_hover.is_some(),
        "Hover should return information for $display"
    );

    let acos_hover = acos_hover.unwrap();
    if let HoverContents::Markup(acos_content) = acos_hover.contents {
        assert!(
            acos_content.value.contains("arc cosine"),
            "Hover should contain information about $acos, got: {}",
            acos_content.value
        );
        assert!(
            acos_content.value.contains("function"),
            "Hover should indicate $acos is a task, got: {}",
            acos_content.value
        );
    } else {
        panic!("Hover should return markup content");
    }
}

#[tokio::test]
/// Test that there is markup content when hovering the mouse over the system function $sin()
async fn test_hover_system_function_tan() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_tan.sv");

    let content = r#"module test;
    initial begin
        logic a = $tan(1);
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

    let result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 19), // row[2] col[19] is the 't' in $tan
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let hover = result.unwrap();
    assert!(hover.is_some(), "Hover should return information for $tan");

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = hover.contents {
        assert!(
            content.value.contains("tan"),
            "Hover should contain information about $tan, got: {}",
            content.value
        );
        assert!(
            content.value.contains("function") || content.value.contains("tangent"),
            "Hover should indicate $tan is a function or mention tangent, got: {}",
            content.value
        );
    } else {
        panic!("Hover should return markup content");
    }
}

#[tokio::test]
/// Test that there is markup content when hovering the mouse over the system function $cos outside
/// of an initial block
async fn test_hover_system_function_cos_in_assign_statement() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_tan.sv");

    let content = r#"module test;
    logic a;
    assign a = $cos(4);
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
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(2, 16), // row[2] col[16] is the 'c' in $cos
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let hover = result.unwrap();
    assert!(hover.is_some(), "Hover should return information for $cos");

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = hover.contents {
        assert!(
            content.value.contains("cos"),
            "Hover should contain information about $cos, got: {}",
            content.value
        );
        assert!(
            content.value.contains("function") || content.value.contains("cosine"),
            "Hover should indicate $cos is a function or mention cosine, got: {}",
            content.value
        );
    } else {
        panic!("Hover should return markup content");
    }
}

#[tokio::test]
/// Test that there is markup content when hovering the mouse over the system function $sin()
async fn test_hover_system_function_sin() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_sin.sv");

    let content = r#"module test;
  // Indents are purposefully 2 spaces.
  initial begin
    logic b = $sin(1);
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

    let result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(3, 15), // row[3] col[15] is the 's' in $sin
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
    let hover = result.unwrap();
    assert!(hover.is_some(), "Hover should return information for $sin");

    let hover = hover.unwrap();
    if let HoverContents::Markup(content) = hover.contents {
        assert!(
            content.value.contains("sin"),
            "Hover should contain information about $sin, got: {}",
            content.value
        );
        assert!(
            content.value.contains("function") || content.value.contains("sine"),
            "Hover should indicate $sin is a function or mention sine, got: {}",
            content.value
        );
    } else {
        panic!("Hover should return markup content");
    }
}

#[tokio::test]
/// Test that there is markup content when hovering the mouse over ```include "file.svh"``
async fn test_hover_include_file() {
    let backend = common::create_test_backend();
    let uri = common::test_uri("/test/hover_include.sv");

    let content = r#"`include "test.svh"
module test;
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

    // Hover over the include file path
    let result = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: common::test_position(0, 11), // Inside the filename
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
}
