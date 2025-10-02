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

/// Test trireg net declaration
#[test]
fn test_net_trireg() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/trireg_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test uwire net declaration
#[test]
fn test_net_uwire() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/uwire_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test wand net declaration
#[test]
fn test_net_wand() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/wand_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test wor net declaration
#[test]
fn test_net_wor() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/wor_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test tri net declaration
#[test]
fn test_net_tri() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/tri_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test triand net declaration
#[test]
fn test_net_triand() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/triand_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test trior net declaration
#[test]
fn test_net_trior() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/trior_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test tri0 net declaration
#[test]
fn test_net_tri0() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/tri0_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test tri1 net declaration
#[test]
fn test_net_tri1() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/tri1_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test supply0 net declaration
#[test]
fn test_net_supply0() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/supply0_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test supply1 net declaration
#[test]
fn test_net_supply1() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/supply1_declaration.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test dynamic array basic declaration
#[test]
fn test_dynamic_array_basic() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/dynamic_array_basic.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test fixed size unpacked array declaration
#[test]
fn test_fixed_unpacked_array() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/fixed_unpacked_array.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}

/// Test multidimensional array with dynamic dimension
#[test]
fn test_multidim_array() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/variables/multidim_array.sv");
    let content = std::fs::read_to_string(&path).unwrap();
    parser.parse_content(&content).unwrap();
}
