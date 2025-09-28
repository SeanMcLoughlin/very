use chumsky::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::preprocessor::Preprocessor;
use crate::{BinaryOp, Expression, ModuleItem, ParseError, Port, PortDirection, SourceUnit};

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

        parser.parse(content).map_err(|errors| {
            // Convert chumsky errors to our ParseError
            let error_msg = errors
                .into_iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join("; ");

            ParseError {
                message: error_msg,
                location: None, // TODO: Extract location from chumsky errors
            }
        })
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

        // Simple expression parser
        let expr = recursive(|expr| {
            let atom = choice((
                identifier.clone().map(Expression::Identifier),
                number.clone().map(Expression::Number),
                expr.delimited_by(just('('), just(')')),
            ))
            .padded_by(whitespace.clone());

            let binary_op = choice((
                just("<->").to(BinaryOp::LogicalEquiv),
                just("&&").to(BinaryOp::LogicalAnd),
                just("||").to(BinaryOp::LogicalOr),
                just("->").to(BinaryOp::LogicalImpl),
                just("==").to(BinaryOp::Equal),
                just("!=").to(BinaryOp::NotEqual),
                just('+').to(BinaryOp::Add),
                just('-').to(BinaryOp::Sub),
                just('*').to(BinaryOp::Mul),
                just('/').to(BinaryOp::Div),
                just('&').to(BinaryOp::And),
                just('|').to(BinaryOp::Or),
                just('^').to(BinaryOp::Xor),
            ))
            .padded_by(whitespace.clone());

            atom.clone()
                .then(binary_op.then(atom).repeated())
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

        // Port in module header can be:
        // - just identifier: clk
        // - direction + identifier: input clk
        // - direction + type + identifier: input wire clk
        let module_port = choice((
            port_direction
                .clone()
                .then(identifier.clone()) // optional type
                .then(identifier.clone())
                .map(|((direction, _type), name)| Port {
                    name,
                    direction: Some(direction),
                }),
            port_direction
                .clone()
                .then(identifier.clone())
                .map(|(direction, name)| Port {
                    name,
                    direction: Some(direction),
                }),
            identifier.clone().map(|name| Port {
                name,
                direction: None,
            }),
        ));

        // Port declaration
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
            .padded_by(whitespace.clone());

        // Assignment statement
        let assignment = just("assign")
            .padded_by(whitespace.clone())
            .ignore_then(identifier.clone())
            .then_ignore(just('=').padded_by(whitespace.clone()))
            .then(expr.clone())
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .map(|(target, expr)| ModuleItem::Assignment { target, expr })
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

        // Module declaration
        let module_declaration = just("module")
            .padded_by(whitespace.clone())
            .ignore_then(identifier.clone())
            .then(port_list)
            .then_ignore(just(';').padded_by(whitespace.clone()))
            .then(module_item.repeated())
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
