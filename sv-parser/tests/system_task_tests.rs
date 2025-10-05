//! System task and function parsing smoke tests.

#[path = "common/mod.rs"]
mod common;

use common::{assert_directory_parses, assert_parse_ok};

/// Ensure every `system_tasks` fixture parses successfully.
#[test]
fn test_parse_all_system_task_files() {
    assert_directory_parses("system_tasks");
}

sv_ok_tests! {
    atan_function => "system_tasks/atan_function.sv",
    sin_function => "system_tasks/sin_function.sv",
    cos_function => "system_tasks/cos_function.sv",
    sampled_rose => "sampled_rose.sv",
    sampled_fell => "sampled_fell.sv",
    sampled_stable => "sampled_stable.sv",
    sampled_past => "sampled_past.sv",
}

/// Quick structural assertion for one of the sampled-value helpers.
#[test]
fn test_sampled_past_parses() {
    assert_parse_ok("sampled_past.sv");
}
