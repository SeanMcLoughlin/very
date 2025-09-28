use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use sv_chumsky::{ModuleItem, SystemVerilogParser};
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_full_pipeline_simple_module() {
    let temp_dir = TempDir::new().unwrap();
    let content = "module simple(input clk, output data); assign data = clk; endmodule";
    let file_path = create_temp_file(&temp_dir, "simple.sv", content);

    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&file_path).unwrap();

    assert_eq!(result.items.len(), 1);
    if let ModuleItem::ModuleDeclaration { name, ports, items } = &result.items[0] {
        assert_eq!(name, "simple");
        assert_eq!(ports.len(), 2);
        assert_eq!(items.len(), 1);
    }
}

#[test]
fn test_full_pipeline_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create file with syntax error
    let content = "module broken syntax error here";
    let file_path = create_temp_file(&temp_dir, "broken.sv", content);

    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&file_path);

    assert!(result.is_err());
}

#[test]
fn test_full_pipeline_missing_include() {
    let temp_dir = TempDir::new().unwrap();

    // Create file that includes a non-existent file
    let content = r#"
`include "nonexistent.sv"
module test; endmodule
"#;
    let file_path = create_temp_file(&temp_dir, "test.sv", content);

    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let result = parser.parse_file(&file_path);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("Include file 'nonexistent.sv' not found"));
}

#[test]
fn test_full_pipeline_file_not_found() {
    let mut parser = SystemVerilogParser::new(vec![], HashMap::new());
    let nonexistent_path = PathBuf::from("/nonexistent/file.sv");

    let result = parser.parse_file(&nonexistent_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Failed to read file"));
}
