//! Drive strength parsing smoke tests.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};

/// All drive-strength fixtures should parse.
#[test]
fn test_parse_all_drive_strength_files() {
    assert_directory_parses("drive_strengths");
}

sv_ok_tests! {
    drive_strength_strong1_highz0 => "drive_strengths/10.3.4--assignment_strong1_highz0.sv",
    drive_strength_pull1_pull0 => "drive_strengths/10.3.4--assignment_pull1_pull0.sv",
    drive_strength_weak1_weak0 => "drive_strengths/10.3.4--assignment_weak1_weak0.sv",
}

/// Sanity check that at least one drive-strength file parses into a module.
#[test]
fn test_drive_strength_strong1_highz0_structure() {
    let unit = assert_parse_ok("drive_strengths/10.3.4--assignment_strong1_highz0.sv");
    assert!(!unit.items.is_empty());
}
