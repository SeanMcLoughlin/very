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
            println!("Testing error file: {}", filename);

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            match parser.parse_content(&content) {
                Ok(_) => {
                    panic!("Expected {} to fail parsing, but it succeeded", filename);
                }
                Err(_) => {
                    println!("  ✅ Failed as expected");
                }
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
