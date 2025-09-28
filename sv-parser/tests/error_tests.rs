use std::collections::HashMap;
use sv_parser::SystemVerilogParser;

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
