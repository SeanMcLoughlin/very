//! Procedural block parsing smoke tests with shared helpers.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};

/// Ensure every procedural-block fixture parses successfully.
#[test]
fn test_parse_all_procedural_block_files() {
    assert_directory_parses("procedural_blocks");
}

/// Compound assignment fixtures live under `assignments/`.
#[test]
fn test_compound_assignment_operators() {
    assert_directory_parses("assignments");
}

sv_ok_tests! {
    priority_case => "procedural_blocks/priority_case.sv",
    unique_case => "procedural_blocks/unique_case.sv",
    unique0_case => "procedural_blocks/unique0_case.sv",
    priority_casex => "procedural_blocks/priority_casex.sv",
    unique_casex => "procedural_blocks/unique_casex.sv",
    unique0_casex => "procedural_blocks/unique0_casex.sv",
    priority_casez => "procedural_blocks/priority_casez.sv",
    unique_casez => "procedural_blocks/unique_casez.sv",
    unique0_casez => "procedural_blocks/unique0_casez.sv",
}

/// Example structural check to ensure we still touch the AST helpers when needed.
#[test]
fn test_priority_case_structure() {
    let unit = assert_parse_ok("procedural_blocks/priority_case.sv");
    assert!(!unit.items.is_empty());
}
