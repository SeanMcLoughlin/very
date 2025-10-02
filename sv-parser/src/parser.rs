use chumsky::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::preprocessor::Preprocessor;
use crate::{
    AssignmentOp, BinaryOp, ClassItem, ClassQualifier, DriveStrength, Expression, ModuleItem,
    ParseError, ParseErrorType, Port, PortDirection, ProceduralBlockType, Range, SingleParseError,
    SourceLocation, SourceUnit, Statement, UnaryOp,
};

#[derive(Debug)]
pub struct SystemVerilogParser {
    preprocessor: Preprocessor,
    fail_fast: bool,
}

impl SystemVerilogParser {
    pub fn new(include_dirs: Vec<PathBuf>, initial_macros: HashMap<String, String>) -> Self {
        Self::with_config(include_dirs, initial_macros, false)
    }

    pub fn with_config(
        include_dirs: Vec<PathBuf>,
        initial_macros: HashMap<String, String>,
        fail_fast: bool,
    ) -> Self {
        Self {
            preprocessor: Preprocessor::new(include_dirs, initial_macros),
            fail_fast,
        }
    }

    pub fn parse_file(&mut self, file_path: &Path) -> Result<SourceUnit, ParseError> {
        // Read the raw file content
        let raw_content = std::fs::read_to_string(file_path).map_err(|e| {
            ParseError::new(SingleParseError::new(
                format!("Failed to read file {}: {}", file_path.display(), e),
                ParseErrorType::PreprocessorError,
            ))
        })?;

        // Parse the raw content (to capture preprocessor directives in AST)
        let mut ast = self.parse_content(&raw_content)?;

        // Resolve include paths in the AST
        self.resolve_includes_in_ast(&mut ast, file_path)?;

        Ok(ast)
    }

    /// Resolve all include directive paths in the AST
    fn resolve_includes_in_ast(
        &self,
        ast: &mut SourceUnit,
        current_file: &Path,
    ) -> Result<(), ParseError> {
        for item in &mut ast.items {
            self.resolve_includes_in_item(item, current_file)?;
        }
        Ok(())
    }

