use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct ParsedArgs {
    pub files: Vec<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub defines: Vec<String>,
    pub verbose: bool,
    pub syntax_only: bool,
}

pub fn parse_vcs_style_args(
    raw_args: Vec<String>,
    verbose: bool,
    syntax_only: bool,
) -> Result<ParsedArgs, String> {
    let mut files = Vec::new();
    let mut include_dirs = Vec::new();
    let mut defines = Vec::new();

    for arg in raw_args {
        if let Some(incdir_path) = arg.strip_prefix("+incdir+") {
            if incdir_path.is_empty() {
                return Err("Empty path in +incdir+ directive".to_string());
            }
            include_dirs.push(PathBuf::from(incdir_path));
        } else if let Some(define_str) = arg.strip_prefix("+define+") {
            if define_str.is_empty() {
                return Err("Empty define in +define+ directive".to_string());
            }
            defines.push(define_str.to_string());
        } else if arg.starts_with('+') {
            // Other VCS-style options that we don't support yet
            eprintln!("Warning: Unsupported VCS option: {}", arg);
        } else if arg.starts_with('-') {
            // Skip clap flags that might have been passed through
            if arg == "-v" || arg == "--verbose" || arg == "-s" || arg == "--syntax-only" {
                continue;
            }
            return Err(format!("Unknown option: {}", arg));
        } else {
            // This is a file
            files.push(PathBuf::from(arg));
        }
    }

    if files.is_empty() {
        return Err("No input files specified".to_string());
    }

    Ok(ParsedArgs {
        files,
        include_dirs,
        defines,
        verbose,
        syntax_only,
    })
}
