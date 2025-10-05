//! Assignment-related tests leveraging the shared parser harness.
//!
//! Focuses on continuous assignments and delays under `test_files/assignments/`.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok, ast::module_items};
use sv_parser::{Delay, Expression, ModuleItem};

/// Smoke test: every assignment fixture should parse successfully.
#[test]
fn test_parse_all_assignment_files() {
    assert_directory_parses("assignments");
}

sv_ok_tests! {
    assign_add => "assignments/add_assign.sv",
    assign_and => "assignments/and_assign.sv",
    assign_ashl => "assignments/ashl_assign.sv",
    assign_ashr => "assignments/ashr_assign.sv",
    assign_cont_delay => "assignments/cont_assignment_delay.sv",
    assign_cont_net_delay => "assignments/cont_assignment_net_delay.sv",
    assign_div => "assignments/div_assign.sv",
    assign_mod => "assignments/mod_assign.sv",
    assign_mul => "assignments/mul_assign.sv",
    assign_or => "assignments/or_assign.sv",
    assign_shl => "assignments/shl_assign.sv",
    assign_shr => "assignments/shr_assign.sv",
    assign_sub => "assignments/sub_assign.sv",
    assign_xor => "assignments/xor_assign.sv",
}

/// Continuous assignment with an inline delay (`assign #10 w = a & b;`).
#[test]
fn test_cont_assignment_with_delay() {
    let result = assert_parse_ok("assignments/cont_assignment_delay.sv");
    let items = module_items(&result, 0);
    assert_eq!(items.len(), 2, "Expected wire declaration + assignment");

    let assign_ref = items[1];
    let ModuleItem::Assignment {
        delay,
        target,
        expr,
        ..
    } = result.module_item_arena.get(assign_ref)
    else {
        panic!("Expected assignment as second module item");
    };

    // Delay should be `#10`.
    match delay {
        Some(Delay::Value(val)) => assert_eq!(val, "10"),
        other => panic!("Expected Delay::Value(\"10\"), got {:?}", other),
    }

    // Target is the identifier `w`.
    let target_expr = result.expr_arena.get(*target);
    let Expression::Identifier(name, _) = target_expr else {
        panic!("Expected identifier target, got {:?}", target_expr);
    };
    assert_eq!(name, "w");

    // RHS should be a binary expression (`a & b`).
    let rhs_expr = result.expr_arena.get(*expr);
    assert!(matches!(rhs_expr, Expression::Binary { .. }));
}

/// Net declaration with delay (`wire #10 w;`) followed by assignment.
#[test]
fn test_net_declaration_with_delay() {
    let result = assert_parse_ok("assignments/cont_assignment_net_delay.sv");
    let items = module_items(&result, 0);
    assert_eq!(items.len(), 2, "Expected wire declaration + assignment");

    // First item: wire declaration with delay.
    let ModuleItem::VariableDeclaration {
        data_type,
        delay,
        name,
        ..
    } = result.module_item_arena.get(items[0])
    else {
        panic!("Expected variable declaration");
    };
    assert_eq!(data_type, "wire");
    match delay {
        Some(Delay::Value(val)) => assert_eq!(val, "10"),
        other => panic!("Expected Delay::Value(\"10\"), got {:?}", other),
    }
    assert_eq!(name, "w");

    // Second item: assignment without an additional delay.
    let ModuleItem::Assignment { delay, .. } = result.module_item_arena.get(items[1]) else {
        panic!("Expected assignment after wire declaration");
    };
    assert!(delay.is_none(), "Assignment should inherit delay from wire");
}
