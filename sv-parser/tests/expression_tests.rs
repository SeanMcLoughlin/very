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

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));
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

    let item = result.module_item_arena.get(result.items[0]);
    let ModuleItem::ModuleDeclaration { items, .. } = item else {
        panic!("Expected module declaration");
    };
    let item0 = result.module_item_arena.get(items[0]);
    let ModuleItem::Assignment { expr, .. } = item0 else {
        panic!("Expected assignment");
    };
    let expr_val = result.expr_arena.get(*expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_val
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Add));
    let left_expr = result.expr_arena.get(*left);
    let Expression::Identifier(left_id, _) = left_expr else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");
    let right_expr = result.expr_arena.get(*right);
    let Expression::Identifier(right_id, _) = right_expr else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "b");
}

#[test]
fn test_number_expressions() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/numbers.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let item = result.module_item_arena.get(result.items[0]);
    let ModuleItem::ModuleDeclaration { items, .. } = item else {
        panic!("Expected module declaration");
    };
    let item0 = result.module_item_arena.get(items[0]);
    let ModuleItem::Assignment { expr, .. } = item0 else {
        panic!("Expected assignment");
    };
    let expr_val = result.expr_arena.get(*expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_val
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Mul));
    let left_expr = result.expr_arena.get(*left);
    let Expression::Number(left_num, _) = left_expr else {
        panic!("Expected number on left");
    };
    assert_eq!(left_num, "42");
    let right_expr = result.expr_arena.get(*right);
    let Expression::Number(right_num, _) = right_expr else {
        panic!("Expected number on right");
    };
    assert_eq!(right_num, "3");
}

#[test]
fn test_parentheses_precedence() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/parentheses.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let item = result.module_item_arena.get(result.items[0]);
    let ModuleItem::ModuleDeclaration { items, .. } = item else {
        panic!("Expected module declaration");
    };
    let item0 = result.module_item_arena.get(items[0]);
    let ModuleItem::Assignment { expr, .. } = item0 else {
        panic!("Expected assignment");
    };
    // Should parse as: (a + b) * c
    let expr_val = result.expr_arena.get(*expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_val
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Mul));
    let left_expr = result.expr_arena.get(*left);
    let Expression::Binary { op: left_op, .. } = left_expr else {
        panic!("Expected binary expression on left");
    };
    assert!(matches!(left_op, BinaryOp::Add));
    let right_expr = result.expr_arena.get(*right);
    let Expression::Identifier(right_id, _) = right_expr else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "c");
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

    let item = result.module_item_arena.get(result.items[0]);
    let ModuleItem::ModuleDeclaration { items, .. } = item else {
        panic!("Expected module declaration");
    };
    let item0 = result.module_item_arena.get(items[0]);
    let ModuleItem::Assignment { expr, .. } = item0 else {
        panic!("Expected assignment");
    };
    let expr_val = result.expr_arena.get(*expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_val
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::NotEqual));
    let left_expr = result.expr_arena.get(*left);
    let Expression::Identifier(left_id, _) = left_expr else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");
    let right_expr = result.expr_arena.get(*right);
    let Expression::Number(right_num, _) = right_expr else {
        panic!("Expected number on right");
    };
    assert_eq!(right_num, "8'b1101z001");
}
