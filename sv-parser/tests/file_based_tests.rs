#[path = "common/mod.rs"]
pub mod common;

use common::{
    assert_directory_fails, assert_directory_parses, assert_parse_ok, ast::first_assignment_expr,
};
use sv_parser::{BinaryOp, Expression, ModuleItem, PortDirection};

/// Test that runs the parser against all SystemVerilog test files
#[test]
fn test_parse_all_valid_files() {
    assert_directory_parses("modules");
    assert_directory_parses("expressions");
    assert_directory_parses("operators");
    assert_directory_parses("assignments");
}

/// Test that error files properly fail to parse
#[test]
fn test_parse_error_files() {
    assert_directory_fails("errors");
}

/// Individual tests for specific parsing features
#[cfg(test)]
mod specific_tests {
    use super::*;

    #[test]
    fn test_empty_module_structure() {
        let result = assert_parse_ok("modules/empty_module.sv");
        assert_eq!(result.items.len(), 1);

        let item = result.module_item_arena.get(result.items[0]);
        if let ModuleItem::ModuleDeclaration {
            name, ports, items, ..
        } = item
        {
            assert_eq!(name, "empty");
            assert_eq!(ports.len(), 0);
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected module declaration");
        }
    }

    #[test]
    fn test_module_with_ports_structure() {
        let result = assert_parse_ok("modules/module_with_ports.sv");

        let item = result.module_item_arena.get(result.items[0]);
        if let ModuleItem::ModuleDeclaration {
            name, ports, items, ..
        } = item
        {
            assert_eq!(name, "test");
            assert_eq!(ports.len(), 2);
            assert_eq!(items.len(), 0);

            // Check first port (input clk)
            assert_eq!(ports[0].name, "clk");
            assert_eq!(ports[0].direction, Some(PortDirection::Input));

            // Check second port (output data)
            assert_eq!(ports[1].name, "data");
            assert_eq!(ports[1].direction, Some(PortDirection::Output));
        } else {
            panic!("Expected module declaration");
        }
    }

    #[test]
    fn test_module_with_array_ports_structure() {
        let result = assert_parse_ok("modules/module_with_array_ports.sv");

        let item = result.module_item_arena.get(result.items[0]);
        if let ModuleItem::ModuleDeclaration { name, ports, .. } = item {
            assert_eq!(name, "test");
            assert_eq!(ports.len(), 2);

            // Check first port (input [3:0] a)
            assert_eq!(ports[0].name, "a");
            assert_eq!(ports[0].direction, Some(PortDirection::Input));
            if let Some(ref range) = ports[0].range {
                assert_eq!(range.msb, "3");
                assert_eq!(range.lsb, "0");
            } else {
                panic!("Expected range for port a");
            }

            // Check second port (output [7:0] b)
            assert_eq!(ports[1].name, "b");
            assert_eq!(ports[1].direction, Some(PortDirection::Output));
            if let Some(ref range) = ports[1].range {
                assert_eq!(range.msb, "7");
                assert_eq!(range.lsb, "0");
            } else {
                panic!("Expected range for port b");
            }
        } else {
            panic!("Expected module declaration");
        }
    }

    #[test]
    fn test_multiple_modules_structure() {
        let result = assert_parse_ok("modules/multiple_modules.sv");
        assert_eq!(result.items.len(), 2);

        let item0 = result.module_item_arena.get(result.items[0]);
        if let ModuleItem::ModuleDeclaration { name, .. } = item0 {
            assert_eq!(name, "first");
        } else {
            panic!("Expected first module");
        }

        let item1 = result.module_item_arena.get(result.items[1]);
        if let ModuleItem::ModuleDeclaration { name, ports, .. } = item1 {
            assert_eq!(name, "second");
            assert_eq!(ports.len(), 1);
        } else {
            panic!("Expected second module");
        }
    }

    #[test]
    fn test_binary_add_expression() {
        let result = assert_parse_ok("expressions/binary_add.sv");

        let expr = first_assignment_expr(&result);
        let expr_val = result.expr_arena.get(expr);
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
    fn test_logical_operators() {
        let result = assert_parse_ok("operators/logical_equivalence.sv");
        let expr_val = result.expr_arena.get(first_assignment_expr(&result));
        let Expression::Binary { op, .. } = expr_val else {
            panic!("Expected binary expression");
        };
        assert!(matches!(op, BinaryOp::LogicalEquiv));

        let result = assert_parse_ok("operators/logical_implication.sv");
        let expr_val = result.expr_arena.get(first_assignment_expr(&result));
        let Expression::Binary { op, .. } = expr_val else {
            panic!("Expected binary expression");
        };
        assert!(matches!(op, BinaryOp::LogicalImpl));
    }
}
