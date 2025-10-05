//! Global clocking tests
//!
//! Tests for global clocking declarations from sv-tests

use std::collections::HashMap;
use std::path::Path;
use sv_parser::{ModuleItem, SystemVerilogParser};

#[test]
fn test_steady_gclk_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20.13--steady_gclk.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    assert_eq!(result.items.len(), 1);

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { name, items, .. } = item {
        assert_eq!(name, "top");

        // Should have: variable declarations (a, clk), global clocking, and assert property
        assert!(
            items.len() >= 2,
            "Expected at least 2 items (variables and global clocking), got {}",
            items.len()
        );

        // Find the global clocking item
        let has_global_clocking = items.iter().any(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            matches!(item, ModuleItem::GlobalClocking { .. })
        });
        assert!(has_global_clocking, "Expected global clocking declaration");
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_changing_gclk_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20.13--changing_gclk.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    assert_eq!(result.items.len(), 1);

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { name, items, .. } = item {
        assert_eq!(name, "top");

        // Find the global clocking item
        let has_global_clocking = items.iter().any(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            matches!(item, ModuleItem::GlobalClocking { .. })
        });
        assert!(has_global_clocking, "Expected global clocking declaration");
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_stable_gclk_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20.13--stable_gclk.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();
    assert_eq!(result.items.len(), 1);

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { name, items, .. } = item {
        assert_eq!(name, "top");

        // Find the global clocking item
        let has_global_clocking = items.iter().any(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            matches!(item, ModuleItem::GlobalClocking { .. })
        });
        assert!(has_global_clocking, "Expected global clocking declaration");
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_global_clocking_with_identifier() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/global_clocking_with_identifier.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = item {
        // Find the global clocking item
        let global_clocking = items.iter().find_map(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            if let ModuleItem::GlobalClocking { identifier, .. } = item {
                Some(identifier)
            } else {
                None
            }
        });

        assert!(
            global_clocking.is_some(),
            "Expected global clocking declaration"
        );
        assert_eq!(global_clocking.unwrap().as_ref().unwrap(), "sys");
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_global_clocking_with_end_label() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/global_clocking_with_end_label.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = item {
        // Find the global clocking item
        let global_clocking = items.iter().find_map(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            if let ModuleItem::GlobalClocking {
                identifier,
                end_label,
                ..
            } = item
            {
                Some((identifier, end_label))
            } else {
                None
            }
        });

        assert!(
            global_clocking.is_some(),
            "Expected global clocking declaration"
        );
        let (id, label) = global_clocking.unwrap();
        assert_eq!(id.as_ref().unwrap(), "my_clk");
        assert_eq!(label.as_ref().unwrap(), "my_clk");
    } else {
        panic!("Expected module declaration");
    }
}

#[test]
fn test_global_clocking_without_identifier() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/global_clocking_without_identifier.sv"),
    )
    .unwrap();

    let result = parser.parse_content(&content).unwrap();

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = item {
        // Find the global clocking item
        let global_clocking = items.iter().find_map(|&item_ref| {
            let item = result.module_item_arena.get(item_ref);
            if let ModuleItem::GlobalClocking { identifier, .. } = item {
                Some(identifier)
            } else {
                None
            }
        });

        assert!(
            global_clocking.is_some(),
            "Expected global clocking declaration"
        );
        assert!(global_clocking.unwrap().is_none(), "Expected no identifier");
    } else {
        panic!("Expected module declaration");
    }
}
