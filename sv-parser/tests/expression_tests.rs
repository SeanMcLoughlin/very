use std::collections::HashMap;
use sv_parser::{BinaryOp, Expression, ModuleItem, SystemVerilogParser};

#[test]
fn test_parse_module_with_assignment() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input clk, output data); assign data = clk; endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        assert_eq!(items.len(), 1);

        if let ModuleItem::Assignment { target, expr } = &items[0] {
            assert_eq!(target, "data");
            if let Expression::Identifier(id) = expr {
                assert_eq!(id, "clk");
            } else {
                panic!("Expected identifier expression");
            }
        } else {
            panic!("Expected assignment");
        }
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_expression_binary_add() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign result = a + b; endmodule";

    let result = parser.parse_content(content).unwrap();

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
fn test_parse_expression_with_numbers() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign result = 42 * 3; endmodule";

    let result = parser.parse_content(content).unwrap();

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
fn test_parse_expression_parentheses() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign result = (a + b) * c; endmodule";

    let result = parser.parse_content(content).unwrap();

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
fn test_parse_systemverilog_number_with_z() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a != 8'b1101z001; endmodule";

    let result = parser.parse_content(content).unwrap();

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
