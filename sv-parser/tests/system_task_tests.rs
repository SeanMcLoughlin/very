//! System task and function tests
//!
//! This module tests system task and function parsing by running the parser
//! against SystemVerilog files in the test_files/system_tasks/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test parsing all system task test files
#[test]
fn test_parse_all_system_task_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/system_tasks");
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

/// Test parsing $atan function
#[test]
fn test_atan_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/system_tasks/atan_function.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $atan function: {:?}",
        result.err()
    );
}

/// Test parsing $sin function
#[test]
fn test_sin_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/system_tasks/sin_function.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $sin function: {:?}",
        result.err()
    );
}

/// Test parsing $cos function
#[test]
fn test_cos_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/system_tasks/cos_function.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $cos function: {:?}",
        result.err()
    );
}

/// Test parsing $rose sampled value function
#[test]
fn test_rose_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/sampled_rose.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $rose function: {:?}",
        result.err()
    );
}

/// Test parsing $fell sampled value function
#[test]
fn test_fell_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/sampled_fell.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $fell function: {:?}",
        result.err()
    );
}

/// Test parsing $stable sampled value function
#[test]
fn test_stable_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/sampled_stable.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $stable function: {:?}",
        result.err()
    );
}

/// Test parsing $past sampled value function
#[test]
fn test_past_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/sampled_past.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content);
    assert!(
        result.is_ok(),
        "Failed to parse $past function: {:?}",
        result.err()
    );
}
