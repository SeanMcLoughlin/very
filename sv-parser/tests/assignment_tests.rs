//! Assignment-related tests
//!
//! This module tests continuous assignment parsing including delay specifications.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{Delay, ModuleItem, SystemVerilogParser};

/// Test parsing all assignment test files
#[test]
fn test_parse_all_assignment_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/assignments");
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

/// Test continuous assignment with delay (#10)
#[test]
fn test_cont_assignment_with_delay() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/assignments/cont_assignment_delay.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    assert_eq!(result.items.len(), 1);

    // Check module structure
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        // Should have: wire w; and assign #10 w = a & b;
        assert_eq!(items.len(), 2, "Expected 2 items in module");

        // Check the assignment (second item)
        if let ModuleItem::Assignment {
            delay,
            target,
            expr,
            ..
        } = &items[1]
        {
            // Verify delay is present and has value "10"
            assert!(delay.is_some(), "Assignment should have a delay");
            if let Some(Delay::Value(val)) = delay {
                assert_eq!(val, "10", "Delay value should be 10");
            } else {
                panic!("Expected Delay::Value, got {:?}", delay);
            }

            // Verify target
            assert_eq!(target, "w", "Assignment target should be 'w'");

            // Expression is a & b - just verify it exists
            assert!(
                format!("{:?}", expr).contains("Binary"),
                "Expression should be binary operation"
            );
        } else {
            panic!("Expected Assignment, got {:?}", items[1]);
        }
    } else {
        panic!("Expected ModuleDeclaration");
    }
}

/// Test net declaration with delay (wire #10 w;)
#[test]
fn test_net_declaration_with_delay() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/assignments/cont_assignment_net_delay.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    assert_eq!(result.items.len(), 1);

    // Check module structure
    if let ModuleItem::ModuleDeclaration { items, .. } = &result.items[0] {
        // Should have: wire #10 w; and assign w = a & b;
        assert_eq!(items.len(), 2, "Expected 2 items in module");

        // Check the wire declaration (first item)
        if let ModuleItem::VariableDeclaration {
            data_type,
            delay,
            name,
            ..
        } = &items[0]
        {
            // Verify it's a wire
            assert_eq!(data_type, "wire", "Should be a wire declaration");

            // Verify delay is present and has value "10"
            assert!(delay.is_some(), "Wire declaration should have a delay");
            if let Some(Delay::Value(val)) = delay {
                assert_eq!(val, "10", "Delay value should be 10");
            } else {
                panic!("Expected Delay::Value, got {:?}", delay);
            }

            // Verify name
            assert_eq!(name, "w", "Wire name should be 'w'");
        } else {
            panic!("Expected VariableDeclaration, got {:?}", items[0]);
        }

        // Check the assignment (second item) - should have no delay
        if let ModuleItem::Assignment { delay, .. } = &items[1] {
            assert!(
                delay.is_none(),
                "Assignment should not have delay (delay is on the wire)"
            );
        } else {
            panic!("Expected Assignment as second item, got {:?}", items[1]);
        }
    } else {
        panic!("Expected ModuleDeclaration");
    }
}
