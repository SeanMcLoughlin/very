use std::collections::HashMap;
use sv_parser::SystemVerilogParser;

#[test]
fn test_initial_block_empty() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
initial begin
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse empty initial block: {:?}",
        result.err()
    );
}

#[test]
fn test_initial_block_with_assignment() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
int a = 12;
int b = 5;
initial begin
    a = ~^b;
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse initial block with assignment: {:?}",
        result.err()
    );
}

#[test]
fn test_final_block() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
wire c;
final begin
    $display(c);
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse final block: {:?}",
        result.err()
    );
}

#[test]
fn test_always_comb_block() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
logic a, b, c;
always_comb begin
    c = a & b;
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse always_comb block: {:?}",
        result.err()
    );
}

#[test]
fn test_always_ff_block() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
logic clk, q, d;
always_ff @(posedge clk) begin
    q = d;
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse always_ff block: {:?}",
        result.err()
    );
}

#[test]
fn test_always_block() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
logic a, b;
always @(a) begin
    b = a;
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse always block: {:?}",
        result.err()
    );
}

#[test]
fn test_display_with_complex_format_string() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = "module top();
wire [7:0] a = 8'b1101x001;
wire [7:0] b = 8'b1101x001;
wire c;
assign a = 8'b1101x001;
assign b = 8'b1101x001;
assign c = a == b;
final begin
    $display(\":assert: ('%s' == '%d')\", \"x\", c);
end
endmodule
";

    let result = parser.parse_content(content);
    assert!(
        result.is_ok(),
        "Failed to parse display with complex format string: {:?}",
        result.err()
    );
}

#[test]
fn test_compound_assignment_operators() {
    use std::path::Path;

    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let test_files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/assignments");

    for entry in std::fs::read_dir(&test_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sv") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            println!("Testing assignment file: {}", filename);

            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            parser
                .parse_content(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));

            println!("  ✅ Parsed successfully");
        }
    }
}
