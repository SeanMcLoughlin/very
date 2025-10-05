//! Operator-related tests using file-based approach
//!
//! This module tests operator parsing by running the parser against
//! SystemVerilog files in the test_files/operators/ directory.

#[path = "common/mod.rs"]
mod common;

use common::{
    assert_directory_parses, assert_parse_ok,
    ast::{assignment_expr, first_assignment_expr, first_initial_block_statements},
};
use sv_parser::{BinaryOp, Expression, Statement, UnaryOp};

/// Test parsing all operator test files
#[test]
fn test_parse_all_operator_files() {
    assert_directory_parses("operators");
}

#[test]
fn test_logical_equivalence_operator() {
    let result = assert_parse_ok("operators/logical_equivalence.sv");

    let expr = first_assignment_expr(&result);

    // Look up expression in arena
    let expr_data = result.expr_arena.get(expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_data
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalEquiv));

    // Look up operands in arena
    let left_data = result.expr_arena.get(*left);
    let Expression::Identifier(left_id, _) = left_data else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");

    let right_data = result.expr_arena.get(*right);
    let Expression::Identifier(right_id, _) = right_data else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "b");
}

#[test]
fn test_logical_implication_operator() {
    let result = assert_parse_ok("operators/logical_implication.sv");

    let expr = first_assignment_expr(&result);

    // Look up expression in arena
    let expr_data = result.expr_arena.get(expr);
    let Expression::Binary {
        op, left, right, ..
    } = expr_data
    else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::LogicalImpl));
    let Expression::Identifier(left_id, _) = result.expr_arena.get(*left) else {
        panic!("Expected identifier on left");
    };
    assert_eq!(left_id, "a");
    let Expression::Identifier(right_id, _) = result.expr_arena.get(*right) else {
        panic!("Expected identifier on right");
    };
    assert_eq!(right_id, "b");
}

#[test]
fn test_equality_operators() {
    for (path, expected_op) in [
        ("operators/equality.sv", BinaryOp::Equal),
        ("operators/not_equal.sv", BinaryOp::NotEqual),
        ("operators/case_equal.sv", BinaryOp::CaseEqual),
        ("operators/case_not_equal.sv", BinaryOp::CaseNotEqual),
        ("operators/wildcard_equal.sv", BinaryOp::WildcardEqual),
        (
            "operators/wildcard_not_equal.sv",
            BinaryOp::WildcardNotEqual,
        ),
    ] {
        let result = assert_parse_ok(path);
        let expr = first_assignment_expr(&result);
        let expr_data = result.expr_arena.get(expr);
        let Expression::Binary { op, .. } = expr_data else {
            panic!("Expected binary expression");
        };
        assert_eq!(*op, expected_op);
    }
}

#[test]
fn test_logical_and_or_operators() {
    for (path, expected_op) in [
        ("operators/logical_and.sv", BinaryOp::LogicalAnd),
        ("operators/logical_or.sv", BinaryOp::LogicalOr),
    ] {
        let result = assert_parse_ok(path);
        let expr = first_assignment_expr(&result);
        let expr_data = result.expr_arena.get(expr);
        let Expression::Binary { op, .. } = expr_data else {
            panic!("Expected binary expression");
        };
        assert_eq!(*op, expected_op);
    }
}

#[test]
fn test_comparison_operators() {
    let result = assert_parse_ok("operators/comparison_operators.sv");

    // This file contains 4 modules with different comparison operators
    assert_eq!(result.items.len(), 4);

    let expected_ops = [
        BinaryOp::GreaterThan,
        BinaryOp::LessThan,
        BinaryOp::GreaterEqual,
        BinaryOp::LessEqual,
    ];

    for (module_index, expected_op) in expected_ops.into_iter().enumerate() {
        let expr = assignment_expr(&result, module_index, 0);
        let expr_data = result.expr_arena.get(expr);
        let Expression::Binary { op, .. } = expr_data else {
            panic!("Expected binary expression");
        };
        assert_eq!(*op, expected_op);
    }
}

