use std::collections::HashMap;
use sv_parser::SystemVerilogParser;

fn make_parser(fail_fast: bool) -> SystemVerilogParser {
    SystemVerilogParser::with_config(vec![], HashMap::new(), fail_fast)
}

/// Simulate the CLI behavior of parsing multiple sources with fail-fast logic
fn parse_contents_with_fail_fast(contents: &[&str], fail_fast: bool) -> (Vec<bool>, bool) {
    let mut results = Vec::new();
    let mut had_error = false;

    for content in contents {
        match make_parser(fail_fast).parse_content(content) {
            Ok(_) => {
                results.push(true);
            }
            Err(_) => {
                results.push(false);
                had_error = true;
                if fail_fast {
                    // Stop parsing remaining files
                    break;
                }
            }
        }
    }

    (results, had_error)
}

#[test]
fn test_fail_fast_stops_after_first_error() {
    // Create two test contents: one with error, one valid
    let err_module = r#"module error_test;
  wire w
  // Missing semicolon - syntax error
endmodule
"#;

    let ok_module = r#"module ok_test;
  wire w;
endmodule
"#;

    let contents = [err_module, ok_module];

    // Parse with fail-fast enabled
    let (results, had_error) = parse_contents_with_fail_fast(&contents, true);

    // Should have an error
    assert!(had_error);

    // Should only parse the first module (which failed)
    assert_eq!(results.len(), 1);
    assert!(!results[0]); // First module should fail

    // Verify we can parse the second module successfully if we try
    assert!(make_parser(false).parse_content(ok_module).is_ok());
}

#[test]
fn test_fail_fast() {
    // Create three modules: error, valid, error
    let module1 = r#"module error1;
  wire w
endmodule
"#;

    let module2 = r#"module ok;
  wire w;
endmodule
"#;

    let module3 = r#"module error2;
  wire x
endmodule
"#;

    let contents = [module1, module2, module3];

    // With fail-fast: should stop after first error, so only module1 should fail.
    let (results_fast, had_error_fast) = parse_contents_with_fail_fast(&contents, true);
    assert!(had_error_fast);
    assert_eq!(results_fast.len(), 1); // Only first module should show error.
    assert!(!results_fast[0]);

    // Without fail-fast: should parse all files
    let (results_no_fast, had_error_no_fast) = parse_contents_with_fail_fast(&contents, false);
    assert!(had_error_no_fast);
    assert_eq!(results_no_fast.len(), 3); // All modules attempted
    assert!(!results_no_fast[0]); // Module 1 fails
    assert!(results_no_fast[1]); // Module 2 succeeds
    assert!(!results_no_fast[2]); // Module 3 fails
}

#[test]
fn test_fail_fast_with_all_legal_code() {
    // Create two valid modules
    let module1 = r#"module test1;
  wire w;
endmodule
"#;

    let module2 = r#"module test2;
  wire x;
endmodule
"#;

    let modules = [module1, module2];

    // Parse with fail-fast enabled
    let (results, had_error) = parse_contents_with_fail_fast(&modules, true);

    // Should not have any errors
    assert!(!had_error);

    // Should parse both modules successfully
    assert_eq!(results.len(), 2);
    assert!(results[0]);
    assert!(results[1]);
}
