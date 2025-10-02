//! Procedural block tests using file-based approach
//!
//! This module tests procedural block parsing by running the parser against
//! SystemVerilog files in the test_files/procedural_blocks/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test parsing all procedural block test files
#[test]
fn test_parse_all_procedural_block_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks");
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

#[test]
fn test_compound_assignment_operators() {
    use std::path::Path;

    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/assignments");

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

/// Test priority case statement
#[test]
fn test_priority_case() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/priority_case.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique case statement
#[test]
fn test_unique_case() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique_case.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique0 case statement
#[test]
fn test_unique0_case() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique0_case.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test priority casex statement
#[test]
fn test_priority_casex() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/procedural_blocks/priority_casex.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique casex statement
#[test]
fn test_unique_casex() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique_casex.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique0 casex statement
#[test]
fn test_unique0_casex() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique0_casex.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test priority casez statement
#[test]
fn test_priority_casez() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/procedural_blocks/priority_casez.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique casez statement
#[test]
fn test_unique_casez() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique_casez.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test unique0 casez statement
#[test]
fn test_unique0_casez() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/procedural_blocks/unique0_casez.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}
