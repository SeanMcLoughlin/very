pub mod cli;
pub mod parser;
pub mod preprocessor;

pub use cli::{parse_vcs_style_args, ParsedArgs};
pub use parser::SystemVerilogParser;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub errors: Vec<SingleParseError>,
}

#[derive(Debug, Clone)]
pub struct SingleParseError {
    pub message: String,
    pub error_type: ParseErrorType,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,                  // 0-based line number
    pub column: usize,                // 0-based column number
    pub span: Option<(usize, usize)>, // character start/end positions
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorType {
    UnexpectedToken,
    ExpectedToken(String),
    UnexpectedEndOfInput,
    InvalidSyntax,
    UnsupportedFeature(String),
    PreprocessorError,
}

impl ParseError {
    pub fn new(error: SingleParseError) -> Self {
        Self {
            errors: vec![error],
        }
    }

    pub fn multiple(errors: Vec<SingleParseError>) -> Self {
        Self { errors }
    }

    pub fn primary_error(&self) -> &SingleParseError {
        &self.errors[0]
    }
}

impl SingleParseError {
    pub fn new(message: String, error_type: ParseErrorType) -> Self {
        Self {
            message,
            error_type,
            location: None,
            suggestions: Vec::new(),
        }
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.len() == 1 {
            write!(f, "{}", self.errors[0])
        } else {
            writeln!(f, "Multiple parse errors:")?;
            for (i, error) in self.errors.iter().enumerate() {
                write!(f, "  {}: {}", i + 1, error)?;
                if i < self.errors.len() - 1 {
                    writeln!(f)?;
                }
            }
            Ok(())
        }
    }
}

impl std::fmt::Display for SingleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.location {
            write!(
                f,
                "Error at line {}, column {}: {}",
                location.line + 1,
                location.column + 1,
                self.message
            )?;
        } else {
            write!(f, "Parse error: {}", self.message)?;
        }

        if !self.suggestions.is_empty() {
            write!(f, " (Suggestions: {})", self.suggestions.join(", "))?;
        }

        Ok(())
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub struct SourceUnit {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    ModuleDeclaration {
        name: String,
        ports: Vec<Port>,
        items: Vec<ModuleItem>,
    },
    PortDeclaration {
        direction: PortDirection,
        port_type: String,
        name: String,
    },
    Assignment {
        target: String,
        expr: Expression,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub name: String,
    pub direction: Option<PortDirection>,
    pub range: Option<Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub msb: String, // Most significant bit (e.g., "7" in [7:0])
    pub lsb: String, // Least significant bit (e.g., "0" in [7:0])
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Number(String),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    LogicalEquiv, // <->
    LogicalImpl,  // ->
    Equal,        // ==
    NotEqual,     // !=
    LogicalAnd,   // &&
    LogicalOr,    // ||
    GreaterThan,  // >
    LessThan,     // <
    GreaterEqual, // >=
    LessEqual,    // <=
    Power,        // **
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,          // +
    Minus,         // -
    Not,           // ~
    ReductionAnd,  // &
    ReductionOr,   // |
    ReductionXor,  // ^
    ReductionNand, // ~&
    ReductionNor,  // ~|
    ReductionXnor, // ~^
    LogicalNot,    // !
}
