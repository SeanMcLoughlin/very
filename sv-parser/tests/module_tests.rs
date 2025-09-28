//! Module-related tests using file-based approach
//!
//! This module tests module parsing by running the parser against
//! SystemVerilog files in the test_files/modules/ directory.

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{ModuleItem, PortDirection, SystemVerilogParser};

/// Test parsing all module test files
#[test]
fn test_parse_all_module_files() {
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules");
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            println!("Testing module file: {}", filename);

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));

            println!("  ✅ Parsed successfully");
        }
    }
}

/// Test specific module structure expectations
#[test]
fn test_empty_module_structure() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/empty_module.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
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
fn test_module_with_ports_structure() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/module_with_ports.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

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
fn test_module_with_array_ports_structure() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/module_with_array_ports.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

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
fn test_multiple_modules_structure() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/modules/multiple_modules.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
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
