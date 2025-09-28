//! Expression-related tests using file-based approach
//!
//! This module tests expression parsing by running the parser against
//! SystemVerilog files in the test_files/expressions/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{BinaryOp, Expression, ModuleItem, SystemVerilogParser};

/// Test parsing all expression test files
#[test]
fn test_parse_all_expression_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            println!("Testing expression file: {}", filename);

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));

            println!("  ✅ Parsed successfully");
        }
    }
}

#[test]
fn test_binary_add_expression() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/binary_add.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::Add));
                if let Expression::Identifier(left_id) = left.as_ref() {
                    assert_eq!(left_id, "a");
                } else {
                    panic!("Expected identifier on left");
                }
                if let Expression::Identifier(right_id) = right.as_ref() {
                    assert_eq!(right_id, "b");
                } else {
                    panic!("Expected identifier on right");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    }
}

#[test]
fn test_number_expressions() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/numbers.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::Mul));
                if let Expression::Number(left_num) = left.as_ref() {
                    assert_eq!(left_num, "42");
                } else {
                    panic!("Expected number on left");
                }
                if let Expression::Number(right_num) = right.as_ref() {
                    assert_eq!(right_num, "3");
                } else {
                    panic!("Expected number on right");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    }
}

#[test]
fn test_parentheses_precedence() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/parentheses.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            // Should parse as: (a + b) * c
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::Mul));
                if let Expression::Binary { op: left_op, .. } = left.as_ref() {
                    assert!(matches!(left_op, BinaryOp::Add));
                } else {
                    panic!("Expected binary expression on left");
                }
                if let Expression::Identifier(right_id) = right.as_ref() {
                    assert_eq!(right_id, "c");
                } else {
                    panic!("Expected identifier on right");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    }
}

#[test]
fn test_systemverilog_numbers() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/expressions/systemverilog_number_with_z.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::NotEqual));
                if let Expression::Identifier(left_id) = left.as_ref() {
                    assert_eq!(left_id, "a");
                } else {
                    panic!("Expected identifier on left");
                }
                if let Expression::Number(right_num) = right.as_ref() {
                    assert_eq!(right_num, "8'b1101z001");
                } else {
                    panic!("Expected number on right");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    }
}
