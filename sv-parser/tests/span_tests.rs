//! Span tracking tests
//!
//! This module tests that the parser correctly tracks source spans for various
//! language constructs. This is essential for features like code folding,
//! go-to-definition, and other IDE functionality.

use std::collections::HashMap;
use sv_parser::{ModuleItem, SystemVerilogParser};

/// Test that module declarations have correct spans
#[test]
fn test_module_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\n    logic a;\nendmodule";
    //            0123456789012345678901234567890123456789
    //            0         1         2         3

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 1);

    let item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration {
        name,
        span,
        name_span,
        ..
    } = item
    {
        assert_eq!(name, "test");
        // The module spans from 'm' in 'module' to 'e' in 'endmodule'
        assert_eq!(span.0, 0, "Module should start at position 0");
        assert_eq!(span.1, content.len(), "Module should end at end of content");

        // The name_span should cover just "test"
        assert_eq!(
            &content[name_span.0..name_span.1],
            "test",
            "Name span should cover 'test'"
        );
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that nested module items have correct spans
#[test]
fn test_nested_variable_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\n    logic a;\n    logic b;\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 2, "Module should have 2 variable declarations");

        // Check first variable
        let var1 = result.module_item_arena.get(items[0]);
        if let ModuleItem::VariableDeclaration {
            name,
            name_span,
            span,
            ..
        } = var1
        {
            assert_eq!(name, "a");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "a",
                "First var name span should cover 'a'"
            );
            assert!(
                &content[span.0..span.1].contains("logic a"),
                "First var span should contain 'logic a'"
            );
        } else {
            panic!("Expected variable declaration for first item");
        }

        // Check second variable
        let var2 = result.module_item_arena.get(items[1]);
        if let ModuleItem::VariableDeclaration {
            name,
            name_span,
            span,
            ..
        } = var2
        {
            assert_eq!(name, "b");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "b",
                "Second var name span should cover 'b'"
            );
            assert!(
                &content[span.0..span.1].contains("logic b"),
                "Second var span should contain 'logic b'"
            );
        } else {
            panic!("Expected variable declaration for second item");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that class declarations have correct spans
#[test]
fn test_class_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top;\nclass MyClass;\n    int x;\nendclass\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 class declaration");

        let class_item = result.module_item_arena.get(items[0]);
        if let ModuleItem::ClassDeclaration {
            name,
            name_span,
            span,
            ..
        } = class_item
        {
            assert_eq!(name, "MyClass");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "MyClass",
                "Class name span should cover 'MyClass'"
            );
            assert!(
                &content[span.0..span.1].contains("class MyClass"),
                "Class span should contain 'class MyClass'"
            );
            assert!(
                &content[span.0..span.1].contains("endclass"),
                "Class span should contain 'endclass'"
            );
        } else {
            panic!("Expected class declaration");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that procedural blocks have correct spans
#[test]
fn test_procedural_block_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\nalways_comb begin\n    a = b;\nend\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 procedural block");

        let proc_block = result.module_item_arena.get(items[0]);
        if let ModuleItem::ProceduralBlock { span, .. } = proc_block {
            assert!(
                &content[span.0..span.1].contains("always_comb"),
                "Procedural block span should contain 'always_comb'"
            );
            assert!(
                &content[span.0..span.1].contains("a = b"),
                "Procedural block span should contain 'a = b'"
            );
        } else {
            panic!("Expected procedural block");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that assignment statements have correct spans
#[test]
fn test_assignment_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\nassign a = b + c;\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 assignment");

        let assign_item = result.module_item_arena.get(items[0]);
        if let ModuleItem::Assignment { span, .. } = assign_item {
            assert!(
                &content[span.0..span.1].contains("assign"),
                "Assignment span should contain 'assign'"
            );
            assert!(
                &content[span.0..span.1].contains("a = b + c"),
                "Assignment span should contain 'a = b + c'"
            );
        } else {
            panic!("Expected assignment");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that port declarations have correct spans
#[test]
fn test_port_declaration_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\ninput logic clk;\noutput logic data;\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 2, "Module should have 2 port declarations");

        // Check input port
        let port1 = result.module_item_arena.get(items[0]);
        if let ModuleItem::PortDeclaration {
            name,
            name_span,
            span,
            ..
        } = port1
        {
            assert_eq!(name, "clk");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "clk",
                "Port name span should cover 'clk'"
            );
            assert!(
                &content[span.0..span.1].contains("input logic clk"),
                "Port span should contain 'input logic clk'"
            );
        } else {
            panic!("Expected port declaration for first item");
        }

        // Check output port
        let port2 = result.module_item_arena.get(items[1]);
        if let ModuleItem::PortDeclaration {
            name,
            name_span,
            span,
            ..
        } = port2
        {
            assert_eq!(name, "data");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "data",
                "Port name span should cover 'data'"
            );
            assert!(
                &content[span.0..span.1].contains("output logic"),
                "Port span should contain 'output logic'"
            );
        } else {
            panic!("Expected port declaration for second item");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that preprocessor directives have correct spans
#[test]
fn test_define_directive_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\n`define FOO 123\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 define directive");

        let define_item = result.module_item_arena.get(items[0]);
        if let ModuleItem::DefineDirective {
            name,
            name_span,
            span,
            ..
        } = define_item
        {
            assert_eq!(name, "FOO");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "FOO",
                "Define name span should cover 'FOO'"
            );
            assert!(
                &content[span.0..span.1].contains("`define FOO"),
                "Define span should contain '`define FOO'"
            );
        } else {
            panic!("Expected define directive");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test that global clocking blocks have correct spans
#[test]
fn test_global_clocking_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\nglobal clocking cb @(posedge clk); endclocking\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 global clocking block");

        let clocking_item = result.module_item_arena.get(items[0]);
        if let ModuleItem::GlobalClocking { span, .. } = clocking_item {
            assert!(
                &content[span.0..span.1].contains("global clocking"),
                "Clocking span should contain 'global clocking'"
            );
            assert!(
                &content[span.0..span.1].contains("endclocking"),
                "Clocking span should contain 'endclocking'"
            );
        } else {
            panic!("Expected global clocking block");
        }
    } else {
        panic!("Expected module declaration");
    }
}

