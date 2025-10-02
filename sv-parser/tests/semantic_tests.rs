//! Semantic analysis tests
//!
//! Tests for semantic validation that goes beyond syntax checking

use std::collections::HashMap;
use sv_parser::{SemanticAnalyzer, SemanticErrorType, SystemVerilogParser};

#[test]
fn test_unknown_system_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    logic a;
    initial begin
        a = $unknown_func(1);
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        SemanticErrorType::UnknownSystemFunction
    );
    assert!(errors[0].message.contains("unknown_func"));
}

#[test]
fn test_typo_in_system_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    logic a;
    initial begin
        a = $fel(1);  // typo: should be $fell
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        SemanticErrorType::UnknownSystemFunction
    );
    assert!(errors[0].message.contains("fel"));
}

#[test]
fn test_valid_system_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    logic a;
    initial begin
        a = $fell(1);
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 0, "Should have no semantic errors");
}

#[test]
fn test_multiple_system_functions() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    logic a, b, c;
    initial begin
        a = $rose(b);
        c = $fell(a);
        b = $stable(c);
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 0, "All system functions should be valid");
}

#[test]
fn test_unknown_system_task() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    initial begin
        $unknown_task();
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        SemanticErrorType::UnknownSystemFunction
    );
    assert!(errors[0].message.contains("unknown_task"));
}

#[test]
fn test_valid_system_tasks() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    initial begin
        $display("Hello");
        $finish;
        $error("Error message");
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 0, "All system tasks should be valid");
}

#[test]
fn test_math_functions() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    real x;
    real y;
    initial begin
        x = $sin(3);
        y = $cos(1);
        x = $sqrt(4);
        y = $ln(2);
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 0, "All math functions should be valid");
}

#[test]
fn test_nested_expressions_with_errors() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
module top();
    logic a, b;
    initial begin
        a = $sin($unknown(b));
    end
endmodule
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("unknown"));
}

#[test]
fn test_class_method_with_system_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
class MyClass;
    logic value;

    function void compute();
        value = $clog2(256);
    endfunction
endclass
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(
        errors.len(),
        0,
        "Should accept valid system function in class method"
    );
}

#[test]
fn test_class_method_with_invalid_function() {
    let parser = SystemVerilogParser::new(vec![], HashMap::new());
    let content = r#"
class MyClass;
    logic value;

    function void compute();
        value = $invalid_func(256);
    endfunction
endclass
"#;

    let ast = parser.parse_content(content).unwrap();
    let errors = parser.analyze_semantics(&ast);

    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("invalid_func"));
}
