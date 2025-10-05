//! Module-related tests using shared fixture utilities.
//!
//! Exercises the parser against module fixtures under `test_files/modules` and
//! verifies selected AST details.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};
use sv_parser::{ModuleItem, PortDirection};

/// Ensure all module fixtures parse without error.
#[test]
fn test_parse_all_module_files() {
    assert_directory_parses("modules");
}

sv_ok_tests! {
    module_empty => "modules/empty_module.sv",
    module_with_ports_fixture => "modules/module_with_ports.sv",
    module_with_array_ports_fixture => "modules/module_with_array_ports.sv",
    module_multiple => "modules/multiple_modules.sv",
    module_port_decl => "modules/module_with_port_declaration.sv",
    module_no_dir_ports => "modules/module_no_direction_ports.sv",
    module_whitespace => "modules/whitespace_handling.sv",
}

/// Empty module fixture should produce a single declaration with no ports/items.
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

/// Module with named ports should track direction metadata.
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

        assert_eq!(ports[0].name, "clk");
        assert_eq!(ports[0].direction, Some(PortDirection::Input));

        assert_eq!(ports[1].name, "data");
        assert_eq!(ports[1].direction, Some(PortDirection::Output));
    } else {
        panic!("Expected module declaration");
    }
}

/// Module with array ports should capture declared ranges.
#[test]
fn test_module_with_array_ports_structure() {
    let result = assert_parse_ok("modules/module_with_array_ports.sv");

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { name, ports, .. } = item {
        assert_eq!(name, "test");
        assert_eq!(ports.len(), 2);

        assert_eq!(ports[0].name, "a");
        assert_eq!(ports[0].direction, Some(PortDirection::Input));
        if let Some(ref range) = ports[0].range {
            assert_eq!(range.msb, "3");
            assert_eq!(range.lsb, "0");
        } else {
            panic!("Expected range for port a");
        }

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

/// Multiple modules in one file should be surfaced individually.
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
