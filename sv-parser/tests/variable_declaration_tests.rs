//! Variable declaration parsing smoke tests using shared helpers.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};

/// Ensure every variable fixture parses successfully.
#[test]
fn test_parse_all_variable_files() {
    assert_directory_parses("variables");
}

sv_ok_tests! {
    time_unsigned => "variables/time_unsigned.sv",
    time_signed => "variables/time_signed.sv",
    int_unsigned => "variables/int_unsigned.sv",
    byte_unsigned => "variables/byte_unsigned.sv",
    shortint_unsigned => "variables/shortint_unsigned.sv",
    longint_unsigned => "variables/longint_unsigned.sv",
    bit_signed => "variables/bit_signed.sv",
    integer_unsigned => "variables/integer_unsigned.sv",
    integer_signed => "variables/integer_signed.sv",
    net_trireg => "variables/trireg_declaration.sv",
    net_uwire => "variables/uwire_declaration.sv",
    net_wand => "variables/wand_declaration.sv",
    net_wor => "variables/wor_declaration.sv",
    net_tri => "variables/tri_declaration.sv",
    net_triand => "variables/triand_declaration.sv",
    net_trior => "variables/trior_declaration.sv",
    net_tri0 => "variables/tri0_declaration.sv",
    net_tri1 => "variables/tri1_declaration.sv",
}

/// Sample structural assertion to ensure helper usage stays easy to adopt.
#[test]
fn test_time_unsigned_structure() {
    let unit = assert_parse_ok("variables/time_unsigned.sv");
    assert!(!unit.items.is_empty(), "Expected at least one declaration");
}
