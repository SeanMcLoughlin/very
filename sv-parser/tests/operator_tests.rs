use std::collections::HashMap;
use sv_parser::{BinaryOp, Expression, ModuleItem, SystemVerilogParser};

#[test]
fn test_parse_logical_equivalence_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a <-> b; endmodule";

    let result = parser.parse_content(content).unwrap();

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
fn test_parse_logical_implication_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a -> b; endmodule";

    let result = parser.parse_content(content).unwrap();

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
fn test_parse_equality_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a == b; endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::Equal));
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
fn test_parse_not_equal_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a != b; endmodule";

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
fn test_parse_logical_and_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a && b; endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::LogicalAnd));
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
fn test_parse_logical_or_operator() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test; assign c = a || b; endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, left, right } = expr {
                assert!(matches!(op, BinaryOp::LogicalOr));
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
fn test_parse_comparison_operators() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test greater than
    let content = "module test; assign c = a > b; endmodule";
    let result = parser.parse_content(content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::GreaterThan));
            }
        }
    }

    // Test less than
    let content = "module test; assign c = a < b; endmodule";
    let result = parser.parse_content(content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LessThan));
            }
        }
    }

    // Test greater than or equal
    let content = "module test; assign c = a >= b; endmodule";
    let result = parser.parse_content(content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::GreaterEqual));
            }
        }
    }

    // Test less than or equal
    let content = "module test; assign c = a <= b; endmodule";
    let result = parser.parse_content(content).unwrap();
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        if let ModuleItem::Assignment { expr, .. } = &items[0] {
            if let Expression::Binary { op, .. } = expr {
                assert!(matches!(op, BinaryOp::LessEqual));
            }
        }
    }
}