#[test]
fn test_unary_operators() {
    for (path, expected_op, expected_ident) in [
        ("operators/unary_not.sv", UnaryOp::Not, Some("a")),
        ("operators/unary_plus.sv", UnaryOp::Plus, None),
        ("operators/unary_minus.sv", UnaryOp::Minus, None),
    ] {
        let result = assert_parse_ok(path);
        let expr = first_assignment_expr(&result);
        let expr_data = result.expr_arena.get(expr);
        let Expression::Unary { op, operand, .. } = expr_data else {
            panic!("Expected unary expression");
        };
        assert_eq!(*op, expected_op);

        if let Some(expected_name) = expected_ident {
            let Expression::Identifier(name, _) = result.expr_arena.get(*operand) else {
                panic!("Expected identifier operand");
            };
            assert_eq!(name, expected_name);
        }
    }
}

#[test]
fn test_reduction_operators() {
    for (path, expected_op) in [
        ("operators/reduction_and.sv", UnaryOp::ReductionAnd),
        ("operators/reduction_or.sv", UnaryOp::ReductionOr),
        ("operators/reduction_xor.sv", UnaryOp::ReductionXor),
        ("operators/reduction_nand.sv", UnaryOp::ReductionNand),
        ("operators/reduction_nor.sv", UnaryOp::ReductionNor),
        ("operators/reduction_xnor.sv", UnaryOp::ReductionXnor),
        ("operators/logical_not.sv", UnaryOp::LogicalNot),
    ] {
        let result = assert_parse_ok(path);
        let expr = first_assignment_expr(&result);
        let expr_data = result.expr_arena.get(expr);
        let Expression::Unary { op, .. } = expr_data else {
            panic!("Expected unary expression");
        };
        assert_eq!(*op, expected_op);
    }
}

#[test]
fn test_shift_operators() {
    for (path, expected_op) in [
        ("binary_op_log_shl.sv", BinaryOp::LogicalShiftLeft),
        ("binary_op_log_shr.sv", BinaryOp::LogicalShiftRight),
        ("binary_op_arith_shl.sv", BinaryOp::ArithmeticShiftLeft),
        ("binary_op_arith_shr.sv", BinaryOp::ArithmeticShiftRight),
    ] {
        let result = assert_parse_ok(path);
        let initial_block = first_initial_block_statements(&result);
        let first_stmt_ref = initial_block[0];
        let first_stmt = result.stmt_arena.get(first_stmt_ref);
        let Statement::Assignment { expr, .. } = first_stmt else {
            panic!("Expected assignment");
        };
        let expr_data = result.expr_arena.get(*expr);
        let Expression::Binary { op, .. } = expr_data else {
            panic!("Expected binary expression");
        };
        assert_eq!(*op, expected_op);
    }
}

#[test]
fn test_modulo_operator() {
    let result = assert_parse_ok("binary_op_mod.sv");
    let initial_block = first_initial_block_statements(&result);
    let first_stmt_ref = initial_block[0];
    let first_stmt = result.stmt_arena.get(first_stmt_ref);
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };
    let expr_data = result.expr_arena.get(*expr);
    let Expression::Binary { op, .. } = expr_data else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::Modulo));
}

#[test]
fn test_bitwise_xnor_operator() {
    let result = assert_parse_ok("binary_op_bit_xnor.sv");
    let initial_block = first_initial_block_statements(&result);
    let first_stmt_ref = initial_block[0];
    let first_stmt = result.stmt_arena.get(first_stmt_ref);
    let Statement::Assignment { expr, .. } = first_stmt else {
        panic!("Expected assignment");
    };
    let expr_data = result.expr_arena.get(*expr);
    let Expression::Binary { op, .. } = expr_data else {
        panic!("Expected binary expression");
    };
    assert!(matches!(op, BinaryOp::BitwiseXnor));
}
