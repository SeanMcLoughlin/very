pub mod cli;
pub mod parser;
pub mod preprocessor;

pub use cli::{parse_vcs_style_args, ParsedArgs};
pub use parser::SystemVerilogParser;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub location: Option<(usize, usize)>, // line, column
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((line, col)) = self.location {
            write!(
                f,
                "Parse error at line {}, column {}: {}",
                line, col, self.message
            )
        } else {
            write!(f, "Parse error: {}", self.message)
        }
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
    // Add more as needed
}

#[derive(Debug, Clone)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: Option<PortDirection>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    Number(String),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
}