    /// Recursively resolve includes in a module item
    fn resolve_includes_in_item(
        &self,
        item: &mut ModuleItem,
        current_file: &Path,
    ) -> Result<(), ParseError> {
        match item {
            ModuleItem::IncludeDirective {
                path,
                resolved_path,
                ..
            } => {
                // Resolve the include path
                *resolved_path = Some(self.resolve_include_path(path, current_file)?);
            }
            ModuleItem::ModuleDeclaration { items, .. } => {
                // Recursively resolve includes in module body
                for sub_item in items {
                    self.resolve_includes_in_item(sub_item, current_file)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Resolve an include path relative to the current file and include directories
    fn resolve_include_path(
        &self,
        filename: &str,
        current_file: &Path,
    ) -> Result<PathBuf, ParseError> {
        // Try to find the file in include directories
        let mut found_path = None;

        // First try relative to current file
        if let Some(parent) = current_file.parent() {
            let candidate = parent.join(filename);
            if candidate.exists() {
                found_path = Some(candidate);
            }
        }

        // Then try include directories from preprocessor
        if found_path.is_none() {
            for inc_dir in &self.preprocessor.include_dirs {
                let candidate = inc_dir.join(filename);
                if candidate.exists() {
                    found_path = Some(candidate);
                    break;
                }
            }
        }

        found_path.ok_or_else(|| {
            ParseError::new(
                SingleParseError::new(
                    format!("Include file '{}' not found", filename),
                    ParseErrorType::PreprocessorError,
                )
                .with_suggestion(format!(
                    "Check if '{}' exists in include directories",
                    filename
                )),
            )
        })
    }

    /// Parse a file and recursively parse all included files, merging them into a single AST
    /// This is useful for building a complete project AST
    pub fn parse_file_with_includes(&mut self, file_path: &Path) -> Result<SourceUnit, ParseError> {
        let mut visited = std::collections::HashSet::new();
        self.parse_file_recursive(file_path, &mut visited)
    }

    /// Recursively parse a file and its includes
    fn parse_file_recursive(
        &mut self,
        file_path: &Path,
        visited: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<SourceUnit, ParseError> {
        // Canonicalize the path to handle relative paths and symlinks
        let canonical_path = file_path.canonicalize().map_err(|e| {
            ParseError::new(SingleParseError::new(
                format!("Failed to canonicalize path {}: {}", file_path.display(), e),
                ParseErrorType::PreprocessorError,
            ))
        })?;

        // Check for circular includes
        if visited.contains(&canonical_path) {
            return Ok(SourceUnit { items: vec![] }); // Skip circular includes silently
        }
        visited.insert(canonical_path.clone());

        // Parse the file normally
        let mut ast = self.parse_file(file_path)?;

        // Process all include directives
        let mut expanded_items = Vec::new();
        for item in ast.items {
            match item {
                ModuleItem::IncludeDirective { resolved_path, .. } => {
                    // Parse the included file recursively
                    if let Some(ref include_path) = resolved_path {
                        match self.parse_file_recursive(include_path, visited) {
                            Ok(included_ast) => {
                                // Add all items from the included file
                                expanded_items.extend(included_ast.items);
                            }
                            Err(_) => {
                                // If we can't parse an included file, skip it but continue
                                // This allows partial parsing of projects
                            }
                        }
                    }
                }
                other => {
                    expanded_items.push(other);
                }
            }
        }

        ast.items = expanded_items;
        Ok(ast)
    }

    /// Parse with error recovery - returns partial AST even if there are errors
    pub fn parse_content_recovery(&self, content: &str) -> crate::ParseResult {
        let parser = self.source_unit_parser();

        let (ast, chumsky_errors) = parser.parse_recovery(content);

        let mut parse_errors = Vec::new();

        // Process all errors from chumsky
        for error in chumsky_errors {
            let (message, error_type, suggestions, improved_span) =
                self.convert_chumsky_error(&error, content);

            // Use improved span if available, otherwise use chumsky's span
            let final_span = improved_span.unwrap_or_else(|| error.span());
            let location = self.span_to_location(final_span.clone(), content);

            let mut single_error = SingleParseError::new(message, error_type);

            if let Some(loc) = location {
                single_error = single_error.with_location(loc);
            }

            if !suggestions.is_empty() {
                single_error = single_error.with_suggestions(suggestions);
            }

            parse_errors.push(single_error);
        }

        // Sort errors by location for better presentation
        parse_errors.sort_by(|a, b| match (&a.location, &b.location) {
            (Some(loc_a), Some(loc_b)) => loc_a
                .line
                .cmp(&loc_b.line)
                .then_with(|| loc_a.column.cmp(&loc_b.column)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        crate::ParseResult {
            ast,
            errors: parse_errors,
        }
    }

    pub fn parse_content(&self, content: &str) -> Result<SourceUnit, ParseError> {
        let parser = self.source_unit_parser();

        parser.parse(content).map_err(|chumsky_errors| {
            let mut parse_errors = Vec::new();

            // Process all errors from chumsky
            for error in chumsky_errors {
                let (message, error_type, suggestions, improved_span) =
                    self.convert_chumsky_error(&error, content);

                // Use improved span if available, otherwise use chumsky's span
                let final_span = improved_span.unwrap_or_else(|| error.span());
                let location = self.span_to_location(final_span.clone(), content);

                let mut single_error = SingleParseError::new(message, error_type);

                if let Some(loc) = location {
                    single_error = single_error.with_location(loc);
                }

                if !suggestions.is_empty() {
                    single_error = single_error.with_suggestions(suggestions);
                }

                parse_errors.push(single_error);

                // If fail-fast is enabled, stop after the first error
                if self.fail_fast {
                    break;
                }
            }

            if parse_errors.is_empty() {
                // Fallback error if we couldn't convert any errors
                ParseError::new(SingleParseError::new(
                    "Unknown parse error".to_string(),
                    ParseErrorType::InvalidSyntax,
                ))
            } else {
                // Sort errors by location for better presentation
                parse_errors.sort_by(|a, b| match (&a.location, &b.location) {
                    (Some(loc_a), Some(loc_b)) => loc_a
                        .line
                        .cmp(&loc_b.line)
                        .then_with(|| loc_a.column.cmp(&loc_b.column)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });

                ParseError::multiple(parse_errors)
            }
        })
    }

    /// Perform semantic analysis on a parsed AST
    /// Returns a vector of semantic errors (empty if no errors found)
    pub fn analyze_semantics(&self, ast: &SourceUnit) -> Vec<crate::semantic::SemanticError> {
        let mut analyzer = crate::semantic::SemanticAnalyzer::new();
        analyzer.analyze(ast)
    }

    fn span_to_location(
        &self,
        span: std::ops::Range<usize>,
        content: &str,
    ) -> Option<SourceLocation> {
        if span.start > content.len() {
            return None;
        }

        let prefix = &content[..span.start];
        let line = prefix.matches('\n').count();
        let column = prefix.split('\n').last().unwrap_or("").len();

        Some(SourceLocation {
            line,
            column,
            span: Some((span.start, span.end)),
        })
    }

    fn convert_chumsky_error(
        &self,
        error: &Simple<char>,
        content: &str,
    ) -> (
        String,
        ParseErrorType,
        Vec<String>,
        Option<std::ops::Range<usize>>,
    ) {
        match error.reason() {
            chumsky::error::SimpleReason::Unexpected => {
                let found = error
                    .found()
                    .map(|c| format!("'{}'", c))
                    .unwrap_or_else(|| "end of input".to_string());

                let expected: Vec<String> = error
                    .expected()
                    .map(|exp| match exp {
                        Some(c) => format!("'{}'", c),
                        None => "end of input".to_string(),
                    })
                    .collect();

                // Analyze the context to provide better error messages
                self.analyze_parse_context(&found, &expected, error.span(), content)
            }
            chumsky::error::SimpleReason::Unclosed { span: _, delimiter } => (
                format!("Unclosed delimiter '{}'", delimiter),
                ParseErrorType::ExpectedToken(delimiter.to_string()),
                vec![format!("Add closing '{}'", delimiter)],
                None,
            ),
            chumsky::error::SimpleReason::Custom(msg) => {
                (msg.clone(), ParseErrorType::InvalidSyntax, Vec::new(), None)
            }
        }
    }

    fn analyze_parse_context(
        &self,
        found: &str,
        expected: &[String],
        _error_span: std::ops::Range<usize>,
        _content: &str,
    ) -> (
        String,
        ParseErrorType,
        Vec<String>,
        Option<std::ops::Range<usize>>,
    ) {
        // Special case: if we're expecting 'n' and found newline/end, it's likely missing "endmodule"
        if expected.contains(&"'n'".to_string()) {
            if found == "end of input" || found == "'\n'" {
                return (
                    "Missing 'endmodule' to close module declaration".to_string(),
                    ParseErrorType::ExpectedToken("endmodule".to_string()),
                    vec!["Add 'endmodule' to complete the module".to_string()],
                    None,
                );
            } else {
                return (
                    format!("Expected 'endmodule', found unexpected character {}", found),
                    ParseErrorType::UnexpectedToken,
                    vec!["Replace with 'endmodule' to close the module".to_string()],
                    None,
                );
            }
        }

        // Check if we're at end of input and expecting statement terminators
        if found == "end of input" && expected.contains(&"';'".to_string()) {
            return (
                "Missing semicolon at end of statement".to_string(),
                ParseErrorType::ExpectedToken(";".to_string()),
                vec!["Add ';' to complete the statement".to_string()],
                None,
            );
        }

        // Check if we're expecting characters that form other keywords
        if self.expects_keyword_pattern(expected) {
            let (msg, err_type, sugg) = self.suggest_keyword_completion(found, expected);
            return (msg, err_type, sugg, None);
        }

        // Check for unexpected character when expecting specific tokens
        if found != "end of input" && !expected.is_empty() {
            let meaningful_expected: Vec<String> = expected
                .iter()
                .filter(|exp| self.is_meaningful_expectation(exp))
                .cloned()
                .collect();

            if !meaningful_expected.is_empty() {
                let suggestions = meaningful_expected
                    .iter()
                    .map(|exp| self.expectation_to_suggestion(exp))
                    .filter(|s| !s.is_empty())
                    .collect();

                return (
                    format!(
                        "Unexpected {}, expected {}",
                        found,
                        self.format_expectations(&meaningful_expected)
                    ),
                    ParseErrorType::UnexpectedToken,
                    suggestions,
                    None,
                );
            }
        }

        // Fallback to generic message
        let message = if expected.is_empty() {
            format!("Unexpected {}", found)
        } else {
            format!("Unexpected {}", found)
        };

        (message, ParseErrorType::UnexpectedToken, Vec::new(), None)
    }

    fn expects_keyword_pattern(&self, expected: &[String]) -> bool {
        // Check if the expected characters suggest we're in the middle of parsing a keyword
        expected.iter().any(|exp| {
            matches!(
                exp.as_str(),
                "'n'" | "'d'" | "'m'" | "'o'" | "'u'" | "'l'" | "'e'"
            )
        })
    }

    fn suggest_keyword_completion(
        &self,
        found: &str,
        expected: &[String],
    ) -> (String, ParseErrorType, Vec<String>) {
        // If we expect 'n' and are at end of input, likely missing "endmodule"
        if expected.contains(&"'n'".to_string()) && found == "end of input" {
            return (
                "Missing 'endmodule' to close module declaration".to_string(),
                ParseErrorType::ExpectedToken("endmodule".to_string()),
                vec!["Add 'endmodule' to complete the module".to_string()],
            );
        }

        // If we expect 'd' and are at end of input, might be missing "end"
        if expected.contains(&"'d'".to_string()) && found == "end of input" {
            return (
                "Missing keyword completion".to_string(),
                ParseErrorType::ExpectedToken("end".to_string()),
                vec!["Complete the keyword (e.g., 'end', 'endmodule')".to_string()],
            );
        }

        // Default for keyword patterns
        (
            format!("Incomplete keyword, found {}", found),
            ParseErrorType::ExpectedToken("keyword".to_string()),
            vec!["Complete the SystemVerilog keyword".to_string()],
        )
    }

    fn is_meaningful_expectation(&self, exp: &str) -> bool {
        // Filter out single letter expectations that are likely part of keywords
        matches!(
            exp,
            "';'" | "'('" | "')'" | "'{'" | "'}'" | "'['" | "']'" | "','" | "'='" | "end of input"
        )
    }

    fn expectation_to_suggestion(&self, exp: &str) -> String {
        match exp {
            "';'" => "Add semicolon ';'".to_string(),
            "'('" => "Add opening parenthesis '('".to_string(),
            "')'" => "Add closing parenthesis ')'".to_string(),
            "'{'" => "Add opening brace '{'".to_string(),
            "'}'" => "Add closing brace '}'".to_string(),
            "'['" => "Add opening bracket '['".to_string(),
            "']'" => "Add closing bracket ']'".to_string(),
            "','" => "Add comma ','".to_string(),
            "'='" => "Add assignment operator '='".to_string(),
            "end of input" => "Check if statement is complete".to_string(),
            _ => String::new(),
        }
    }

    fn format_expectations(&self, expected: &[String]) -> String {
        if expected.len() == 1 {
            expected[0].clone()
        } else if expected.len() <= 3 {
            expected.join(" or ")
        } else {
            format!("one of: {}", expected.join(", "))
        }
    }

    fn source_unit_parser(&self) -> impl Parser<char, SourceUnit, Error = Simple<char>> + Clone {
        // Comments
        let line_comment = just("//").then(filter(|c| *c != '\n').repeated()).ignored();

        let block_comment = just("/*")
            .then(just("*/").not().rewind().then(any()).repeated())
            .then(just("*/"))
            .ignored();

        let comment = choice((line_comment, block_comment));

        let whitespace =
            choice((one_of(" \t\r\n").repeated().at_least(1).ignored(), comment)).repeated();

        // Basic tokens
        let identifier_inner = filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
            .then(filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_').repeated())
            .map(|(first, rest): (char, Vec<char>)| {
                let mut result = String::new();
                result.push(first);
                result.extend(rest);
                result
            });

        let identifier = identifier_inner.clone().padded_by(whitespace.clone());

        // Helper to get identifier with its span (before padding)
        let identifier_with_span = identifier_inner
            .clone()
            .map_with_span(|name, span: std::ops::Range<usize>| (name, (span.start, span.end)))
            .padded_by(whitespace.clone());

        // Support both simple numbers and SystemVerilog sized numbers like 8'b1101z001
        let number = choice((
            // SystemVerilog sized number: size'base_value (e.g., 8'b1101z001, 4'hA, 32'd123)
            filter(|c: &char| c.is_ascii_digit())
                .repeated()
                .at_least(1)
                .then_ignore(just('\''))
                .then(one_of("bBdDhHoO"))
                .then(
                    filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
                        .repeated()
                        .at_least(1),
                )
                .map(
                    |((size_digits, base), value_chars): ((Vec<char>, char), Vec<char>)| {
                        let mut result = String::new();
                        result.extend(size_digits);
                        result.push('\'');
                        result.push(base);
                        result.extend(value_chars);
                        result
                    },
                ),
            // Simple decimal number
            filter(|c: &char| c.is_ascii_digit())
                .repeated()
                .at_least(1)
                .collect::<String>(),
        ))
        .padded_by(whitespace.clone());

        // String literal parser - handles escaped quotes
        let string_literal = just('"')
            .ignore_then(
                filter(|c: &char| *c != '"' && *c != '\\')
                    .or(just('\\').ignore_then(any()))
                    .repeated()
                    .collect::<String>(),
            )
            .then_ignore(just('"'))
            .padded_by(whitespace.clone());

        // Expression parser with unary and binary operators
        let expr = recursive(|expr| {
            // System function call like $sin(4)
            let system_function = just('$')
                .ignore_then(identifier.clone())
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(whitespace.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not()
                        .map(|args| args.unwrap_or_default()),
                )
                .map_with_span(|(name, arguments), span: std::ops::Range<usize>| {
                    Expression::SystemFunctionCall {
                        name,
                        arguments,
                        span: (span.start, span.end),
                    }
                })
                .padded_by(whitespace.clone());

            // New expression (class instantiation) - parentheses are optional
            let new_expr = just("new")
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(whitespace.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not(),
                )
                .map_with_span(
                    |(_new, arguments), span: std::ops::Range<usize>| Expression::New {
                        arguments: arguments.unwrap_or_default(),
                        span: (span.start, span.end),
                    },
                )
                .padded_by(whitespace.clone());

            let atom = choice((
                new_expr,
                system_function,
                string_literal
                    .clone()
                    .map_with_span(|s, span: std::ops::Range<usize>| {
                        Expression::StringLiteral(s, (span.start, span.end))
                    }),
                identifier
                    .clone()
                    .map_with_span(|name, span: std::ops::Range<usize>| {
                        Expression::Identifier(name, (span.start, span.end))
                    }),
                number
                    .clone()
                    .map_with_span(|num, span: std::ops::Range<usize>| {
                        Expression::Number(num, (span.start, span.end))
                    }),
                expr.clone().delimited_by(just('('), just(')')),
            ))
            .padded_by(whitespace.clone());

            // Unary operators - note the order is important for correct parsing
            let unary_op = choice((
                just("~&").to(UnaryOp::ReductionNand),
                just("~|").to(UnaryOp::ReductionNor),
                just("~^").to(UnaryOp::ReductionXnor),
                just("~").to(UnaryOp::Not),
                just("!").to(UnaryOp::LogicalNot),
                just("+").to(UnaryOp::Plus),
                just("-").to(UnaryOp::Minus),
                just("&").to(UnaryOp::ReductionAnd),
                just("|").to(UnaryOp::ReductionOr),
                just("^").to(UnaryOp::ReductionXor),
            ))
            .padded_by(whitespace.clone());

            // Member access handles dot notation (e.g., obj.property)
            let member_access = atom
                .clone()
                .then(
                    just('.')
                        .ignore_then(identifier_with_span.clone())
                        .repeated(),
                )
                .foldl(|object, (member, member_span)| {
                    let start = match &object {
                        Expression::Identifier(_, span)
                        | Expression::Number(_, span)
                        | Expression::StringLiteral(_, span)
                        | Expression::Binary { span, .. }
                        | Expression::Unary { span, .. }
                        | Expression::MacroUsage { span, .. }
                        | Expression::SystemFunctionCall { span, .. }
                        | Expression::New { span, .. }
                        | Expression::MemberAccess { span, .. }
                        | Expression::FunctionCall { span, .. } => span.0,
                    };
                    Expression::MemberAccess {
                        object: Box::new(object),
                        member,
                        member_span,
                        span: (start, member_span.1),
                    }
                })
                .padded_by(whitespace.clone());

            // Function call handles function/method calls (e.g., func() or obj.method())
            let function_call = member_access
                .clone()
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(whitespace.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not(),
                )
                .map_with_span(|(function, maybe_args), span: std::ops::Range<usize>| {
                    if let Some(args) = maybe_args {
                        Expression::FunctionCall {
                            function: Box::new(function),
                            arguments: args,
                            span: (span.start, span.end),
                        }
                    } else {
                        function
                    }
                })
                .padded_by(whitespace.clone());

            // Factor handles unary operators, member access, and function calls
            let factor = choice((
                unary_op.clone().then(expr.clone()).map_with_span(
                    |(op, operand), span: std::ops::Range<usize>| Expression::Unary {
                        op,
                        operand: Box::new(operand),
                        span: (span.start, span.end),
                    },
                ),
                function_call,
            ))
            .padded_by(whitespace.clone());

            // Multi-character operators (must come before single-character ones)
            let multi_char_ops = choice((
                just("<->").to(BinaryOp::LogicalEquiv),
                just("**").to(BinaryOp::Power),
                just("<<<").to(BinaryOp::ArithmeticShiftLeft),
                just(">>>").to(BinaryOp::ArithmeticShiftRight),
                just("<<").to(BinaryOp::LogicalShiftLeft),
                just(">>").to(BinaryOp::LogicalShiftRight),
                just("<=").to(BinaryOp::LessEqual),
                just(">=").to(BinaryOp::GreaterEqual),
                just("&&").to(BinaryOp::LogicalAnd),
                just("||").to(BinaryOp::LogicalOr),
                just("->").to(BinaryOp::LogicalImpl),
                just("===").to(BinaryOp::CaseEqual),
                just("!==").to(BinaryOp::CaseNotEqual),
                just("==?").to(BinaryOp::WildcardEqual),
                just("!=?").to(BinaryOp::WildcardNotEqual),
                just("==").to(BinaryOp::Equal),
                just("!=").to(BinaryOp::NotEqual),
                just("~^").to(BinaryOp::BitwiseXnor),
            ));

            let single_char_ops = choice((
                just('<').to(BinaryOp::LessThan),
                just('>').to(BinaryOp::GreaterThan),
                just('+').to(BinaryOp::Add),
                just('-').to(BinaryOp::Sub),
                just('*').to(BinaryOp::Mul),
                just('/').to(BinaryOp::Div),
                just('%').to(BinaryOp::Modulo),
                just('&').to(BinaryOp::And),
                just('|').to(BinaryOp::Or),
                just('^').to(BinaryOp::Xor),
            ));

            let binary_op = choice((multi_char_ops, single_char_ops)).padded_by(whitespace.clone());

            factor
                .clone()
                .then(binary_op.then(factor).repeated())
                .foldl(|left, (op, right)| {
                    // Calculate span from left to right
                    let left_span = match &left {
                        Expression::Identifier(_, s)
                        | Expression::Number(_, s)
                        | Expression::StringLiteral(_, s) => *s,
                        Expression::Binary { span, .. }
                        | Expression::Unary { span, .. }
                        | Expression::MacroUsage { span, .. }
                        | Expression::SystemFunctionCall { span, .. }
                        | Expression::New { span, .. }
                        | Expression::MemberAccess { span, .. }
                        | Expression::FunctionCall { span, .. } => *span,
                    };
                    let right_span = match &right {
                        Expression::Identifier(_, s)
                        | Expression::Number(_, s)
                        | Expression::StringLiteral(_, s) => *s,
                        Expression::Binary { span, .. }
                        | Expression::Unary { span, .. }
                        | Expression::MacroUsage { span, .. }
                        | Expression::SystemFunctionCall { span, .. }
                        | Expression::New { span, .. }
                        | Expression::MemberAccess { span, .. }
                        | Expression::FunctionCall { span, .. } => *span,
                    };
                    Expression::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                        span: (left_span.0, right_span.1),
                    }
                })
        });

        // Port direction
        let port_direction = choice((
            just("input").to(PortDirection::Input),
            just("output").to(PortDirection::Output),
            just("inout").to(PortDirection::Inout),
        ))
        .padded_by(whitespace.clone());

        // Range parser [msb:lsb] e.g., [7:0] or [3:0]
        let range = just('[')
            .ignore_then(choice((number.clone(), identifier.clone())))
            .then_ignore(just(':'))
            .then(choice((number.clone(), identifier.clone())))
            .then_ignore(just(']'))
            .map(|(msb, lsb)| Range { msb, lsb })
            .padded_by(whitespace.clone());

        // Port in module header can be:
        // - just identifier: clk
        // - direction + identifier: input clk
        // - direction + range + identifier: input [3:0] clk
        // - direction + type + identifier: input wire clk
        // - direction + type + range + identifier: input wire [3:0] clk
        let module_port = choice((
            // direction + type + range + identifier: input wire [3:0] clk
            port_direction
                .clone()
                .then(identifier.clone()) // type (wire, reg, etc.)
                .then(range.clone().or_not())
                .then(identifier_with_span.clone()) // name
                .map_with_span(
                    |(((direction, _type), range), (name, name_span)),
                     span: std::ops::Range<usize>| Port {
                        name: name.clone(),
                        name_span,
                        direction: Some(direction),
                        range,
                        span: (span.start, span.end),
                    },
                ),
            // direction + range + identifier: input [3:0] clk
            port_direction
                .clone()
                .then(range.clone().or_not())
                .then(identifier_with_span.clone()) // name
                .map_with_span(
                    |((direction, range), (name, name_span)), span: std::ops::Range<usize>| Port {
                        name: name.clone(),
                        name_span,
                        direction: Some(direction),
                        range,
                        span: (span.start, span.end),
                    },
                ),
            // just identifier: clk
            identifier_with_span.clone().map_with_span(
                |(name, name_span), span: std::ops::Range<usize>| Port {
                    name: name.clone(),
                    name_span,
                    direction: None,
                    range: None,
                    span: (span.start, span.end),
                },
            ),
        ));

        // Port declaration with error recovery
        let port_declaration = port_direction
            .then(identifier.clone()) // port type (like wire, reg)
            .then(identifier_with_span.clone()) // port name
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map_with_span(
                |((direction, port_type), (name, name_span)), span: std::ops::Range<usize>| {
                    ModuleItem::PortDeclaration {
                        direction,
                        port_type,
                        name: name.clone(),
                        name_span,
                        span: (span.start, span.end),
                    }
                },
            )
            .recover_with(skip_then_retry_until([';']))
            .padded_by(whitespace.clone());

        // Assignment statement with error recovery
        let assignment = just("assign")
            .padded_by(whitespace.clone())
            .ignore_then(identifier_with_span.clone())
            .then_ignore(just('=').padded_by(whitespace.clone()))
            .then(expr.clone())
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map_with_span(
                |((target, target_span), expr), span: std::ops::Range<usize>| {
                    ModuleItem::Assignment {
                        target: target.clone(),
                        target_span,
                        expr,
                        span: (span.start, span.end),
                    }
                },
            )
            .recover_with(skip_then_retry_until([';']))
            .padded_by(whitespace.clone());

        // Data type keywords - order matters! Longer keywords first to avoid prefix matching
        let data_type = choice((
            text::keyword("shortint").to("shortint".to_string()),
            text::keyword("longint").to("longint".to_string()),
            text::keyword("integer").to("integer".to_string()),
            text::keyword("supply0").to("supply0".to_string()),
            text::keyword("supply1").to("supply1".to_string()),
            text::keyword("trireg").to("trireg".to_string()),
            text::keyword("triand").to("triand".to_string()),
            text::keyword("trior").to("trior".to_string()),
            text::keyword("logic").to("logic".to_string()),
            text::keyword("uwire").to("uwire".to_string()),
            text::keyword("wire").to("wire".to_string()),
            text::keyword("wand").to("wand".to_string()),
            text::keyword("byte").to("byte".to_string()),
            text::keyword("time").to("time".to_string()),
            text::keyword("tri0").to("tri0".to_string()),
            text::keyword("tri1").to("tri1".to_string()),
            text::keyword("wor").to("wor".to_string()),
            text::keyword("tri").to("tri".to_string()),
            text::keyword("int").to("int".to_string()),
            text::keyword("bit").to("bit".to_string()),
            text::keyword("reg").to("reg".to_string()),
        ))
        .padded_by(whitespace.clone());

        // Signing keywords (signed/unsigned)
        // Use text::keyword to ensure word boundaries
        let signing = choice((
            text::keyword("signed").to("signed".to_string()),
            text::keyword("unsigned").to("unsigned".to_string()),
        ))
        .padded_by(whitespace.clone());

        // Drive strength parser
        // Syntax: (strength0, strength1) or (strength1, strength0)
        // strength1: supply1, strong1, pull1, weak1, highz1
        // strength0: supply0, strong0, pull0, weak0, highz0
        let strength0_keyword = choice((
            text::keyword("supply0").to("supply0".to_string()),
            text::keyword("strong0").to("strong0".to_string()),
            text::keyword("highz0").to("highz0".to_string()),
            text::keyword("pull0").to("pull0".to_string()),
            text::keyword("weak0").to("weak0".to_string()),
        ))
        .padded_by(whitespace.clone());

        let strength1_keyword = choice((
            text::keyword("supply1").to("supply1".to_string()),
            text::keyword("strong1").to("strong1".to_string()),
            text::keyword("highz1").to("highz1".to_string()),
            text::keyword("pull1").to("pull1".to_string()),
            text::keyword("weak1").to("weak1".to_string()),
        ))
        .padded_by(whitespace.clone());

        let drive_strength = just('(')
            .padded_by(whitespace.clone())
            .ignore_then(
                // Try both orders: (strength0, strength1) or (strength1, strength0)
                strength0_keyword
                    .clone()
                    .then_ignore(just(',').padded_by(whitespace.clone()))
                    .then(strength1_keyword.clone())
                    .map(|(s0, s1)| DriveStrength {
                        strength0: s0,
                        strength1: s1,
                    })
                    .or(strength1_keyword
                        .clone()
                        .then_ignore(just(',').padded_by(whitespace.clone()))
                        .then(strength0_keyword.clone())
                        .map(|(s1, s0)| DriveStrength {
                            strength0: s0,
                            strength1: s1,
                        })),
            )
            .then_ignore(just(')').padded_by(whitespace.clone()));

        // Variable declaration with optional range and initialization
        // Examples:
        //   wire a;
        //   wire [7:0] data;
        //   int count = 5;
        //   logic [3:0] addr = 4'b0000;
        //   logic a, b, c;  (multiple variables)
        let _single_var = identifier_with_span
            .clone()
            .then(
                just('=')
                    .padded_by(whitespace.clone())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .map(|((name, name_span), initial_value)| (name, name_span, initial_value));

        // Variable name that can't be "signed" or "unsigned" (for use after data types)
        let single_var_not_signing = identifier_with_span
            .clone()
            .try_map(|(name, span), map_span| {
                if name == "signed" || name == "unsigned" {
                    Err(chumsky::error::Error::expected_input_found(
                        map_span,
                        vec![],
                        Some('s'),
                    ))
                } else {
                    Ok((name, span))
                }
            })
            .then(
                just('=')
                    .padded_by(whitespace.clone())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .map(|((name, name_span), initial_value)| (name, name_span, initial_value));

        // Variable declaration with built-in types
        let builtin_var_decl = data_type
            .then(signing.clone().or_not())
            .then(drive_strength.clone().or_not())
            .then(range.clone().or_not())
            .then(
                single_var_not_signing
                    .clone()
                    .separated_by(just(',').padded_by(whitespace.clone()))
                    .at_least(1),
            )
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map_with_span(
                |((((data_type, signing), drive_strength), range), vars),
                 span: std::ops::Range<usize>| {
                    let (name, name_span, initial_value) = vars.into_iter().next().unwrap();
                    ModuleItem::VariableDeclaration {
                        data_type,
                        signing,
                        drive_strength,
                        range,
                        name: name.clone(),
                        name_span,
                        initial_value,
                        span: (span.start, span.end),
                    }
                },
            );

        // Variable declaration with built-in types only
        let variable_declaration = builtin_var_decl
            .recover_with(skip_then_retry_until([';']))
            .padded_by(whitespace.clone());

        // Class-typed variable declaration (separate parser to control ordering)
        // This matches: <identifier> <identifier> ;
        // But we need to make sure the first identifier is NOT a reserved keyword
        let class_var_decl = identifier
            .clone()
            .then(identifier_with_span.clone())
            .then(
                just('=')
                    .padded_by(whitespace.clone())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map_with_span(
                |((data_type, (name, name_span)), initial_value), span: std::ops::Range<usize>| {
                    ModuleItem::VariableDeclaration {
                        data_type,
                        signing: None,
                        drive_strength: None,
                        range: None,
                        name,
                        name_span,
                        initial_value,
                        span: (span.start, span.end),
                    }
                },
            )
            .padded_by(whitespace.clone());

        // Port list in module header
        let port_list = module_port
            .separated_by(just(',').padded_by(whitespace.clone()))
            .delimited_by(just('('), just(')'))
            .or_not()
            .map(|ports| ports.unwrap_or_default())
            .padded_by(whitespace.clone());

        // Statement parser for procedural blocks
        let statement = recursive(|_statement| {
            // Assignment operators
            let assignment_op = choice((
                just(">>>=").to(AssignmentOp::AShrAssign),
                just("<<<=").to(AssignmentOp::AShlAssign),
                just(">>=").to(AssignmentOp::ShrAssign),
                just("<<=").to(AssignmentOp::ShlAssign),
                just("+=").to(AssignmentOp::AddAssign),
                just("-=").to(AssignmentOp::SubAssign),
                just("*=").to(AssignmentOp::MulAssign),
                just("/=").to(AssignmentOp::DivAssign),
                just("%=").to(AssignmentOp::ModAssign),
                just("&=").to(AssignmentOp::AndAssign),
                just("|=").to(AssignmentOp::OrAssign),
                just("^=").to(AssignmentOp::XorAssign),
                just("=").to(AssignmentOp::Assign),
            ))
            .padded_by(whitespace.clone());

            // Assignment statement (without 'assign' keyword)
            let stmt_assignment = identifier_with_span
                .clone()
                .then(assignment_op)
                .then(expr.clone())
                .then_ignore(just(';').padded_by(whitespace.clone()))
                .map_with_span(
                    |(((target, target_span), op), expr), span: std::ops::Range<usize>| {
                        Statement::Assignment {
                            target: target.clone(),
                            target_span,
                            op,
                            expr,
                            span: (span.start, span.end),
                        }
                    },
                )
                .padded_by(whitespace.clone());

            // System call like $display(...)
            let system_call = just('$')
                .ignore_then(identifier.clone())
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(whitespace.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not()
                        .map(|args| args.unwrap_or_default()),
                )
                .then_ignore(just(';').padded_by(whitespace.clone()))
                .map_with_span(
                    |(name, args), span: std::ops::Range<usize>| Statement::SystemCall {
                        name,
                        args,
                        span: (span.start, span.end),
                    },
                )
                .padded_by(whitespace.clone());

            // Case statement modifiers
            let case_modifier = choice((
                text::keyword("unique0").to("unique0".to_string()),
                text::keyword("unique").to("unique".to_string()),
                text::keyword("priority").to("priority".to_string()),
            ))
            .padded_by(whitespace.clone())
            .or_not();

            // Case type (case, casex, casez)
            let case_type = choice((
                text::keyword("casez").to("casez".to_string()),
                text::keyword("casex").to("casex".to_string()),
                text::keyword("case").to("case".to_string()),
            ))
            .padded_by(whitespace.clone());

            // Case statement
            let case_stmt = case_modifier
                .then(case_type)
                .then(expr.clone().delimited_by(
                    just('(').padded_by(whitespace.clone()),
                    just(')').padded_by(whitespace.clone()),
                ))
                .then_ignore(
                    // Case items - simplified parser that just skips to endcase
                    filter(|c| *c != 'e')
                        .repeated()
                        .then(text::keyword("endcase"))
                        .padded_by(whitespace.clone()),
                )
                .map_with_span(
                    |((modifier, case_type), case_expr), span: std::ops::Range<usize>| {
                        Statement::CaseStatement {
                            modifier,
                            case_type,
                            expr: case_expr,
                            span: (span.start, span.end),
                        }
                    },
                )
                .padded_by(whitespace.clone());

            // Assert property statement
            let assert_property = text::keyword("assert")
                .padded_by(whitespace.clone())
                .ignore_then(text::keyword("property").padded_by(whitespace.clone()))
                .ignore_then(expr.clone().delimited_by(
                    just('(').padded_by(whitespace.clone()),
                    just(')').padded_by(whitespace.clone()),
                ))
                .then(
                    text::keyword("else")
                        .padded_by(whitespace.clone())
                        .ignore_then(
                            just('$')
                                .ignore_then(identifier.clone())
                                .then(
                                    expr.clone()
                                        .separated_by(just(',').padded_by(whitespace.clone()))
                                        .delimited_by(just('('), just(')'))
                                        .or_not()
                                        .map(|args| args.unwrap_or_default()),
                                )
                                .map_with_span(|(name, args), span: std::ops::Range<usize>| {
                                    Statement::SystemCall {
                                        name,
                                        args,
                                        span: (span.start, span.end),
                                    }
                                }),
                        )
                        .or_not(),
                )
                .then_ignore(just(';').padded_by(whitespace.clone()))
                .map_with_span(
                    |(property_expr, action_block), span: std::ops::Range<usize>| {
                        Statement::AssertProperty {
                            property_expr,
                            action_block: action_block.map(Box::new),
                            span: (span.start, span.end),
                        }
                    },
                )
                .padded_by(whitespace.clone());

            // Expression statement (for method calls, function calls, etc.)
            let expr_stmt = expr
                .clone()
                .then_ignore(just(';').padded_by(whitespace.clone()))
                .map_with_span(|expr, span: std::ops::Range<usize>| {
                    Statement::ExpressionStatement {
                        expr,
                        span: (span.start, span.end),
                    }
                })
                .padded_by(whitespace.clone());

            choice((
                assert_property,
                case_stmt,
                stmt_assignment,
                system_call,
                expr_stmt,
            ))
        });

        // Procedural block type
        let block_type = choice((
            just("initial").to(ProceduralBlockType::Initial),
            just("final").to(ProceduralBlockType::Final),
            just("always_comb").to(ProceduralBlockType::AlwaysComb),
            just("always_ff").to(ProceduralBlockType::AlwaysFF),
            just("always").to(ProceduralBlockType::Always),
        ))
        .padded_by(whitespace.clone());

        // Procedural block
        let procedural_block = block_type
            .then_ignore(
                // Optional event control like @(posedge clk) or @(a)
                just('@')
                    .padded_by(whitespace.clone())
                    .ignore_then(
                        just('(')
                            .ignore_then(filter(|c| *c != ')').repeated())
                            .then_ignore(just(')')),
                    )
                    .or_not(),
            )
            .then(choice((
                // Multiple statements with begin/end
                statement.clone().repeated().delimited_by(
                    just("begin").padded_by(whitespace.clone()),
                    just("end").padded_by(whitespace.clone()),
                ),
                // Single statement without begin/end
                statement.clone().map(|s| vec![s]),
            )))
            .map_with_span(|(block_type, statements), span: std::ops::Range<usize>| {
                ModuleItem::ProceduralBlock {
                    block_type,
                    statements,
                    span: (span.start, span.end),
                }
            })
            .padded_by(whitespace.clone());

        // Preprocessor directive parsers
        let define_directive = just('`')
            .ignore_then(just("define"))
            .padded_by(whitespace.clone())
            .ignore_then(identifier_with_span.clone())
            .then(
                // Optional parameter list
                just('(')
                    .ignore_then(
                        identifier
                            .clone()
                            .separated_by(just(',').padded_by(whitespace.clone())),
                    )
                    .then_ignore(just(')'))
                    .or_not()
                    .padded_by(whitespace.clone()),
            )
            .then(
                // Macro value - everything until end of line
                filter(|c: &char| *c != '\n').repeated().collect::<String>(),
            )
            .map_with_span(
                |(((name, name_span), params), value), span: std::ops::Range<usize>| {
                    ModuleItem::DefineDirective {
                        name,
                        name_span,
                        parameters: params.unwrap_or_default(),
                        value: value.trim().to_string(),
                        span: (span.start, span.end),
                    }
                },
            );

        let include_directive = just('`')
            .ignore_then(just("include"))
            .padded_by(whitespace.clone())
            .ignore_then(
                // Parse "filename" or <filename>
                choice((
                    filter(|c: &char| *c != '\n' && *c != '"' && *c != '<' && *c != '>')
                        .repeated()
                        .collect::<String>()
                        .delimited_by(just('"'), just('"')),
                    filter(|c: &char| *c != '\n' && *c != '"' && *c != '<' && *c != '>')
                        .repeated()
                        .collect::<String>()
                        .delimited_by(just('<'), just('>')),
                ))
                .map_with_span(|path: String, span: std::ops::Range<usize>| {
                    (path, (span.start, span.end))
                }),
            )
            .map_with_span(|(path, path_span), span: std::ops::Range<usize>| {
                ModuleItem::IncludeDirective {
                    path,
                    path_span,
                    resolved_path: None, // Will be resolved later
                    span: (span.start, span.end),
                }
            });

        // Class qualifier parser (local or protected)
        let class_qualifier = || {
            choice((
                just("local").to(ClassQualifier::Local),
                just("protected").to(ClassQualifier::Protected),
            ))
            .padded_by(whitespace.clone())
        };

        // Class property parser
        let class_property = class_qualifier()
            .or_not()
            .then(identifier.clone()) // data_type
            .then(identifier_with_span.clone()) // name
            .then(
                just('=')
                    .padded_by(whitespace.clone())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map_with_span(
                |(((qualifier, data_type), (name, name_span)), initial_value),
                 span: std::ops::Range<usize>| {
                    ClassItem::Property {
                        qualifier,
                        data_type,
                        name,
                        name_span,
                        initial_value,
                        span: (span.start, span.end),
                    }
                },
            );

        // Class method parser
        let class_method = class_qualifier()
            .or_not()
            .then_ignore(just("function").padded_by(whitespace.clone()))
            .then(identifier.clone().then_ignore(whitespace.clone()).or_not()) // return_type (optional, defaults to void)
            .then(identifier_with_span.clone()) // method name
            .then(
                // parameter list
                just('(')
                    .padded_by(whitespace.clone())
                    .ignore_then(just(')'))
                    .to(Vec::new())
                    .or(just('(')
                        .padded_by(whitespace.clone())
                        .ignore_then(
                            identifier
                                .clone()
                                .separated_by(just(',').padded_by(whitespace.clone())),
                        )
                        .then_ignore(just(')').padded_by(whitespace.clone()))),
            )
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then(
                // function body - statements until endfunction
                statement
                    .recover_with(skip_then_retry_until(['$', 'e']))
                    .repeated(),
            )
            .then_ignore(just("endfunction").padded_by(whitespace.clone()))
            .map_with_span(
                |((((qualifier, return_type), (name, name_span)), _parameters), body),
                 span: std::ops::Range<usize>| {
                    ClassItem::Method {
                        qualifier,
                        return_type,
                        name,
                        name_span,
                        parameters: Vec::new(), // simplified for now
                        body,
                        span: (span.start, span.end),
                    }
                },
            );

        // Class item
        let class_item = choice((class_method, class_property));

        // Class body
        let class_body = class_item
            .recover_with(skip_then_retry_until([';', 'l', 'p', 'f', 'e']))
            .repeated();

        // Class declaration
        let class_declaration = just("class")
            .padded_by(whitespace.clone())
            .ignore_then(identifier_with_span.clone())
            .then(
                just("extends")
                    .padded_by(whitespace.clone())
                    .ignore_then(identifier.clone())
                    .or_not(),
            )
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then(class_body)
            .then_ignore(just("endclass").padded_by(whitespace.clone()))
            .map_with_span(
                |(((name, name_span), extends), items), span: std::ops::Range<usize>| {
                    ModuleItem::ClassDeclaration {
                        name: name.clone(),
                        name_span,
                        extends,
                        items,
                        span: (span.start, span.end),
                    }
                },
            )
            .padded_by(whitespace.clone());

        // Concurrent assertion (module-level assert property)
        // For now, just parse the structure without detailed validation
        let concurrent_assertion = text::keyword("assert")
            .padded_by(whitespace.clone())
            .ignore_then(text::keyword("property").padded_by(whitespace.clone()))
            .then_ignore(
                filter(|c| *c != ';')
                    .repeated()
                    .then_ignore(just(';').padded_by(whitespace.clone())),
            )
            .map_with_span(
                |_, span: std::ops::Range<usize>| ModuleItem::ConcurrentAssertion {
                    statement: Statement::ExpressionStatement {
                        expr: Expression::Identifier(
                            "placeholder".to_string(),
                            (span.start, span.end),
                        ),
                        span: (span.start, span.end),
                    },
                    span: (span.start, span.end),
                },
            )
            .padded_by(whitespace.clone());

        // Event control (e.g., @(posedge clk) or @(clk1 or clk2))
        // For now, we capture it as a placeholder expression
        let event_control = just('@').padded_by(whitespace.clone()).ignore_then(
            just('(')
                .padded_by(whitespace.clone())
                .ignore_then(
                    // Parse nested parentheses correctly
                    filter(|c| *c != ')' && *c != '(')
                        .or(just('(')
                            .then(filter(|c| *c != ')').repeated())
                            .then(just(')'))
                            .map(|_| ' '))
                        .repeated()
                        .collect::<String>()
                        .map(|s| Expression::Identifier(format!("@({})", s.trim()), (0, 0))),
                )
                .then_ignore(just(')').padded_by(whitespace.clone())),
        );

        // Global clocking declaration
        let global_clocking = text::keyword("global")
            .padded_by(whitespace.clone())
            .ignore_then(text::keyword("clocking").padded_by(whitespace.clone()))
            .ignore_then(identifier_with_span.clone().or_not())
            .then(event_control.clone())
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then_ignore(text::keyword("endclocking").padded_by(whitespace.clone()))
            .then(
                just(':')
                    .padded_by(whitespace.clone())
                    .ignore_then(identifier.clone())
                    .or_not(),
            )
            .map_with_span(
                |((opt_id, event), end_label), span: std::ops::Range<usize>| {
                    ModuleItem::GlobalClocking {
                        identifier: opt_id.as_ref().map(|(id, _)| id.clone()),
                        identifier_span: opt_id.as_ref().map(|(_, s)| *s),
                        clocking_event: event,
                        end_label,
                        span: (span.start, span.end),
                    }
                },
            )
            .padded_by(whitespace.clone());

        // Module item - order matters! Put more specific parsers first
        // class_var_decl is last because it can match any two identifiers
        let module_item = choice((
            define_directive.clone(),
            include_directive.clone(),
            class_declaration.clone(),
            global_clocking,
            concurrent_assertion,
            port_declaration,
            variable_declaration,
            assignment,
            procedural_block,
            class_var_decl, // Must be last - matches any two identifiers
        ));

        // Module body with error recovery - try to parse multiple statements
        let module_body = module_item
            .recover_with(skip_then_retry_until([';', 'a', 'i', 'o', 'e'])) // Skip to next statement or endmodule
            .repeated();

        // Module declaration
        let module_declaration = just("module")
            .padded_by(whitespace.clone())
            .ignore_then(identifier_with_span.clone())
            .then(port_list)
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then(module_body)
            .then_ignore(just("endmodule").padded_by(whitespace.clone()))
            .map_with_span(
                |(((name, name_span), ports), items), span: std::ops::Range<usize>| {
                    ModuleItem::ModuleDeclaration {
                        name: name.clone(),
                        name_span,
                        ports,
                        items,
                        span: (span.start, span.end),
                    }
                },
            )
            .padded_by(whitespace.clone());

        // Top-level items (modules, classes and preprocessor directives)
        let top_level_item = choice((
            define_directive.clone(),
            include_directive.clone(),
            class_declaration,
            module_declaration,
        ));

        // Top-level source unit
        let source_unit = top_level_item
            .repeated()
            .then_ignore(end())
            .map(|items| SourceUnit { items })
            .padded_by(whitespace);

        source_unit
    }
}
