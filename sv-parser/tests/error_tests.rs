//! Error handling tests built atop the shared harness utilities.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_fails, assert_parse_err};
use sv_parser::SystemVerilogParser;

/// Error fixtures in `test_files/errors/` should all fail.
#[test]
fn test_parse_all_error_files() {
    assert_directory_fails("errors");
}

sv_err_tests! {
    invalid_syntax_fixture => "errors/invalid_syntax.sv",
    incomplete_module_fixture => "errors/incomplete_module.sv",
}

#[test]
fn test_invalid_syntax_error() {
    assert_parse_err("errors/invalid_syntax.sv");
}

#[test]
fn test_incomplete_module_error() {
    assert_parse_err("errors/incomplete_module.sv");
}

#[test]
fn test_error_span_positions() {
    let parser = SystemVerilogParser::new(vec![], Default::default());

    // Missing semicolon should surface an error on the second line with column info.
    let content = "module test;\n    int foo\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            assert!(
                err.errors.iter().any(|e| e.location.is_some()),
                "Expected at least one error with location info"
            );
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}

#[test]
fn test_error_span_coverage() {
    let parser = SystemVerilogParser::new(vec![], Default::default());

    let content = "module test;\n    assign x = (1 + 2;\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            assert!(err.errors.iter().any(|e| {
                e.location
                    .as_ref()
                    .map(|loc| loc.line == 1 && loc.column > 0)
                    .unwrap_or(false)
            }));
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}

#[test]
fn test_multiple_errors_different_columns() {
    let parser = SystemVerilogParser::new(vec![], Default::default());

    let content = "module test;\n    int foo\n        int bar\nendmodule\n";

    match parser.parse_content(content) {
        Err(err) => {
            let columns: Vec<usize> = err
                .errors
                .iter()
                .filter_map(|e| e.location.as_ref())
                .map(|loc| loc.column)
                .collect();
            assert!(!columns.is_empty());
            assert!(columns.iter().all(|&col| col > 0));
        }
        Ok(_) => panic!("Expected parse to fail"),
    }
}
