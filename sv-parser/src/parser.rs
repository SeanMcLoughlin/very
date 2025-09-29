use chumsky::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::preprocessor::Preprocessor;
use crate::{
    BinaryOp, Expression, ModuleItem, ParseError, ParseErrorType, Port, PortDirection, Range,
    SingleParseError, SourceLocation, SourceUnit, UnaryOp,
};

#[derive(Debug)]
struct ErrorInfo {
    message: String,
    suggestions: Vec<String>,
}

pub struct SystemVerilogParser {
    preprocessor: Preprocessor,
}

impl SystemVerilogParser {
    pub fn new(include_dirs: Vec<PathBuf>, initial_macros: HashMap<String, String>) -> Self {
        Self {
            preprocessor: Preprocessor::new(include_dirs, initial_macros),
        }
    }

    pub fn parse_file(&mut self, file_path: &Path) -> Result<SourceUnit, ParseError> {
        // First preprocess the file
        let preprocessed_content = self.preprocessor.preprocess_file(file_path)?;

        // Then parse the preprocessed content
        self.parse_content(&preprocessed_content)
    }

    pub fn parse_content(&self, content: &str) -> Result<SourceUnit, ParseError> {
        let parser = self.source_unit_parser();

        parser.parse(content).map_err(|chumsky_errors| {
            let mut parse_errors = Vec::new();

            // Debug: Show how many errors chumsky found
            // eprintln!("DEBUG: Chumsky found {} errors", chumsky_errors.len());

            // Process all errors from chumsky
            for error in chumsky_errors {
                let location = self.span_to_location(error.span(), content);
                let (message, error_type, suggestions) =
                    self.convert_chumsky_error(&error, content);

                let mut single_error = SingleParseError::new(message, error_type);

                if let Some(loc) = location {
                    single_error = single_error.with_location(loc);
                }

                if !suggestions.is_empty() {
                    single_error = single_error.with_suggestions(suggestions);
                }

                parse_errors.push(single_error);
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

                // Try to find additional errors that the parser missed
                self.find_additional_errors(content, &mut parse_errors);

                ParseError::multiple(parse_errors)
            }
        })
    }

