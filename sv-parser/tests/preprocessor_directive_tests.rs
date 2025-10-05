use std::collections::HashMap;
use std::fs;
use sv_parser::{ModuleItem, SystemVerilogParser};

#[test]
fn test_define_directive_simple() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
`define WORDSIZE 8
module test; endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    assert!(
        ast.items.len() >= 2,
        "Should have at least define and module"
    );

    // Check the define directive
    let item = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::DefineDirective {
        name,
        value,
        parameters,
        ..
    } = item
    else {
        panic!("Expected DefineDirective, got {:?}", item);
    };

    assert_eq!(name, "WORDSIZE");
    assert_eq!(value, "8");
    assert!(parameters.is_empty());
}

#[test]
fn test_define_directive_with_parameters() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
`define MAX(a, b) ((a) > (b) ? (a) : (b))
module test; endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    let item = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::DefineDirective {
        name,
        value,
        parameters,
        ..
    } = item
    else {
        panic!("Expected DefineDirective");
    };

    assert_eq!(name, "MAX");
    assert_eq!(parameters.len(), 2);
    assert_eq!(parameters[0], "a");
    assert_eq!(parameters[1], "b");
    assert!(value.contains("((a) > (b) ? (a) : (b))"));
}

#[test]
fn test_include_directive() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
`include "test.sv"
module test; endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    let item = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::IncludeDirective { path, .. } = item else {
        panic!("Expected IncludeDirective");
    };

    assert_eq!(path, "test.sv");
}

#[test]
fn test_include_directive_angle_brackets() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
`include <stdlib.sv>
module test; endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    let item = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::IncludeDirective { path, .. } = item else {
        panic!("Expected IncludeDirective");
    };

    assert_eq!(path, "stdlib.sv");
}

#[test]
fn test_multiple_directives() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
`define WIDTH 32
`define HEIGHT 16
`include "common.sv"
module test; endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    assert_eq!(
        ast.items.len(),
        4,
        "Should have 2 defines, 1 include, and 1 module"
    );

    // First define
    let item0 = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::DefineDirective { name, .. } = item0 else {
        panic!("Expected DefineDirective");
    };
    assert_eq!(name, "WIDTH");

    // Second define
    let item1 = ast.module_item_arena.get(ast.items[1]);
    let ModuleItem::DefineDirective { name, .. } = item1 else {
        panic!("Expected DefineDirective");
    };
    assert_eq!(name, "HEIGHT");

    // Include
    let item2 = ast.module_item_arena.get(ast.items[2]);
    let ModuleItem::IncludeDirective { path, .. } = item2 else {
        panic!("Expected IncludeDirective");
    };
    assert_eq!(path, "common.sv");
}

#[test]
fn test_directives_inside_module() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());

    let content = r#"
module test;
    `define LOCAL_VAR 1
    `include "module_helper.sv"
endmodule
"#;

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Parse should succeed");

    let ast = result.unwrap();
    let item = ast.module_item_arena.get(ast.items[0]);
    let ModuleItem::ModuleDeclaration { items, .. } = item else {
        panic!("Expected ModuleDeclaration");
    };

    assert_eq!(
        items.len(),
        2,
        "Should have define and include inside module"
    );

    let item0 = ast.module_item_arena.get(items[0]);
    let ModuleItem::DefineDirective { name, .. } = item0 else {
        panic!("Expected DefineDirective inside module");
    };
    assert_eq!(name, "LOCAL_VAR");

    let item1 = ast.module_item_arena.get(items[1]);
    let ModuleItem::IncludeDirective { path, .. } = item1 else {
        panic!("Expected IncludeDirective inside module");
    };
    assert_eq!(path, "module_helper.sv");
}

#[test]
fn test_include_path_resolution() {
    // Create temporary directory and files
    let temp_dir = std::env::temp_dir().join("sv_parser_test_include_resolution");
    let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
    fs::create_dir_all(&temp_dir).unwrap();

    // Create an include file
    let include_file = temp_dir.join("included.sv");
    fs::write(&include_file, "// Included content\n").unwrap();

    // Create main file that includes it
    let main_file = temp_dir.join("main.sv");
    fs::write(
        &main_file,
        r#"`include "included.sv"
