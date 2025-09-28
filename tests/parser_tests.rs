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
fn test_parse_module_with_array_ports() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test(input [3:0] a, output [7:0] b); endmodule";

    let result = parser.parse_content(content).unwrap();

    if let ModuleItem::ModuleDeclaration { name, ports, .. } = &result.items[0] {
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
