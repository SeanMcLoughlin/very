use std::path::PathBuf;
use sv_parser::parse_vcs_style_args;

#[test]
fn test_parse_simple_file() {
    let args = vec!["test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(result.files, vec![PathBuf::from("test.sv")]);
    assert_eq!(result.include_dirs, Vec::<PathBuf>::new());
    assert_eq!(result.defines, Vec::<String>::new());
    assert!(!result.verbose);
    assert!(!result.syntax_only);
    assert!(!result.fail_fast);
}

#[test]
fn test_parse_multiple_files() {
    let args = vec!["test1.sv".to_string(), "test2.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(
        result.files,
        vec![PathBuf::from("test1.sv"), PathBuf::from("test2.sv")]
    );
}

#[test]
fn test_parse_incdir_single() {
    let args = vec![
        "+incdir+/path/to/includes".to_string(),
        "test.sv".to_string(),
    ];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(
        result.include_dirs,
        vec![PathBuf::from("/path/to/includes")]
    );
    assert_eq!(result.files, vec![PathBuf::from("test.sv")]);
}

#[test]
fn test_parse_incdir_multiple() {
    let args = vec![
        "+incdir+/path/one".to_string(),
        "+incdir+/path/two".to_string(),
        "test.sv".to_string(),
    ];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(
        result.include_dirs,
        vec![PathBuf::from("/path/one"), PathBuf::from("/path/two")]
    );
}

#[test]
fn test_parse_define_with_value() {
    let args = vec!["+define+DEBUG=1".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(result.defines, vec!["DEBUG=1".to_string()]);
}

#[test]
fn test_parse_define_without_value() {
    let args = vec!["+define+DEBUG".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(result.defines, vec!["DEBUG".to_string()]);
}

#[test]
fn test_parse_define_multiple() {
    let args = vec![
        "+define+DEBUG=1".to_string(),
        "+define+VERBOSE".to_string(),
        "+define+MODE=test".to_string(),
        "test.sv".to_string(),
    ];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    assert_eq!(
        result.defines,
        vec![
            "DEBUG=1".to_string(),
            "VERBOSE".to_string(),
            "MODE=test".to_string()
        ]
    );
}

#[test]
fn test_parse_mixed_args() {
    let args = vec![
        "+incdir+/includes".to_string(),
        "+define+DEBUG=1".to_string(),
        "test1.sv".to_string(),
        "+incdir+/more/includes".to_string(),
        "test2.sv".to_string(),
        "+define+VERBOSE".to_string(),
    ];
    let result = parse_vcs_style_args(args, true, false, false).unwrap();

    assert_eq!(
        result.files,
        vec![PathBuf::from("test1.sv"), PathBuf::from("test2.sv")]
    );
    assert_eq!(
        result.include_dirs,
        vec![PathBuf::from("/includes"), PathBuf::from("/more/includes")]
    );
    assert_eq!(
        result.defines,
        vec!["DEBUG=1".to_string(), "VERBOSE".to_string()]
    );
    assert!(result.verbose);
    assert!(!result.syntax_only);
}

#[test]
fn test_parse_verbose_and_syntax_only() {
    let args = vec!["test.sv".to_string()];
    let result = parse_vcs_style_args(args, true, true, false).unwrap();

    assert!(result.verbose);
    assert!(result.syntax_only);
}

#[test]
fn test_parse_fail_fast() {
    let args = vec!["test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, true).unwrap();

    assert!(result.fail_fast);
    assert!(!result.verbose);
    assert!(!result.syntax_only);
}

#[test]
fn test_parse_empty_incdir_error() {
    let args = vec!["+incdir+".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Empty path in +incdir+ directive");
}

#[test]
fn test_parse_empty_define_error() {
    let args = vec!["+define+".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Empty define in +define+ directive");
}

#[test]
fn test_parse_no_files_error() {
    let args = vec!["+incdir+/includes".to_string()];
    let result = parse_vcs_style_args(args, false, false, false);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No input files specified");
}

#[test]
fn test_parse_unknown_option_error() {
    let args = vec!["--unknown".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unknown option: --unknown");
}

#[test]
fn test_parse_unsupported_vcs_option_warning() {
    let args = vec!["+timescale+1ns/1ps".to_string(), "test.sv".to_string()];
    let result = parse_vcs_style_args(args, false, false, false);

    // Should succeed but warn about unsupported option
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.files, vec![PathBuf::from("test.sv")]);
}

#[test]
fn test_skip_clap_flags() {
    let args = vec![
        "-v".to_string(),
        "--verbose".to_string(),
        "test.sv".to_string(),
    ];
    let result = parse_vcs_style_args(args, false, false, false).unwrap();

    // Should skip the clap flags and just parse the file
    assert_eq!(result.files, vec![PathBuf::from("test.sv")]);
}
