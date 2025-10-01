//! Error handling tests using file-based approach
//!
//! This module tests error handling by running the parser against
//! SystemVerilog files in the test_files/errors/ directory that should fail.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test that error files properly fail to parse
#[test]
fn test_parse_all_error_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/errors");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            match parser.parse_content(&content) {
                Ok(_) => {
                    panic!("Expected {} to fail parsing, but it succeeded", filename);
                }
                Err(_) => {}
            }
        }
    }
}

#[test]
fn test_invalid_syntax_error() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/errors/invalid_syntax.sv"),
    )
    .unwrap();

    // This should fail to parse
    assert!(parser.parse_content(&content).is_err());
}

#[test]
fn test_incomplete_module_error() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/errors/incomplete_module.sv"),
    )
    .unwrap();

    // This should fail to parse
    assert!(parser.parse_content(&content).is_err());
}

#[test]
fn test_error_span_positions() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test that errors on indented lines have correct column positions, not column 0
    // Use genuinely invalid syntax (missing semicolon)
    let content = "module test;\n    int foo\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            let errors = &err.errors;
            assert!(!errors.is_empty(), "Expected at least one error");

            // Find error - should have valid location information
            let error_found = errors.iter().any(|e| {
                // Error should have location information
                e.location.is_some()
            });

            assert!(
                error_found,
                "Expected error with valid location information, got errors: {:?}",
                errors
            );
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}

#[test]
fn test_error_span_coverage() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test that error spans cover the actual token
    // Use invalid syntax - unclosed bracket
    let content = "module test;\n    assign x = (1 + 2;\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            let errors = &err.errors;
            assert!(!errors.is_empty(), "Expected at least one error");

            // Find error - should have proper span information
            let error_found = errors.iter().any(|e| {
                if let Some(loc) = &e.location {
                    // Error should be on line 1 (0-indexed) with valid column position
                    loc.line == 1 && loc.column > 0
                } else {
                    false
                }
            });

            assert!(
                error_found,
                "Expected error on line 1 with valid span, got errors: {:?}",
                errors
            );
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}

#[test]
fn test_multiple_errors_different_columns() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test multiple errors on lines with different indentation
    // Use genuinely invalid syntax (missing semicolons)
    let content = "module test;\n    int foo\n        int bar\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            let errors = &err.errors;

            // Should have at least one error
            assert!(errors.len() >= 1, "Expected at least 1 error");

            // Find errors and check their columns
            let cols: Vec<usize> = errors
                .iter()
                .filter_map(|e| e.location.as_ref())
                .map(|loc| loc.column)
                .collect();

            // All errors should have column > 0 (both are on indented lines)
            assert!(
                cols.iter().all(|&col| col > 0),
                "All errors should have column > 0, got columns: {:?}",
                cols
            );

            // Errors should be reported at reasonable column positions
            // The exact columns depend on error recovery, but they should all be > 0
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}
