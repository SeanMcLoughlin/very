//! Expression-related tests using shared fixture utilities.
//!
//! These tests verify parsing of SystemVerilog expression fixtures and
//! assert specific AST shapes where relevant.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};
use sv_parser::{BinaryOp, Expression, ModuleItem};

/// Test parsing all expression test files
#[test]
fn test_parse_all_expression_files() {
    assert_directory_parses("expressions");
}

sv_ok_tests! {
    expr_binary_add => "expressions/binary_add.sv",
    expr_module_with_assignment => "expressions/module_with_assignment.sv",
    expr_numbers => "expressions/numbers.sv",
    expr_parentheses => "expressions/parentheses.sv",
    expr_systemverilog_number_with_z => "expressions/systemverilog_number_with_z.sv",
}

#[test]
fn test_binary_add_expression() {
    let result = assert_parse_ok("expressions/binary_add.sv");

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
    let result = assert_parse_ok("expressions/numbers.sv");

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
    let result = assert_parse_ok("expressions/parentheses.sv");

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
    let result = assert_parse_ok("expressions/systemverilog_number_with_z.sv");

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
