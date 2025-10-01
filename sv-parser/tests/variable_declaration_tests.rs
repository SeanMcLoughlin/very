use std::collections::HashMap;
use sv_parser::SystemVerilogParser;

#[test]
fn test_wire_declaration() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
wire a;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse wire declaration: {:?}", result.err());
}

#[test]
fn test_wire_declaration_with_width() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
wire [7:0] data;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse wire declaration with width: {:?}", result.err());
}

#[test]
fn test_wire_declaration_with_init() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
wire [7:0] a = 8'b1101x001;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse wire declaration with initialization: {:?}", result.err());
}

#[test]
fn test_logic_declaration() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
logic clk;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse logic declaration: {:?}", result.err());
}

#[test]
fn test_int_declaration() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
int count;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse int declaration: {:?}", result.err());
}

#[test]
fn test_int_declaration_with_init() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
int a = 12;
int b = 5;
endmodule
";

    let result = parser.parse_content(content);
    assert!(result.is_ok(), "Failed to parse int declaration with initialization: {:?}", result.err());
}