    fn find_additional_errors(&self, content: &str, errors: &mut Vec<SingleParseError>) {
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for lines that contain obvious syntax errors
            if let Some(error_info) = self.analyze_line_for_errors(trimmed) {
                // Check if we already have an error for this line
                let has_error_on_line = errors.iter().any(|e| {
                    if let Some(loc) = &e.location {
                        loc.line == line_idx
                    } else {
                        false
                    }
                });

                if !has_error_on_line {
                    let error =
                        SingleParseError::new(error_info.message, ParseErrorType::InvalidSyntax)
                            .with_location(SourceLocation {
                                line: line_idx,
                                column: 0,
                                span: None,
                            })
                            .with_suggestions(error_info.suggestions);

                    errors.push(error);
                }
            }
        }
    }

    fn analyze_line_for_errors(&self, line: &str) -> Option<ErrorInfo> {
        let trimmed = line.trim();

        // Skip empty lines, comments, and known valid constructs
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("module")
            || trimmed.starts_with("endmodule")
            || trimmed.starts_with("assign")
            || trimmed.starts_with("input")
            || trimmed.starts_with("output")
            || trimmed.starts_with("inout")
        {
            return None;
        }

        // Tokenize the line to get actual identifiers
        let tokens = self.simple_tokenize(trimmed);

        // Check if the first token is an unknown identifier
        if let Some(first_token) = tokens.first() {
            if self.looks_like_invalid_statement(&tokens) {
                return Some(ErrorInfo {
                    message: format!(
                        "Invalid statement '{}', expected valid SystemVerilog statement",
                        first_token
                    ),
                    suggestions: vec![format!(
                        "Replace '{}' with a valid SystemVerilog keyword",
                        first_token
                    )],
                });
            }
        }

        None
    }

    fn simple_tokenize(&self, line: &str) -> Vec<String> {
        line.split_whitespace()
            .map(|token| token.trim_end_matches(';').to_string())
            .collect()
    }

    fn looks_like_invalid_statement(&self, tokens: &[String]) -> bool {
        if tokens.is_empty() {
            return false;
        }

        let first_token = &tokens[0];

        // Check if it's an unknown identifier that's not a SystemVerilog keyword
        if tokens.len() >= 2
            && !self.is_known_systemverilog_keyword(first_token)
            && first_token.chars().all(|c| c.is_alphabetic() || c == '_')
        {
            return true;
        }

        false
    }

    fn detect_invalid_statement(
        &self,
        error_span: std::ops::Range<usize>,
        content: &str,
        _found: &str,
    ) -> Option<(String, ParseErrorType, Vec<String>)> {
        // Get the line containing the error
        if error_span.start >= content.len() {
            return None;
        }

        let prefix = &content[..error_span.start];
        let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);

        // Find the end of the current line
        let suffix = &content[error_span.start..];
        let line_end = suffix
            .find('\n')
            .map(|i| error_span.start + i)
            .unwrap_or(content.len());

        let current_line = &content[line_start..line_end];
        let trimmed_line = current_line.trim();

        // Check if this line looks like an invalid statement
        if self.is_invalid_statement_line(trimmed_line) {
            let invalid_token = trimmed_line
                .split_whitespace()
                .next()
                .unwrap_or(trimmed_line);

            return Some((
                format!(
                    "Invalid statement '{}', expected valid SystemVerilog statement",
                    invalid_token
                ),
                ParseErrorType::InvalidSyntax,
                vec![
                    "Use 'assign' for continuous assignments".to_string(),
                    "Use 'input', 'output', or 'inout' for port declarations".to_string(),
                    "Use 'always' or 'always_ff' for procedural blocks".to_string(),
                ],
            ));
        }

        None
    }

    fn is_invalid_statement_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return false;
        }

        // Skip known valid statement beginnings
        if trimmed.starts_with("module")
            || trimmed.starts_with("endmodule")
            || trimmed.starts_with("assign")
            || trimmed.starts_with("input")
            || trimmed.starts_with("output")
            || trimmed.starts_with("inout")
            || trimmed.starts_with("always")
            || trimmed.starts_with("initial")
        {
            return false;
        }

        // Check for obvious invalid patterns
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // If the first word is not a SystemVerilog keyword and contains only letters/underscores,
        // it's likely an invalid statement
        if !first_word.is_empty()
            && first_word.chars().all(|c| c.is_alphabetic() || c == '_')
            && !self.is_known_systemverilog_keyword(first_word)
        {
            return true;
        }

        // Special case: if it starts with a keyword but isn't a valid statement structure
        // For example: "this is invalid" - "this" is a keyword but this isn't valid syntax
        if first_word == "this" && trimmed.split_whitespace().count() > 1 {
            let second_word = trimmed.split_whitespace().nth(1).unwrap_or("");
            // "this" should be followed by specific keywords in valid contexts
            if !matches!(second_word, "new" | "super" | "randomize") {
                return true;
            }
        }

        false
    }

    fn is_known_systemverilog_keyword(&self, word: &str) -> bool {
        matches!(
            word,
            "module"
                | "endmodule"
                | "assign"
                | "input"
                | "output"
                | "inout"
                | "wire"
                | "reg"
                | "logic"
                | "always"
                | "initial"
                | "begin"
                | "end"
                | "if"
                | "else"
                | "case"
                | "default"
                | "for"
                | "while"
                | "function"
                | "task"
                | "return"
                | "class"
                | "interface"
                | "package"
                | "import"
                | "export"
                | "extends"
                | "implements"
                | "virtual"
                | "static"
                | "local"
                | "protected"
                | "private"
                | "public"
                | "pure"
                | "extern"
                | "typedef"
                | "enum"
                | "struct"
                | "union"
                | "packed"
                | "unpacked"
                | "signed"
                | "unsigned"
                | "bit"
                | "byte"
                | "shortint"
                | "int"
                | "longint"
                | "integer"
                | "time"
                | "real"
                | "shortreal"
                | "string"
                | "event"
                | "chandle"
                | "void"
                | "null"
                | "this"
                | "super"
                | "randomize"
                | "constraint"
                | "solve"
                | "before"
                | "inside"
                | "foreach"
                | "repeat"
                | "forever"
                | "do"
                | "break"
                | "continue"
                | "unique"
                | "priority"
                | "cover"
                | "covergroup"
                | "coverpoint"
                | "cross"
                | "bins"
                | "illegal_bins"
                | "ignore_bins"
                | "with"
                | "matches"
                | "assert"
                | "assume"
                | "restrict"
                | "expect"
                | "disable"
                | "iff"
                | "property"
                | "sequence"
                | "clocking"
                | "modport"
                | "forkjoin"
                | "join_any"
                | "join_none"
                | "wait"
                | "wait_order"
                | "final"
                | "bind"
                | "alias"
                | "generate"
                | "genvar"
                | "localparam"
                | "parameter"
                | "defparam"
                | "specify"
                | "specparam"
                | "timescale"
                | "include"
                | "define"
                | "ifdef"
                | "ifndef"
                | "elsif"
                | "endif"
                | "undef"
                | "undefineall"
                | "celldefine"
                | "endcelldefine"
                | "default_nettype"
                | "resetall"
                | "line"
                | "pragma"
        )
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
    ) -> (String, ParseErrorType, Vec<String>) {
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
            ),
            chumsky::error::SimpleReason::Custom(msg) => {
                (msg.clone(), ParseErrorType::InvalidSyntax, Vec::new())
            }
        }
    }

    fn analyze_parse_context(
        &self,
        found: &str,
        expected: &[String],
        error_span: std::ops::Range<usize>,
        content: &str,
    ) -> (String, ParseErrorType, Vec<String>) {
        // Debug: Let's see what we're actually expecting
        // eprintln!("DEBUG: found={}, expected={:?}", found, expected);

        // First, try to detect if this is an invalid statement by looking at the context
        if let Some(invalid_statement_error) =
            self.detect_invalid_statement(error_span, content, found)
        {
            return invalid_statement_error;
        }

        // Special case: if we're expecting 'n' and found newline/end, it's likely missing "endmodule"
        if expected.contains(&"'n'".to_string()) {
            if found == "end of input" || found == "'\n'" {
                return (
                    "Missing 'endmodule' to close module declaration".to_string(),
                    ParseErrorType::ExpectedToken("endmodule".to_string()),
                    vec!["Add 'endmodule' to complete the module".to_string()],
                );
            } else {
                return (
                    format!("Expected 'endmodule', found unexpected character {}", found),
                    ParseErrorType::UnexpectedToken,
                    vec!["Replace with 'endmodule' to close the module".to_string()],
                );
            }
        }

        // Check if we're at end of input and expecting statement terminators
        if found == "end of input" && expected.contains(&"';'".to_string()) {
            return (
                "Missing semicolon at end of statement".to_string(),
                ParseErrorType::ExpectedToken(";".to_string()),
                vec!["Add ';' to complete the statement".to_string()],
            );
        }

        // Check if we're expecting characters that form other keywords
        if self.expects_keyword_pattern(expected) {
            return self.suggest_keyword_completion(found, expected);
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
                );
            }
        }

        // Fallback to generic message
        let message = if expected.is_empty() {
            format!("Unexpected {}", found)
        } else {
            format!("Unexpected {}", found)
        };

        (message, ParseErrorType::UnexpectedToken, Vec::new())
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
        let identifier = filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
            .then(filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_').repeated())
            .map(|(first, rest): (char, Vec<char>)| {
                let mut result = String::new();
                result.push(first);
                result.extend(rest);
                result
            })
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

        // Expression parser with unary and binary operators
        let expr = recursive(|expr| {
            let atom = choice((
                identifier.clone().map(Expression::Identifier),
                number.clone().map(Expression::Number),
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

            // Factor handles unary operators and atoms
            let factor = choice((
                unary_op
                    .clone()
                    .then(expr.clone())
                    .map(|(op, operand)| Expression::Unary {
                        op,
                        operand: Box::new(operand),
                    }),
                atom.clone(),
            ))
            .padded_by(whitespace.clone());

            let binary_op = choice((
                just("<->").to(BinaryOp::LogicalEquiv),
                just("**").to(BinaryOp::Power),
                just("<=").to(BinaryOp::LessEqual),
                just(">=").to(BinaryOp::GreaterEqual),
                just("&&").to(BinaryOp::LogicalAnd),
                just("||").to(BinaryOp::LogicalOr),
                just("->").to(BinaryOp::LogicalImpl),
                just("==").to(BinaryOp::Equal),
                just("!=").to(BinaryOp::NotEqual),
                just('<').to(BinaryOp::LessThan),
                just('>').to(BinaryOp::GreaterThan),
                just('+').to(BinaryOp::Add),
                just('-').to(BinaryOp::Sub),
                just('*').to(BinaryOp::Mul),
                just('/').to(BinaryOp::Div),
                just('&').to(BinaryOp::And),
                just('|').to(BinaryOp::Or),
                just('^').to(BinaryOp::Xor),
            ))
            .padded_by(whitespace.clone());

            factor
                .clone()
                .then(binary_op.then(factor).repeated())
                .foldl(|left, (op, right)| Expression::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
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
                .then(identifier.clone()) // name
                .map(|(((direction, _type), range), name)| Port {
                    name,
                    direction: Some(direction),
                    range,
                }),
            // direction + range + identifier: input [3:0] clk
            port_direction
                .clone()
                .then(range.clone().or_not())
                .then(identifier.clone()) // name
                .map(|((direction, range), name)| Port {
                    name,
                    direction: Some(direction),
                    range,
                }),
            // just identifier: clk
            identifier.clone().map(|name| Port {
                name,
                direction: None,
                range: None,
            }),
        ));

        // Port declaration with error recovery
        let port_declaration = port_direction
            .then(identifier.clone()) // port type (like wire, reg)
            .then(identifier.clone()) // port name
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map(
                |((direction, port_type), name)| ModuleItem::PortDeclaration {
                    direction,
                    port_type,
                    name,
                },
            )
            .recover_with(skip_then_retry_until([';']))
            .padded_by(whitespace.clone());

        // Assignment statement with error recovery
        let assignment = just("assign")
            .padded_by(whitespace.clone())
            .ignore_then(identifier.clone())
            .then_ignore(just('=').padded_by(whitespace.clone()))
            .then(expr.clone())
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map(|(target, expr)| ModuleItem::Assignment { target, expr })
            .recover_with(skip_then_retry_until([';']))
            .padded_by(whitespace.clone());

        // Port list in module header
        let port_list = module_port
            .separated_by(just(',').padded_by(whitespace.clone()))
            .delimited_by(just('('), just(')'))
            .or_not()
            .map(|ports| ports.unwrap_or_default())
            .padded_by(whitespace.clone());

        // Module item
        let module_item = choice((port_declaration, assignment));

        // Module body with error recovery - try to parse multiple statements
        let module_body = module_item
            .recover_with(skip_then_retry_until([';', 'a', 'i', 'o', 'e'])) // Skip to next statement or endmodule
            .repeated();

        // Module declaration
        let module_declaration = just("module")
            .padded_by(whitespace.clone())
            .ignore_then(identifier.clone())
            .then(port_list)
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then(module_body)
            .then_ignore(just("endmodule").padded_by(whitespace.clone()))
            .map(|((name, ports), items)| ModuleItem::ModuleDeclaration { name, ports, items })
            .padded_by(whitespace.clone());

        // Top-level source unit
        let source_unit = module_declaration
            .repeated()
            .then_ignore(end())
            .map(|items| SourceUnit { items })
            .padded_by(whitespace);

        source_unit
    }
}
