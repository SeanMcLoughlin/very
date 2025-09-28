use std::collections::HashMap;
use sv_chumsky::{BinaryOp, Expression, ModuleItem, PortDirection, SystemVerilogParser};

#[test]
fn test_parse_empty_module() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module empty; endmodule";

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "empty");
        assert_eq!(ports.len(), 0);
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_parse_module_with_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input clk, output reg data); endmodule";

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
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
fn test_parse_module_with_no_direction_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(clk, reset); endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { ports, .. } = &result.items[0] {
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].name, "clk");
        assert_eq!(ports[0].direction, None);
        assert_eq!(ports[1].name, "reset");
        assert_eq!(ports[1].direction, None);
    } else {
        panic!("Expected module declaration");
    }
}

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
fn test_parse_module_with_port_declaration() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module test(clk, data);
    input wire clk;
    output reg data;
endmodule"#;

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        assert_eq!(items.len(), 2);

        // Check input declaration
        if let ModuleItem::PortDeclaration {
            direction,
            port_type,
            name,
        } = &items[0]
        {
            assert_eq!(*direction, PortDirection::Input);
            assert_eq!(port_type, "wire");
            assert_eq!(name, "clk");
        } else {
            panic!("Expected port declaration");
        }

        // Check output declaration
        if let ModuleItem::PortDeclaration {
            direction,
            port_type,
            name,
        } = &items[1]
        {
            assert_eq!(*direction, PortDirection::Output);
            assert_eq!(port_type, "reg");
            assert_eq!(name, "data");
        } else {
            panic!("Expected port declaration");
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
fn test_parse_multiple_modules() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module first; endmodule
module second(input clk); endmodule
"#;

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 2);

    if let ModuleItem::ModuleDeclaration { name, .. } = &result.items[0] {
        assert_eq!(name, "first");
    } else {
        panic!("Expected first module");
    }

    if let ModuleItem::ModuleDeclaration { name, ports, .. } = &result.items[1] {
        assert_eq!(name, "second");
        assert_eq!(ports.len(), 1);
    } else {
        panic!("Expected second module");
    }
}

#[test]
fn test_parse_error_invalid_syntax() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module invalid syntax here";

    let result = parser.parse_content(content);
    assert!(result.is_err());
}

#[test]
fn test_parse_error_incomplete_module() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input clk)";

    let result = parser.parse_content(content);
    assert!(result.is_err());
}

#[test]
fn test_parse_whitespace_handling() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"

    module   test   (   input   clk   ,   output   data   )   ;
        assign   data   =   clk   ;
    endmodule

    "#;

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "test");
        assert_eq!(ports.len(), 2);
        assert_eq!(items.len(), 1);
    } else {
        panic!("Expected module declaration");
    }
}
