//! Operator-related tests using file-based approach
//!
//! This module tests operator parsing by running the parser against
//! SystemVerilog files in the test_files/operators/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{
    BinaryOp, Expression, ModuleItem, ProceduralBlockType, Statement, SystemVerilogParser, UnaryOp,
};

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

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));
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

    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary {
        op, left, right, ..
    } = expr
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalEquiv));
    let Expression::Identifier(left_id, _) = left.as_ref() else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");
    let Expression::Identifier(right_id, _) = right.as_ref() else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "b");
}

#[test]
fn test_logical_implication_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_implication.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary {
        op, left, right, ..
    } = expr
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalImpl));
    let Expression::Identifier(left_id, _) = left.as_ref() else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");
    let Expression::Identifier(right_id, _) = right.as_ref() else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "b");
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
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Equal));

    // Test not equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/not_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::NotEqual));

    // Test case equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/case_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::CaseEqual));

    // Test case not equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/case_not_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::CaseNotEqual));

    // Test wildcard equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/wildcard_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::WildcardEqual));

    // Test wildcard not equal
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/wildcard_not_equal.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::WildcardNotEqual));
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
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalAnd));

    // Test logical OR
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_or.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalOr));
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
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::GreaterThan));

    // Test less than
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[1] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LessThan));

    // Test greater than or equal
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[2] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::GreaterEqual));

    // Test less than or equal
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[3] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LessEqual));
}

#[test]
fn test_unary_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test unary NOT (~)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/unary_not.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, operand, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::Not));
    let Expression::Identifier(id, _) = operand.as_ref() else {
        panic!("Expected identifier operand");
    };
    assert_eq!(id, "a");

    // Test unary plus (+)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/unary_plus.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::Plus));

    // Test unary minus (-)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/unary_minus.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::Minus));
}

#[test]
fn test_reduction_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test reduction AND (&)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_and.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionAnd));

    // Test reduction OR (|)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_or.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionOr));

    // Test reduction XOR (^)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_xor.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionXor));

    // Test reduction NAND (~&)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_nand.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionNand));

    // Test reduction NOR (~|)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_nor.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionNor));

    // Test reduction XNOR (~^)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/reduction_xnor.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::ReductionXnor));

    // Test logical NOT (!)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/operators/logical_not.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let ModuleItem::Assignment { expr, .. } = &items[0] else {
        panic!("Expected assignment");
    };
    let Expression::Unary { op, .. } = expr else {
        panic!("Expected unary expression");
    };
    assert!(matches!(op, UnaryOp::LogicalNot));
}

#[test]
fn test_shift_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test logical left shift (<<)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_log_shl.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    // Find the initial block
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    // Get the first statement (the assignment)
    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment, got {:?}", first_stmt);
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalShiftLeft));

    // Test logical right shift (>>)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_log_shr.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalShiftRight));

    // Test arithmetic left shift (<<<)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_arith_shl.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::ArithmeticShiftLeft));

    // Test arithmetic right shift (>>>)
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_arith_shr.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::ArithmeticShiftRight));
}

#[test]
fn test_modulo_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_mod.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Modulo));
}

#[test]
fn test_bitwise_xnor_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/binary_op_bit_xnor.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] else {
        panic!("Expected module declaration");
    };
    let initial_block = items
        .iter()
        .find_map(|item| {
            if let ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } = item
            {
                if *block_type == ProceduralBlockType::Initial {
                    Some(statements)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Expected initial block");

    let first_stmt = &initial_block[0];
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };

    let Expression::Binary { op, .. } = expr else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::BitwiseXnor));
}