module test; endmodule
"#,
    )
    .unwrap();

    // Parse the main file
    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&main_file);

    if let Err(ref e) = result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(result.is_ok(), "Parse should succeed");
    let ast = result.unwrap();

    // Includes are now expanded - the IncludeDirective is replaced with the file contents
    // The included file has a comment, so we should have at least the module
    let module_count = ast
        .items
        .iter()
        .filter(|&item_ref| {
            let item = ast.module_item_arena.get(*item_ref);
            matches!(item, ModuleItem::ModuleDeclaration { .. })
        })
        .count();

    assert_eq!(module_count, 1, "Should have the test module");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_include_from_include_dir() {
    // Create temporary directories
    let temp_dir = std::env::temp_dir().join("sv_parser_test_include_dir");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let inc_dir = temp_dir.join("include");
    fs::create_dir_all(&inc_dir).unwrap();

    // Create an include file in the include directory
    let include_file = inc_dir.join("lib.sv");
    fs::write(&include_file, "// Library content\n").unwrap();

    // Create main file in a different directory
    let main_file = temp_dir.join("main.sv");
    fs::write(
        &main_file,
        r#"`include "lib.sv"
module test; endmodule
"#,
    )
    .unwrap();

    // Parse with include directory specified
    let mut parser = SystemVerilogParser::new(vec![inc_dir.clone()], HashMap::new());
    let result = parser.parse_file(&main_file);

    assert!(result.is_ok(), "Parse should succeed");
    let ast = result.unwrap();

    // Includes are now expanded - the IncludeDirective is replaced with the file contents
    // The included file has a comment, so we should have at least the module
    let module_count = ast
        .items
        .iter()
        .filter(|&item_ref| {
            let item = ast.module_item_arena.get(*item_ref);
            matches!(item, ModuleItem::ModuleDeclaration { .. })
        })
        .count();

    assert_eq!(module_count, 1, "Should have the test module");

    // No need to check resolved_path since includes are expanded
    /*assert_eq!(
        resolved_path.as_ref().unwrap().canonicalize().unwrap(),
        include_file.canonicalize().unwrap(),
        "Should find include in specified include directory"
    );*/

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_with_includes_recursive() {
    // Create temporary directory structure
    let temp_dir = std::env::temp_dir().join("sv_parser_test_recursive_includes");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Create a chain of includes: main.sv -> header.sv -> defs.sv

    // defs.sv (bottom of the chain)
    let defs_file = temp_dir.join("defs.sv");
    fs::write(
        &defs_file,
        r#"`define DEPTH 32
module defs_module;
endmodule
"#,
    )
    .unwrap();

    // header.sv (middle of the chain)
    let header_file = temp_dir.join("header.sv");
    fs::write(
        &header_file,
        r#"`include "defs.sv"
`define WIDTH 16
module header_module;
endmodule
"#,
    )
    .unwrap();

    // main.sv (top of the chain)
    let main_file = temp_dir.join("main.sv");
    fs::write(
        &main_file,
        r#"`include "header.sv"
module main_module;
endmodule
"#,
    )
    .unwrap();

    // Parse with includes expanded
    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&main_file);

    assert!(result.is_ok(), "Recursive parse should succeed");
    let ast = result.unwrap();

    // Check that we got items from all three files
    // We should have 2 defines and 3 modules
    let define_count = ast
        .items
        .iter()
        .filter(|&item_ref| {
            let item = ast.module_item_arena.get(*item_ref);
            matches!(item, ModuleItem::DefineDirective { .. })
        })
        .count();
    let module_count = ast
        .items
        .iter()
        .filter(|&item_ref| {
            let item = ast.module_item_arena.get(*item_ref);
            matches!(item, ModuleItem::ModuleDeclaration { .. })
        })
        .count();

    assert_eq!(define_count, 2, "Should have 2 defines (DEPTH, WIDTH)");
    assert_eq!(
        module_count, 3,
        "Should have 3 modules (defs_module, header_module, main_module)"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_with_circular_includes() {
    // Create temporary directory
    let temp_dir = std::env::temp_dir().join("sv_parser_test_circular_includes");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Create circular includes: a.sv -> b.sv -> a.sv

    // a.sv
    let a_file = temp_dir.join("a.sv");
    fs::write(
        &a_file,
        r#"`include "b.sv"
module a_module;
endmodule
"#,
    )
    .unwrap();

    // b.sv
    let b_file = temp_dir.join("b.sv");
    fs::write(
        &b_file,
        r#"`include "a.sv"
module b_module;
endmodule
"#,
    )
    .unwrap();

    // Parse with includes - should handle circular includes gracefully
    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&a_file);

    // Should succeed (circular includes are detected and skipped)
    assert!(result.is_ok(), "Should handle circular includes gracefully");
    let ast = result.unwrap();

    // Should have both modules (each included once)
    let module_count = ast
        .items
        .iter()
        .filter(|&item_ref| {
            let item = ast.module_item_arena.get(*item_ref);
            matches!(item, ModuleItem::ModuleDeclaration { .. })
        })
        .count();

    assert_eq!(
        module_count, 2,
        "Should have both modules despite circular includes"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
