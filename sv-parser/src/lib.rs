pub mod cli;
pub mod parser;
pub mod preprocessor;
pub mod semantic;

pub use cli::{parse_vcs_style_args, ParsedArgs};
pub use parser::SystemVerilogParser;
pub use semantic::{SemanticAnalyzer, SemanticError, SemanticErrorType};

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
pub struct ParseResult {
    pub ast: Option<SourceUnit>,
    pub errors: Vec<SingleParseError>,
}

/// Span represents a byte range in the source code (start, end)
pub type Span = (usize, usize);

/// ExprRef is an index into the ExprArena
pub type ExprRef = u32;

/// Arena for storing all Expression nodes in a flat array
/// This avoids stack overflow from deeply nested recursive structures
#[derive(Debug, Clone)]
pub struct ExprArena {
    pub nodes: Vec<Expression>,
}

impl ExprArena {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, expr: Expression) -> ExprRef {
        let idx = self.nodes.len() as u32;
        self.nodes.push(expr);
        idx
    }

    pub fn get(&self, idx: ExprRef) -> &Expression {
        &self.nodes[idx as usize]
    }

    pub fn get_mut(&mut self, idx: ExprRef) -> &mut Expression {
        &mut self.nodes[idx as usize]
    }
}

impl Default for ExprArena {
    fn default() -> Self {
        Self::new()
    }
}

/// StmtRef is an index into the StmtArena
pub type StmtRef = u32;

/// Arena for storing all Statement nodes in a flat array
/// This avoids stack overflow from deeply nested recursive structures
#[derive(Debug, Clone)]
pub struct StmtArena {
    pub nodes: Vec<Statement>,
}

impl StmtArena {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, stmt: Statement) -> StmtRef {
        let idx = self.nodes.len() as u32;
        self.nodes.push(stmt);
        idx
    }

    pub fn get(&self, idx: StmtRef) -> &Statement {
        &self.nodes[idx as usize]
    }

    pub fn get_mut(&mut self, idx: StmtRef) -> &mut Statement {
        &mut self.nodes[idx as usize]
    }
}

impl Default for StmtArena {
    fn default() -> Self {
        Self::new()
    }
}

/// ModuleItemRef is an index into the ModuleItemArena
pub type ModuleItemRef = u32;

/// Arena for storing all ModuleItem nodes in a flat array
/// This avoids stack overflow from deeply nested module structures
#[derive(Debug, Clone)]
pub struct ModuleItemArena {
    pub nodes: Vec<ModuleItem>,
}

impl ModuleItemArena {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, item: ModuleItem) -> ModuleItemRef {
        let idx = self.nodes.len() as u32;
        self.nodes.push(item);
        idx
    }

    pub fn get(&self, idx: ModuleItemRef) -> &ModuleItem {
        &self.nodes[idx as usize]
    }

    pub fn get_mut(&mut self, idx: ModuleItemRef) -> &mut ModuleItem {
        &mut self.nodes[idx as usize]
    }
}

