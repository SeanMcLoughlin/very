//! Variable declaration tests using file-based approach
//!
//! This module tests variable declaration parsing by running the parser against
//! SystemVerilog files in the test_files/variables/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test parsing all variable declaration test files
#[test]
fn test_parse_all_variable_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables");
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

/// Test time unsigned declaration
#[test]
fn test_time_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/time_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test time signed declaration
#[test]
fn test_time_signed() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/time_signed.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test int unsigned declaration
#[test]
fn test_int_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/int_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test byte unsigned declaration
#[test]
fn test_byte_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/byte_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test shortint unsigned declaration
#[test]
fn test_shortint_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/shortint_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test longint unsigned declaration
#[test]
fn test_longint_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/longint_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test bit signed declaration
#[test]
fn test_bit_signed() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/bit_signed.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test integer unsigned declaration
#[test]
fn test_integer_unsigned() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/integer_unsigned.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test integer signed declaration
#[test]
fn test_integer_signed() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/integer_signed.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}
