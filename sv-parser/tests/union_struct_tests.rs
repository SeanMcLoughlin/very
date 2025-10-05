//! Union and struct tests
//!
//! This module tests parsing of union and struct declarations.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test basic unpacked union declaration
#[test]
fn test_basic_union() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/basic_union.sv");
    let content = std::fs::read_to_string(&path).unwrap();

    // Just verify it parses without error
    let _result = parser.parse_content(&content).unwrap();
}