impl Default for ModuleItemArena {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SourceUnit {
    pub items: Vec<ModuleItemRef>,
    pub expr_arena: ExprArena,
    pub stmt_arena: StmtArena,
    pub module_item_arena: ModuleItemArena,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    ModuleDeclaration {
        name: String,
        name_span: Span,
        ports: Vec<Port>,
        items: Vec<ModuleItemRef>,
        span: Span,
    },
    PortDeclaration {
        direction: PortDirection,
        port_type: String,
        name: String,
        name_span: Span,
        span: Span,
    },
    VariableDeclaration {
        data_type: String,
        signing: Option<String>,
        drive_strength: Option<DriveStrength>,
        delay: Option<Delay>,
        range: Option<Range>,
        name: String,
        name_span: Span,
        unpacked_dimensions: Vec<UnpackedDimension>,
        initial_value: Option<ExprRef>,
        span: Span,
    },
    Assignment {
        delay: Option<Delay>,
        target: ExprRef,
        expr: ExprRef,
        span: Span,
    },
    ProceduralBlock {
        block_type: ProceduralBlockType,
        statements: Vec<StmtRef>,
        span: Span,
    },
    DefineDirective {
        name: String,
        name_span: Span,
        parameters: Vec<String>, // macro parameters (e.g., for `define FOO(a, b))
        value: String,           // macro replacement text
        span: Span,
    },
    IncludeDirective {
        path: String, // the include path (with quotes or angle brackets)
        path_span: Span,
        resolved_path: Option<std::path::PathBuf>, // the resolved absolute path
        span: Span,
    },
    ClassDeclaration {
        name: String,
        name_span: Span,
        extends: Option<String>,
        items: Vec<ClassItem>,
        span: Span,
    },
    ConcurrentAssertion {
        statement: StmtRef,
        span: Span,
    },
    GlobalClocking {
        identifier: Option<String>,
        identifier_span: Option<Span>,
        clocking_event: ExprRef, // The event expression like @(posedge clk)
        end_label: Option<String>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum ClassItem {
    Property {
        qualifier: Option<ClassQualifier>,
        data_type: String,
        name: String,
        name_span: Span,
        unpacked_dimensions: Vec<UnpackedDimension>,
        initial_value: Option<ExprRef>,
        span: Span,
    },
    Method {
        qualifier: Option<ClassQualifier>,
        return_type: Option<String>, // None for void
        name: String,
        name_span: Span,
        parameters: Vec<String>, // simplified for now
        body: Vec<StmtRef>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassQualifier {
    Local,
    Protected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProceduralBlockType {
    Initial,
    Final,
    Always,
    AlwaysComb,
    AlwaysFF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,     // =
    AddAssign,  // +=
    SubAssign,  // -=
    MulAssign,  // *=
    DivAssign,  // /=
    ModAssign,  // %=
    AndAssign,  // &=
    OrAssign,   // |=
    XorAssign,  // ^=
    ShlAssign,  // <<=
    ShrAssign,  // >>=
    AShlAssign, // <<<=
    AShrAssign, // >>>=
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        target: ExprRef,
        op: AssignmentOp,
        expr: ExprRef,
        span: Span,
    },
    SystemCall {
        name: String,
        args: Vec<ExprRef>,
        span: Span,
    },
    CaseStatement {
        modifier: Option<String>, // priority, unique, or unique0
        case_type: String,        // case, casex, or casez
        expr: ExprRef,
        span: Span,
    },
    ExpressionStatement {
        expr: ExprRef,
        span: Span,
    },
    AssertProperty {
        property_expr: ExprRef,
        action_block: Option<StmtRef>,
        span: Span,
    },
    VariableDeclaration {
        data_type: String,
        name: String,
        name_span: Span,
        initial_value: Option<ExprRef>,
        span: Span,
    },
    // Placeholder for other statement types
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
    pub name_span: Span,
    pub direction: Option<PortDirection>,
    pub range: Option<Range>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub msb: String, // Most significant bit (e.g., "7" in [7:0])
    pub lsb: String, // Least significant bit (e.g., "0" in [7:0])
}

/// Represents an unpacked array dimension
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnpackedDimension {
    /// Dynamic array dimension: []
    Dynamic,
    /// Fixed-size unpacked array dimension: [N]
    FixedSize(String),
    /// Range-based unpacked array: [msb:lsb]
    Range(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStrength {
    pub strength0: String, // Strength for 0 value (e.g., "highz0", "strong0")
    pub strength1: String, // Strength for 1 value (e.g., "strong1", "pull1")
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delay {
    /// Simple delay: #10
    Value(String),
    /// Delay with expression: #(expr)
    Expression(String), // For now, store as string; could be Expression later
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String, Span),
    Number(String, Span),
    StringLiteral(String, Span),
    Binary {
        op: BinaryOp,
        left: ExprRef,
        right: ExprRef,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: ExprRef,
        span: Span,
    },
    MacroUsage {
        name: String,
        name_span: Span,
        arguments: Vec<ExprRef>,
        span: Span,
    },
    SystemFunctionCall {
        name: String,
        arguments: Vec<ExprRef>,
        span: Span,
    },
    New {
        arguments: Vec<ExprRef>,
        span: Span,
    },
    MemberAccess {
        object: ExprRef,
        member: String,
        member_span: Span,
        span: Span,
    },
    FunctionCall {
        function: ExprRef,
        arguments: Vec<ExprRef>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Modulo, // %
    And,
    Or,
    Xor,
    BitwiseXnor,          // ~^
    LogicalShiftLeft,     // <<
    LogicalShiftRight,    // >>
    ArithmeticShiftLeft,  // <<<
    ArithmeticShiftRight, // >>>
    LogicalEquiv,         // <->
    LogicalImpl,          // ->
    Equal,                // ==
    NotEqual,             // !=
    CaseEqual,            // ===
    CaseNotEqual,         // !==
    WildcardEqual,        // ==?
    WildcardNotEqual,     // !=?
    LogicalAnd,           // &&
    LogicalOr,            // ||
    GreaterThan,          // >
    LessThan,             // <
    GreaterEqual,         // >=
    LessEqual,            // <=
    Power,                // **
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
