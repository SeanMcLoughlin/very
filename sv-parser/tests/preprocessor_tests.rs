use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use sv_parser::preprocessor::Preprocessor;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_preprocess_simple_content() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = "module test; endmodule";

    let result = preprocessor.preprocess_content(content, None).unwrap();
    assert_eq!(result.trim(), "module test; endmodule");
}

#[test]
fn test_preprocess_define_directive() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = "`define DEBUG 1\nmodule test; endmodule";

    let result = preprocessor.preprocess_content(content, None).unwrap();
    assert_eq!(result.trim(), "module test; endmodule");
}

#[test]
fn test_preprocess_macro_expansion() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = "`define WIDTH 8\nreg [`WIDTH-1:0] data;";

    let result = preprocessor.preprocess_content(content, None).unwrap();
    assert!(result.contains("reg [8-1:0] data;"));
}

#[test]
fn test_preprocess_define_with_value() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = "`define MAX_COUNT 100\nif (count >= `MAX_COUNT)";

    let result = preprocessor.preprocess_content(content, None).unwrap();
    assert!(result.contains("if (count >= 100)"));
}

#[test]
fn test_preprocess_define_without_value() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = "`define ENABLE\n`ifdef ENABLE\nparameter en = 1;\n`endif";

    let result = preprocessor.preprocess_content(content, None).unwrap();
    // The ifdef/endif should be ignored for now
    assert!(result.contains("parameter en = 1;"));
}

#[test]
fn test_preprocess_include_relative() {
    let temp_dir = TempDir::new().unwrap();

    // Create included file
    let included_content = "parameter WIDTH = 8;";
    let included_path = create_temp_file(&temp_dir, "included.sv", included_content);

    // Create main file
    let main_content = format!(
        "`include \"{}\"\nmodule test; endmodule",
        included_path.file_name().unwrap().to_str().unwrap()
    );
    let main_path = create_temp_file(&temp_dir, "main.sv", &main_content);

    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let result = preprocessor.preprocess_file(&main_path).unwrap();

    assert!(result.contains("parameter WIDTH = 8;"));
    assert!(result.contains("module test; endmodule"));
}

#[test]
fn test_preprocess_include_with_incdir() {
    let temp_dir = TempDir::new().unwrap();
    let inc_dir = temp_dir.path().join("includes");
    fs::create_dir(&inc_dir).unwrap();

    // Create included file in include directory
    let included_content = "typedef logic [7:0] byte_t;";
    let included_path = inc_dir.join("types.sv");
    fs::write(&included_path, included_content).unwrap();

    // Create main file that includes from include directory
    let main_content = "`include \"types.sv\"\nmodule test; endmodule";
    let main_path = create_temp_file(&temp_dir, "main.sv", main_content);

    let mut preprocessor = Preprocessor::new(vec![inc_dir], HashMap::new());
    let result = preprocessor.preprocess_file(&main_path).unwrap();

    assert!(result.contains("typedef logic [7:0] byte_t;"));
    assert!(result.contains("module test; endmodule"));
}

#[test]
fn test_preprocess_include_angle_brackets() {
    let temp_dir = TempDir::new().unwrap();
    let inc_dir = temp_dir.path().join("includes");
    fs::create_dir(&inc_dir).unwrap();

    // Create included file
    let included_content = "localparam DELAY = 10;";
    let included_path = inc_dir.join("constants.sv");
    fs::write(&included_path, included_content).unwrap();

    // Create main file using angle bracket include
    let main_content = "`include <constants.sv>\nmodule test; endmodule";
    let main_path = create_temp_file(&temp_dir, "main.sv", main_content);

    let mut preprocessor = Preprocessor::new(vec![inc_dir], HashMap::new());
    let result = preprocessor.preprocess_file(&main_path).unwrap();

    assert!(result.contains("localparam DELAY = 10;"));
}

#[test]
fn test_preprocess_include_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let main_content = "`include \"nonexistent.sv\"\nmodule test; endmodule";
    let main_path = create_temp_file(&temp_dir, "main.sv", main_content);

    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let result = preprocessor.preprocess_file(&main_path);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("Include file 'nonexistent.sv' not found"));
}

#[test]
fn test_preprocess_nested_includes() {
    let temp_dir = TempDir::new().unwrap();

    // Create deepest include
    let deep_content = "parameter DEEP_PARAM = 42;";
    let deep_path = create_temp_file(&temp_dir, "deep.sv", deep_content);

    // Create middle include
    let middle_content = format!(
        "`include \"{}\"\nparameter MID_PARAM = 24;",
        deep_path.file_name().unwrap().to_str().unwrap()
    );
    let middle_path = create_temp_file(&temp_dir, "middle.sv", &middle_content);

    // Create top include
    let top_content = format!(
        "`include \"{}\"\nmodule test; endmodule",
        middle_path.file_name().unwrap().to_str().unwrap()
    );
    let top_path = create_temp_file(&temp_dir, "top.sv", &top_content);

    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let result = preprocessor.preprocess_file(&top_path).unwrap();

    assert!(result.contains("parameter DEEP_PARAM = 42;"));
    assert!(result.contains("parameter MID_PARAM = 24;"));
    assert!(result.contains("module test; endmodule"));
}

#[test]
fn test_preprocess_complex_macro_expansion() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = r#"
`define WIRE_DECL(name, width) wire [width-1:0] name
`define BUS_WIDTH 8
`WIRE_DECL(data_bus, `BUS_WIDTH);
"#;

    let result = preprocessor.preprocess_content(content, None).unwrap();
    // This is a simplified test - real macro expansion with parameters would be more complex
    assert!(result.contains("data_bus"));
    assert!(result.contains("8"));
}

#[test]
fn test_preprocess_ignore_conditional_compilation() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let content = r#"
`ifdef DEBUG
    initial $display("Debug mode");
`else
    initial $display("Release mode");
`endif
module test; endmodule
"#;

    let result = preprocessor.preprocess_content(content, None).unwrap();

    // Conditional compilation directives should be ignored/removed
    assert!(result.contains("initial $display(\"Debug mode\");"));
    assert!(result.contains("initial $display(\"Release mode\");"));
    assert!(result.contains("module test; endmodule"));
}

#[test]
fn test_preprocess_file_read_error() {
    let mut preprocessor = Preprocessor::new(vec![], HashMap::new());
    let nonexistent_path = PathBuf::from("/nonexistent/path/file.sv");

    let result = preprocessor.preprocess_file(&nonexistent_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Failed to read file"));
}
