//! Operator-related tests using file-based approach
//!
//! This module tests operator parsing by running the parser against
//! SystemVerilog files in the test_files/operators/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{BinaryOp, Expression, ModuleItem, SystemVerilogParser};

/// Test parsing all operator test files
#[test]
fn test_parse_all_operator_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            println!("Testing operator file: {}", filename);

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
fn test_logical_equivalence_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_equivalence.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::LogicalEquiv));
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
fn test_logical_implication_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_implication.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::LogicalImpl));
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
fn test_equality_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test equality
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/equality.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::Equal));
            }
        }
    }

    // Test not equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/not_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::NotEqual));
            }
        }
    }
}

#[test]
fn test_logical_and_or_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test logical AND
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_and.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LogicalAnd));
            }
        }
    }

    // Test logical OR
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_or.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LogicalOr));
            }
        }
    }
}

#[test]
fn test_comparison_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/comparison_operators.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    // This file contains 4 modules with different comparison operators
    assert_eq!(result.items.len(), 4);

    // Test greater than
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::GreaterThan));
            }
        }
    }

    // Test less than
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[1] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LessThan));
            }
        }
    }

    // Test greater than or equal
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[2] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::GreaterEqual));
            }
        }
    }

    // Test less than or equal
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[3] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LessEqual));
            }
        }
    }
}
