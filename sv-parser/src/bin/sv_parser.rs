#!/usr/bin/env rust
use clap::Parser;
use std::process;
use sv_parser::{parse_vcs_style_args, SystemVerilogParser};

#[derive(Parser)]
#[command(name = "sv_parser")]
#[command(about = "Parser for very -- the SystemVerilog Language Server")]
#[command(version)]
#[command(disable_help_flag = true)]
struct Cli {
    /// All arguments (mix of +incdir+ options and files)
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,

    /// Show help information
    #[arg(long = "help", short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Verbose output (show parsed AST)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Only check syntax without elaboration
    #[arg(short = 's', long = "syntax-only")]
    syntax_only: bool,

    /// Stop parsing after the first error
    #[arg(long = "fail-fast")]
    fail_fast: bool,
}

fn main() {
    let cli_args = Cli::parse();

    let parsed_args = match parse_vcs_style_args(
        cli_args.args,
        cli_args.verbose,
        cli_args.syntax_only,
        cli_args.fail_fast,
    ) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!();
            eprintln!("Usage: sv-parser [OPTIONS] [+incdir+<path>]... [+define+<macro>[=<value>]]... <file>...");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  -v, --verbose        Verbose output (show parsed AST)");
            eprintln!("  -s, --syntax-only    Only check syntax without elaboration");
            eprintln!("      --fail-fast      Stop parsing after the first error");
            eprintln!("  -h, --help           Show this help message");
            eprintln!();
            eprintln!("VCS-style options:");
            eprintln!("  +incdir+<path>       Add include directory for `include directives");
            eprintln!("  +define+<macro>=<val> Define preprocessor macro");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  sv-parser design.sv");
            eprintln!("  sv-parser +incdir+/my/includes design.sv testbench.sv");
            eprintln!("  sv-parser +incdir+inc +define+DEBUG=1 design.sv");
            process::exit(1);
        }
    };

    if parsed_args.verbose {
        if !parsed_args.include_dirs.is_empty() {
            eprintln!("Include directories: {:?}", parsed_args.include_dirs);
        }
        if !parsed_args.defines.is_empty() {
            eprintln!("Macro defines: {:?}", parsed_args.defines);
        }
        eprintln!("Files to parse: {:?}", parsed_args.files);
    }

    let mut had_errors = false;

    // Setup common parsing parameters
    let include_paths = parsed_args.include_dirs.clone();
    let mut initial_macros = std::collections::HashMap::new();

    // Convert defines to initial macros
    for define in &parsed_args.defines {
        if let Some(eq_pos) = define.find('=') {
            let name = define[..eq_pos].to_string();
            let value = define[eq_pos + 1..].to_string();
            initial_macros.insert(name, value);
        } else {
            // Define without value (empty macro)
            initial_macros.insert(define.clone(), String::new());
        }
    }

    for file_path in &parsed_args.files {
        if parsed_args.verbose {
            eprintln!("Parsing file: {}", file_path.display());
        }

        // Create a new parser instance for each file
        let mut parser = if parsed_args.fail_fast {
            SystemVerilogParser::with_config(include_paths.clone(), initial_macros.clone(), true)
        } else {
            SystemVerilogParser::new(include_paths.clone(), initial_macros.clone())
        };

        match parser.parse_file(file_path) {
            Ok(ast) => {
                // Perform semantic analysis
                let semantic_errors = parser.analyze_semantics(&ast);

                if !semantic_errors.is_empty() {
                    // Report semantic errors
                    eprintln!("Semantic errors in {}:", file_path.display());
                    for error in &semantic_errors {
                        eprintln!(
                            "  Error at {}:{}: {}",
                            error.span.0, error.span.1, error.message
                        );
                    }
                    had_errors = true;
                    if parsed_args.fail_fast {
                        process::exit(1);
                    }
                } else if parsed_args.verbose {
                    println!("Successfully parsed {}", file_path.display());
                    println!("AST: {:#?}", ast);
                } else {
                    // Just indicate success
                    if parsed_args.files.len() > 1 {
                        println!("{}: OK", file_path.display());
                    }
                }
            }
            Err(parse_err) => {
                eprintln!("Error parsing {}: {}", file_path.display(), parse_err);
                had_errors = true;
                if parsed_args.fail_fast {
                    process::exit(1);
                }
            }
        }
    }

    if had_errors {
        process::exit(1);
    } else {
        if !parsed_args.verbose && parsed_args.files.len() == 1 {
            // Single file success case - don't print anything for compatibility
            // with other parsers in sv-tests
        }
        process::exit(0);
    }
}
