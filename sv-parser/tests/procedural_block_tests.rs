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
