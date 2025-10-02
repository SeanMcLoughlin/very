//! Drive strength tests
//!
//! This module tests drive strength parsing by running the parser against
//! SystemVerilog files in the test_files/drive_strengths/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test parsing all drive strength test files
#[test]
fn test_parse_all_drive_strength_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/drive_strengths");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));
        }
    }
}

/// Test strong1 highz0 assignment
#[test]
fn test_drive_strength_strong1_highz0() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/drive_strengths/10.3.4--assignment_strong1_highz0.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test pull1 pull0 assignment
#[test]
fn test_drive_strength_pull1_pull0() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/drive_strengths/10.3.4--assignment_pull1_pull0.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test weak1 weak0 assignment
#[test]
fn test_drive_strength_weak1_weak0() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/drive_strengths/10.3.4--assignment_weak1_weak0.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}
