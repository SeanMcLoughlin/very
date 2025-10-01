use std::collections::HashMap;
use std::path::Path;
use sv_parser::SystemVerilogParser;

/// Test that runs the parser against all SystemVerilog test files
#[test]
fn test_parse_all_valid_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test modules
    test_directory(&parser, &test_files_dir.join("modules"), true);

    // Test expressions
    test_directory(&parser, &test_files_dir.join("expressions"), true);

    // Test operators
    test_directory(&parser, &test_files_dir.join("operators"), true);

    // Test preprocessor (may have missing includes, so allow some failures)
    test_directory(&parser, &test_files_dir.join("preprocessor"), false);
}

/// Test that error files properly fail to parse
#[test]
fn test_parse_error_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    // Test error files - these should fail to parse
    test_directory_should_fail(&parser, &test_files_dir.join("errors"));
}

fn test_directory(parser: &SystemVerilogParser, dir: &Path, expect_all_success: bool) {
    if !dir.exists() {
        return;
    }

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();

            match std::fs::read_to_string(&path) {
                Ok(content) => match parser.parse_content(&content) {
                    Ok(_) => {}
                    Err(e) => {
                        if expect_all_success {
                            panic!(
                                "Expected {} to parse successfully, but got error: {}",
                                filename, e
                            );
                        }
                    }
                },
                Err(e) => {
                    if expect_all_success {
                        panic!("Failed to read test file {}: {}", filename, e);
                    }
                }
            }
        }
    }
}

fn test_directory_should_fail(parser: &SystemVerilogParser, dir: &Path) {
    if !dir.exists() {
        return;
    }

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();

            match std::fs::read_to_string(&path) {
                Ok(content) => match parser.parse_content(&content) {
                    Ok(_) => {
                        panic!("Expected {} to fail parsing, but it succeeded", filename);
                    }
                    Err(_) => {}
                },
                Err(e) => {
                    panic!("Failed to read error test file {}: {}", filename, e);
                }
            }
        }
    }
}

/// Individual tests for specific parsing features
#[cfg(test)]
mod specific_tests {
    use super::*;
    use sv_parser::{BinaryOp, Expression, ModuleItem, PortDirection};

    #[test]
    fn test_empty_module_structure() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/empty_module.sv"),
        )
        .unwrap();

        let result = parser.parse_content(&content).unwrap();
        assert_eq!(result.items.len(), 1);

        if let ModuleItem::ModuleDeclaration {
            name, ports, items, ..
        } = &result.items[0]
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
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/module_with_ports.sv"),
        )
        .unwrap();

        let result = parser.parse_content(&content).unwrap();

        if let ModuleItem::ModuleDeclaration {
            name, ports, items, ..
        } = &result.items[0]
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
    fn test_binary_add_expression() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/expressions/binary_add.sv"),
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
        assert!(matches!(op, BinaryOp::Add));
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
    fn test_logical_operators() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());

        // Test logical equivalence
        let content = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/operators/logical_equivalence.sv"),
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
        assert!(matches!(op, BinaryOp::LogicalEquiv));

        // Test logical implication
        let content = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/operators/logical_implication.sv"),
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
        assert!(matches!(op, BinaryOp::LogicalImpl));
    }
}