/// Test span tracking with multiple modules
#[test]
fn test_multiple_module_spans() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module first;\nendmodule\n\nmodule second;\n    logic x;\nendmodule";

    let result = parser.parse_content(content).unwrap();
    assert_eq!(result.items.len(), 2, "Should have 2 modules");

    // Check first module
    let module1 = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { name, span, .. } = module1 {
        assert_eq!(name, "first");
        assert!(
            &content[span.0..span.1].contains("module first"),
            "First module span should contain 'module first'"
        );
        assert!(
            &content[span.0..span.1].contains("endmodule"),
            "First module span should contain 'endmodule'"
        );
        // Make sure first module span doesn't include second module
        assert!(
            !&content[span.0..span.1].contains("module second"),
            "First module span should not contain second module"
        );
    } else {
        panic!("Expected first module declaration");
    }

    // Check second module
    let module2 = result.module_item_arena.get(result.items[1]);
    if let ModuleItem::ModuleDeclaration { name, span, .. } = module2 {
        assert_eq!(name, "second");
        assert!(
            &content[span.0..span.1].contains("module second"),
            "Second module span should contain 'module second'"
        );
        assert!(
            &content[span.0..span.1].contains("logic x"),
            "Second module span should contain 'logic x'"
        );
        // Make sure second module span doesn't include first module
        assert!(
            !&content[span.0..span.1].contains("module first"),
            "Second module span should not contain first module"
        );
    } else {
        panic!("Expected second module declaration");
    }
}

/// Test that variable declarations with initial values have correct spans
#[test]
fn test_variable_with_initial_value_span() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module test;\nlogic data = 1;\nendmodule";

    let result = parser.parse_content(content).unwrap();

    let module_item = result.module_item_arena.get(result.items[0]);
    if let ModuleItem::ModuleDeclaration { items, .. } = module_item {
        assert_eq!(items.len(), 1, "Module should have 1 variable declaration");

        let var_item = result.module_item_arena.get(items[0]);
        if let ModuleItem::VariableDeclaration {
            name,
            name_span,
            span,
            ..
        } = var_item
        {
            assert_eq!(name, "data");
            assert_eq!(
                &content[name_span.0..name_span.1],
                "data",
                "Variable name span should cover 'data'"
            );
            assert!(
                &content[span.0..span.1].contains("logic data"),
                "Variable span should contain 'logic data'"
            );
            assert!(
                &content[span.0..span.1].contains("= 1"),
                "Variable span should contain initial value"
            );
        } else {
            panic!("Expected variable declaration");
        }
    } else {
        panic!("Expected module declaration");
    }
}
