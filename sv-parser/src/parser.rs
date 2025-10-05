use chumsky::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::preprocessor::Preprocessor;
use crate::{
    AssignmentOp, BinaryOp, ClassItem, ClassQualifier, Delay, DriveStrength, ExprArena, ExprRef,
    Expression, ModuleItem, ModuleItemArena, ModuleItemRef, ParseError, ParseErrorType, Port,
    PortDirection, ProceduralBlockType, Range, SingleParseError, SourceUnit, Span, Statement,
    StmtArena, StmtRef, UnaryOp, UnpackedDimension,
};

/// Temporary expression type used during parsing with Box-based recursion
/// After parsing, this gets flattened into Expression + ExprArena
#[derive(Clone, PartialEq)]
enum ParsedExpression {
    Identifier(String, Span),
    Number(String, Span),
    StringLiteral(String, Span),
    Binary {
        op: BinaryOp,
        left: Box<ParsedExpression>,
        right: Box<ParsedExpression>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<ParsedExpression>,
        span: Span,
    },
    #[allow(dead_code)]
    MacroUsage {
        name: String,
        name_span: Span,
        arguments: Vec<ParsedExpression>,
        span: Span,
    },
    SystemFunctionCall {
        name: String,
        arguments: Vec<ParsedExpression>,
        span: Span,
    },
    New {
        arguments: Vec<ParsedExpression>,
        span: Span,
    },
    MemberAccess {
        object: Box<ParsedExpression>,
        member: String,
        member_span: Span,
        span: Span,
    },
    FunctionCall {
        function: Box<ParsedExpression>,
        arguments: Vec<ParsedExpression>,
        span: Span,
    },
}

impl ParsedExpression {
    /// Flatten this expression tree into an arena and return the root ExprRef
    fn flatten(self, arena: &mut ExprArena) -> ExprRef {
        match self {
            ParsedExpression::Identifier(name, span) => {
                arena.alloc(Expression::Identifier(name, span))
            }
            ParsedExpression::Number(num, span) => arena.alloc(Expression::Number(num, span)),
            ParsedExpression::StringLiteral(s, span) => {
                arena.alloc(Expression::StringLiteral(s, span))
            }
            ParsedExpression::Binary {
                op,
                left,
                right,
                span,
            } => {
                let left_ref = left.flatten(arena);
                let right_ref = right.flatten(arena);
                arena.alloc(Expression::Binary {
                    op,
                    left: left_ref,
                    right: right_ref,
                    span,
                })
            }
            ParsedExpression::Unary { op, operand, span } => {
                let operand_ref = operand.flatten(arena);
                arena.alloc(Expression::Unary {
                    op,
                    operand: operand_ref,
                    span,
                })
            }
            ParsedExpression::MacroUsage {
                name,
                name_span,
                arguments,
                span,
            } => {
                let arg_refs: Vec<ExprRef> =
                    arguments.into_iter().map(|a| a.flatten(arena)).collect();
                arena.alloc(Expression::MacroUsage {
                    name,
                    name_span,
                    arguments: arg_refs,
                    span,
                })
            }
            ParsedExpression::SystemFunctionCall {
                name,
                arguments,
                span,
            } => {
                let arg_refs: Vec<ExprRef> =
                    arguments.into_iter().map(|a| a.flatten(arena)).collect();
                arena.alloc(Expression::SystemFunctionCall {
                    name,
                    arguments: arg_refs,
                    span,
                })
            }
            ParsedExpression::New { arguments, span } => {
                let arg_refs: Vec<ExprRef> =
                    arguments.into_iter().map(|a| a.flatten(arena)).collect();
                arena.alloc(Expression::New {
                    arguments: arg_refs,
                    span,
                })
            }
            ParsedExpression::MemberAccess {
                object,
                member,
                member_span,
                span,
            } => {
                let object_ref = object.flatten(arena);
                arena.alloc(Expression::MemberAccess {
                    object: object_ref,
                    member,
                    member_span,
                    span,
                })
            }
            ParsedExpression::FunctionCall {
                function,
                arguments,
                span,
            } => {
                let function_ref = function.flatten(arena);
                let arg_refs: Vec<ExprRef> =
                    arguments.into_iter().map(|a| a.flatten(arena)).collect();
                arena.alloc(Expression::FunctionCall {
                    function: function_ref,
                    arguments: arg_refs,
                    span,
                })
            }
        }
    }
}

/// Temporary statement that holds ParsedExpressions during parsing
#[derive(Clone)]
enum ParsedStatement {
    Assignment {
        target: ParsedExpression,
        op: AssignmentOp,
        expr: ParsedExpression,
    },
    SystemCall {
        name: String,
        args: Vec<ParsedExpression>,
        span: Span,
    },
    CaseStatement {
        modifier: Option<String>,
        case_type: String,
        expr: ParsedExpression,
    },
    AssertProperty {
        property_expr: ParsedExpression,
        action_block: Option<Box<ParsedStatement>>,
    },
    ExpressionStatement {
        expr: ParsedExpression,
    },
    VariableDeclaration {
        data_type: String,
        name: String,
        name_span: Span,
        initial_value: Option<ParsedExpression>,
        span: Span,
    },
}

impl ParsedStatement {
    fn flatten(self, expr_arena: &mut ExprArena, _stmt_arena: &mut StmtArena) -> Statement {
        match self {
            ParsedStatement::Assignment { target, op, expr } => {
                let target_ref = target.flatten(expr_arena);
                let expr_ref = expr.flatten(expr_arena);
                Statement::Assignment {
                    target: target_ref,
                    op,
                    expr: expr_ref,
                    span: (0, 0),
                }
            }
            ParsedStatement::SystemCall { name, args, span } => {
                let arg_refs = args.into_iter().map(|a| a.flatten(expr_arena)).collect();
                Statement::SystemCall {
                    name,
                    args: arg_refs,
                    span,
                }
            }
            ParsedStatement::CaseStatement {
                modifier,
                case_type,
                expr,
            } => {
                let expr_ref = expr.flatten(expr_arena);
                Statement::CaseStatement {
                    modifier,
                    case_type,
                    expr: expr_ref,
                    span: (0, 0),
                }
            }
            ParsedStatement::AssertProperty {
                property_expr,
                action_block,
            } => {
                let property_ref = property_expr.flatten(expr_arena);
                let action_ref = action_block.map(|stmt| {
                    let flattened = stmt.flatten(expr_arena, _stmt_arena);
                    _stmt_arena.alloc(flattened)
                });
                Statement::AssertProperty {
                    property_expr: property_ref,
                    action_block: action_ref,
                    span: (0, 0),
                }
            }
            ParsedStatement::ExpressionStatement { expr } => {
                let expr_ref = expr.flatten(expr_arena);
                Statement::ExpressionStatement {
                    expr: expr_ref,
                    span: (0, 0),
                }
            }
            ParsedStatement::VariableDeclaration {
                data_type,
                name,
                name_span,
                initial_value,
                span,
            } => {
                let initial_value_ref = initial_value.map(|expr| expr.flatten(expr_arena));
                Statement::VariableDeclaration {
                    data_type,
                    name,
                    name_span,
                    initial_value: initial_value_ref,
                    span,
                }
            }
        }
    }
}

/// Temporary class item that holds ParsedExpressions during parsing
#[derive(Clone)]
enum ParsedClassItem {
    Property {
        qualifier: Option<ClassQualifier>,
        data_type: String,
        name: String,
        unpacked_dimensions: Vec<UnpackedDimension>,
        initial_value: Option<ParsedExpression>,
    },
    Method {
        qualifier: Option<ClassQualifier>,
        return_type: Option<String>,
        name: String,
        parameters: Vec<String>,
        body: Vec<ParsedStatement>,
    },
}

impl ParsedClassItem {
    fn flatten(self, expr_arena: &mut ExprArena, stmt_arena: &mut StmtArena) -> ClassItem {
        match self {
            ParsedClassItem::Property {
                qualifier,
                data_type,
                name,
                unpacked_dimensions,
                initial_value,
            } => ClassItem::Property {
                qualifier,
                data_type,
                name,
                name_span: (0, 0),
                unpacked_dimensions,
                initial_value: initial_value.map(|e| e.flatten(expr_arena)),
                span: (0, 0),
            },
            ParsedClassItem::Method {
                qualifier,
                return_type,
                name,
                parameters,
                body,
            } => {
                let body_refs: Vec<StmtRef> = body
                    .into_iter()
                    .map(|s| {
                        let stmt = s.flatten(expr_arena, stmt_arena);
                        stmt_arena.alloc(stmt)
                    })
                    .collect();
                ClassItem::Method {
                    qualifier,
                    return_type,
                    name,
                    name_span: (0, 0),
                    parameters,
                    body: body_refs,
                    span: (0, 0),
                }
            }
        }
    }
}

/// Temporary module item that holds ParsedExpressions during parsing
#[derive(Clone)]
enum ParsedModuleItem {
    ModuleDeclaration {
        name: String,
        name_span: Span,
        ports: Vec<Port>,
        items: Vec<ParsedModuleItem>,
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
        initial_value: Option<ParsedExpression>,
        span: Span,
    },
    Assignment {
        delay: Option<Delay>,
        target: ParsedExpression,
        expr: ParsedExpression,
        span: Span,
    },
    ProceduralBlock {
        block_type: ProceduralBlockType,
        statements: Vec<ParsedStatement>,
        span: Span,
    },
    ClassDeclaration {
        name: String,
        name_span: Span,
        extends: Option<String>,
        items: Vec<ParsedClassItem>,
        span: Span,
    },
    PortDeclaration {
        direction: PortDirection,
        port_type: String,
        name: String,
        name_span: Span,
        span: Span,
    },
    DefineDirective {
        name: String,
        name_span: Span,
        parameters: Vec<String>,
        value: String,
        span: Span,
    },
    IncludeDirective {
        path: String,
        path_span: Span,
        span: Span,
    },
    ConcurrentAssertion {
        statement: ParsedStatement,
        span: Span,
    },
    GlobalClocking {
        identifier: Option<String>,
        identifier_span: Option<Span>,
        clocking_event: ParsedExpression,
        end_label: Option<String>,
        span: Span,
    },
}

impl ParsedModuleItem {
    /// Flatten this parsed module item into a real ModuleItem + arena
    fn flatten(
        self,
        expr_arena: &mut ExprArena,
        stmt_arena: &mut StmtArena,
        module_item_arena: &mut ModuleItemArena,
    ) -> ModuleItem {
        match self {
            ParsedModuleItem::ModuleDeclaration {
                name,
                name_span,
                ports,
                items,
                span,
            } => {
                // First flatten all child items into ModuleItems
                let flattened_items: Vec<ModuleItem> = items
                    .into_iter()
                    .map(|item| item.flatten(expr_arena, stmt_arena, module_item_arena))
                    .collect();

                // Then allocate them in the arena and collect their refs
                let item_refs: Vec<ModuleItemRef> = flattened_items
                    .into_iter()
                    .map(|item| module_item_arena.alloc(item))
                    .collect();

                ModuleItem::ModuleDeclaration {
                    name,
                    name_span,
                    ports,
                    items: item_refs,
                    span,
                }
            }
            ParsedModuleItem::VariableDeclaration {
                data_type,
                signing,
                drive_strength,
                delay,
                range,
                name,
                name_span,
                unpacked_dimensions,
                initial_value,
                span,
            } => ModuleItem::VariableDeclaration {
                data_type,
                signing,
                drive_strength,
                delay,
                range,
                name,
                name_span,
                unpacked_dimensions,
                initial_value: initial_value.map(|e| e.flatten(expr_arena)),
                span,
            },
            ParsedModuleItem::Assignment {
                delay,
                target,
                expr,
                span,
            } => {
                let target_ref = target.flatten(expr_arena);
                let expr_ref = expr.flatten(expr_arena);
                ModuleItem::Assignment {
                    delay,
                    target: target_ref,
                    expr: expr_ref,
                    span,
                }
            }
            ParsedModuleItem::ProceduralBlock {
                block_type,
                statements,
                span,
            } => {
                let statement_refs: Vec<StmtRef> = statements
                    .into_iter()
                    .map(|s| {
                        let stmt = s.flatten(expr_arena, stmt_arena);
                        stmt_arena.alloc(stmt)
                    })
                    .collect();
                ModuleItem::ProceduralBlock {
                    block_type,
                    statements: statement_refs,
                    span,
                }
            }
            ParsedModuleItem::ClassDeclaration {
                name,
                name_span,
                extends,
                items,
                span,
            } => {
                let flattened_items: Vec<ClassItem> = items
                    .into_iter()
                    .map(|item| item.flatten(expr_arena, stmt_arena))
                    .collect();
                ModuleItem::ClassDeclaration {
                    name,
                    name_span,
                    extends,
                    items: flattened_items,
                    span,
                }
            }
            ParsedModuleItem::PortDeclaration {
                direction,
                port_type,
                name,
                name_span,
                span,
            } => ModuleItem::PortDeclaration {
                direction,
                port_type,
                name,
                name_span,
                span,
            },
            ParsedModuleItem::DefineDirective {
                name,
                name_span,
                parameters,
                value,
                span,
            } => ModuleItem::DefineDirective {
                name,
                name_span,
                parameters,
                value,
                span,
            },
            ParsedModuleItem::IncludeDirective {
                path,
                path_span,
                span,
            } => ModuleItem::IncludeDirective {
                path,
                path_span,
                resolved_path: None,
                span,
            },
            ParsedModuleItem::ConcurrentAssertion { statement, span } => {
                let stmt = statement.flatten(expr_arena, stmt_arena);
                let stmt_ref = stmt_arena.alloc(stmt);
                ModuleItem::ConcurrentAssertion {
                    statement: stmt_ref,
                    span,
                }
            }
            ParsedModuleItem::GlobalClocking {
                identifier,
                identifier_span,
                clocking_event,
                end_label,
                span,
            } => {
                let event_ref = clocking_event.flatten(expr_arena);
                ModuleItem::GlobalClocking {
                    identifier,
                    identifier_span,
                    clocking_event: event_ref,
                    end_label,
                    span,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SystemVerilogParser {
    preprocessor: Preprocessor,
    #[allow(dead_code)]
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
        let mut included_files = std::collections::HashSet::new();
        self.parse_file_with_includes(file_path, &mut included_files)
    }

    fn parse_file_with_includes(
        &mut self,
        file_path: &Path,
        included_files: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> Result<SourceUnit, ParseError> {
        // Canonicalize the file path to detect circular includes
        let canonical_path = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());

        // Check for circular includes
        if included_files.contains(&canonical_path) {
            // Already included, return empty AST to avoid infinite recursion
            return Ok(SourceUnit {
                items: Vec::new(),
                expr_arena: ExprArena::new(),
                stmt_arena: StmtArena::new(),
                module_item_arena: ModuleItemArena::new(),
            });
        }

        included_files.insert(canonical_path.clone());

        let raw_content = std::fs::read_to_string(file_path).map_err(|e| {
            ParseError::new(SingleParseError::new(
                format!("Failed to read file {}: {}", file_path.display(), e),
                ParseErrorType::PreprocessorError,
            ))
        })?;

        let mut ast = self.parse_content(&raw_content)?;
        self.expand_includes_in_ast(&mut ast, file_path, included_files)?;
        Ok(ast)
    }

    fn expand_includes_in_ast(
        &mut self,
        ast: &mut SourceUnit,
        current_file: &Path,
        included_files: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> Result<(), ParseError> {
        let mut i = 0;
        while i < ast.items.len() {
            let item_ref = ast.items[i];
            let item = ast.module_item_arena.get(item_ref);

            // Check if this is an include directive
            if let ModuleItem::IncludeDirective { path, .. } = item {
                let include_path = path.clone();

                // Resolve the include path
                let resolved_path = self.resolve_include_path(&include_path, current_file)?;

                // Parse the included file
                let included_ast = self.parse_file_with_includes(&resolved_path, included_files)?;

                // Remove the include directive from the AST
                ast.items.remove(i);

                // Merge the included AST into the current AST
                // First, we need to copy the arenas and remap references
                let item_offset = ast.module_item_arena.nodes.len() as u32;
                let expr_offset = ast.expr_arena.nodes.len() as u32;
                let stmt_offset = ast.stmt_arena.nodes.len() as u32;

                // Merge arenas
                ast.expr_arena.nodes.extend(included_ast.expr_arena.nodes);
                ast.stmt_arena.nodes.extend(included_ast.stmt_arena.nodes);

                // Copy and remap module items
                for included_item in included_ast.module_item_arena.nodes {
                    let remapped_item =
                        Self::remap_item(included_item, expr_offset, stmt_offset, item_offset);
                    ast.module_item_arena.nodes.push(remapped_item);
                }

                // Insert the included items into the current position
                for included_item_ref in included_ast.items {
                    ast.items.insert(i, included_item_ref + item_offset);
                    i += 1;
                }

                // Continue processing from the current position
                // (don't increment i, as we've already advanced it)
            } else {
                // Not an include directive, check if it's a module with nested includes
                self.expand_includes_in_module(item_ref, current_file, ast, included_files)?;
                i += 1;
            }
        }
        Ok(())
    }

    fn expand_includes_in_module(
        &mut self,
        item_ref: ModuleItemRef,
        current_file: &Path,
        ast: &mut SourceUnit,
        included_files: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> Result<(), ParseError> {
        let item = ast.module_item_arena.get(item_ref);

        if let ModuleItem::ModuleDeclaration { items, .. } = item {
            let nested_items = items.clone();
            let _ = item; // Release the borrow

            let mut new_items = Vec::new();

            for &nested_ref in &nested_items {
                let nested_item = ast.module_item_arena.get(nested_ref);

                if let ModuleItem::IncludeDirective { path, .. } = nested_item {
                    let include_path = path.clone();
                    let _ = nested_item;

                    // Resolve and parse the included file
                    let resolved_path = self.resolve_include_path(&include_path, current_file)?;
                    let included_ast =
                        self.parse_file_with_includes(&resolved_path, included_files)?;

                    // Merge the included AST
                    let item_offset = ast.module_item_arena.nodes.len() as u32;
                    let expr_offset = ast.expr_arena.nodes.len() as u32;
                    let stmt_offset = ast.stmt_arena.nodes.len() as u32;

                    ast.expr_arena.nodes.extend(included_ast.expr_arena.nodes);
                    ast.stmt_arena.nodes.extend(included_ast.stmt_arena.nodes);

                    for included_item in included_ast.module_item_arena.nodes {
                        let remapped_item =
                            Self::remap_item(included_item, expr_offset, stmt_offset, item_offset);
                        ast.module_item_arena.nodes.push(remapped_item);
                    }

                    for included_item_ref in included_ast.items {
                        new_items.push(included_item_ref + item_offset);
                    }
                } else {
                    new_items.push(nested_ref);
                }
            }

            // Update the module's items
            let item_mut = ast.module_item_arena.get_mut(item_ref);
            if let ModuleItem::ModuleDeclaration { items, .. } = item_mut {
                *items = new_items.clone();
            }

            // Now recursively process nested modules
            for &nested_ref in &new_items {
                self.expand_includes_in_module(nested_ref, current_file, ast, included_files)?;
            }
        }
        Ok(())
    }

    fn remap_item(
        item: ModuleItem,
        expr_offset: u32,
        stmt_offset: u32,
        item_offset: u32,
    ) -> ModuleItem {
        match item {
            ModuleItem::ModuleDeclaration {
                name,
                name_span,
                ports,
                items,
                span,
            } => ModuleItem::ModuleDeclaration {
                name,
                name_span,
                ports,
                items: items.into_iter().map(|r| r + item_offset).collect(),
                span,
            },
            ModuleItem::VariableDeclaration {
                data_type,
                signing,
                drive_strength,
                delay,
                range,
                name,
                name_span,
                unpacked_dimensions,
                initial_value,
                span,
            } => ModuleItem::VariableDeclaration {
                data_type,
                signing,
                drive_strength,
                delay,
                range,
                name,
                name_span,
                unpacked_dimensions,
                initial_value: initial_value.map(|r| r + expr_offset),
                span,
            },
            ModuleItem::Assignment {
                delay,
                target,
                expr,
                span,
            } => ModuleItem::Assignment {
                delay,
                target: target + expr_offset,
                expr: expr + expr_offset,
                span,
            },
            ModuleItem::ProceduralBlock {
                block_type,
                statements,
                span,
            } => ModuleItem::ProceduralBlock {
                block_type,
                statements: statements.into_iter().map(|r| r + stmt_offset).collect(),
                span,
            },
            ModuleItem::ClassDeclaration {
                name,
                name_span,
                extends,
                items,
                span,
            } => {
                // Class items may contain expression references too
                let remapped_items = items
                    .into_iter()
                    .map(|class_item| match class_item {
                        ClassItem::Property {
                            qualifier,
                            data_type,
                            name,
                            name_span,
                            unpacked_dimensions,
                            initial_value,
                            span,
                        } => ClassItem::Property {
                            qualifier,
                            data_type,
                            name,
                            name_span,
                            unpacked_dimensions,
                            initial_value: initial_value.map(|r| r + expr_offset),
                            span,
                        },
                        ClassItem::Method {
                            qualifier,
                            return_type,
                            name,
                            name_span,
                            parameters,
                            body,
                            span,
                        } => ClassItem::Method {
                            qualifier,
                            return_type,
                            name,
                            name_span,
                            parameters,
                            body: body.into_iter().map(|r| r + stmt_offset).collect(),
                            span,
                        },
                    })
                    .collect();

                ModuleItem::ClassDeclaration {
                    name,
                    name_span,
                    extends,
                    items: remapped_items,
                    span,
                }
            }
            ModuleItem::ConcurrentAssertion { statement, span } => {
                ModuleItem::ConcurrentAssertion {
                    statement: statement + stmt_offset,
                    span,
                }
            }
            ModuleItem::GlobalClocking {
                identifier,
                identifier_span,
                clocking_event,
                end_label,
                span,
            } => ModuleItem::GlobalClocking {
                identifier,
                identifier_span,
                clocking_event: clocking_event + expr_offset,
                end_label,
                span,
            },
            // Items that don't need remapping
            other => other,
        }
    }

    fn resolve_include_path(
        &self,
        filename: &str,
        current_file: &Path,
    ) -> Result<PathBuf, ParseError> {
        let mut found_path = None;

        if let Some(parent) = current_file.parent() {
            let candidate = parent.join(filename);
            if candidate.exists() {
                found_path = Some(candidate);
            }
        }

        if found_path.is_none() {
            for include_dir in &self.preprocessor.include_dirs {
                let candidate = include_dir.join(filename);
                if candidate.exists() {
                    found_path = Some(candidate);
                    break;
                }
            }
        }

        found_path.ok_or_else(|| {
            ParseError::new(SingleParseError::new(
                format!("Include file '{}' not found", filename),
                ParseErrorType::PreprocessorError,
            ))
        })
    }

    pub fn parse_content(&self, content: &str) -> Result<SourceUnit, ParseError> {
        let mut expr_arena = ExprArena::new();
        let mut stmt_arena = StmtArena::new();
        let mut module_item_arena = ModuleItemArena::new();
        let parser = self.build_parser();

        match parser.parse(content) {
            Ok(parsed_items) => {
                // Flatten ParsedModuleItems into ModuleItems + arena, then allocate them
                let item_refs: Vec<ModuleItemRef> = parsed_items
                    .into_iter()
                    .map(|item| {
                        let module_item =
                            item.flatten(&mut expr_arena, &mut stmt_arena, &mut module_item_arena);
                        module_item_arena.alloc(module_item)
                    })
                    .collect();

                Ok(SourceUnit {
                    items: item_refs,
                    expr_arena,
                    stmt_arena,
                    module_item_arena,
                })
            }
            Err(errors) => {
                let parse_errors: Vec<SingleParseError> = errors
                    .into_iter()
                    .map(|e| {
                        let span = e.span();
                        let location = Self::span_to_location(content, span);
                        SingleParseError::new(
                            format!("Parse error: {:?}", e),
                            ParseErrorType::InvalidSyntax,
                        )
                        .with_location(location)
                    })
                    .collect();
                Err(ParseError::multiple(parse_errors))
            }
        }
    }

    /// Convert a character span to a SourceLocation with line/column information
    fn span_to_location(content: &str, span: std::ops::Range<usize>) -> crate::SourceLocation {
        let start = span.start;

        // Count lines and columns
        let mut line = 0;
        let mut last_line_start = 0;

        for (i, ch) in content.char_indices() {
            if i >= start {
                break;
            }
            if ch == '\n' {
                line += 1;
                last_line_start = i + 1;
            }
        }

        let column = start - last_line_start;

        crate::SourceLocation {
            line,
            column,
            span: Some((span.start, span.end)),
        }
    }

    pub fn analyze_semantics(
        &self,
        source_unit: &SourceUnit,
    ) -> Vec<crate::semantic::SemanticError> {
        let mut analyzer = crate::semantic::SemanticAnalyzer::new();
        analyzer.analyze(source_unit)
    }

    fn build_parser(&self) -> impl Parser<char, Vec<ParsedModuleItem>, Error = Simple<char>> + '_ {
        // Comments
        let line_comment = just("//")
            .then(take_until(text::newline::<char, Simple<char>>().or(end())))
            .ignored();
        let block_comment = just("/*").then(take_until(just("*/"))).ignored();

        // Whitespace and comments - match any combination
        let ws = choice((
            filter(|c: &char| c.is_whitespace()).ignored(),
            line_comment,
            block_comment,
        ))
        .repeated()
        .ignored();

        // Keywords that should not be identifiers
        let keywords = [
            "module",
            "endmodule",
            "input",
            "output",
            "inout",
            "wire",
            "assign",
            "initial",
            "always",
            "always_comb",
            "always_ff",
            "final",
            "begin",
            "end",
            "if",
            "else",
            "case",
            "casex",
            "casez",
            "endcase",
            "int",
            "logic",
            "bit",
            "byte",
            "reg",
            "signed",
            "unsigned",
            "integer",
            "time",
            "shortint",
            "longint",
            "class",
            "endclass",
            "extends",
            "function",
            "endfunction",
            "local",
            "protected",
            "new",
            "assert",
            "property",
            "unique",
            "unique0",
            "priority",
            "global",
            "clocking",
            "endclocking",
            "struct",
            "union",
            "packed",
            "soft",
            "tagged",
            "supply0",
            "supply1",
            "tri",
            "triand",
            "trior",
        ];

        // Identifier: [a-zA-Z_][a-zA-Z0-9_$]* (but not keywords)
        let identifier = filter(|c: &char| c.is_alphabetic() || *c == '_')
            .chain::<char, _, _>(
                filter(|c: &char| c.is_alphanumeric() || *c == '_' || *c == '$').repeated(),
            )
            .collect::<String>()
            .try_map(move |s, span| {
                if keywords.contains(&s.as_str()) {
                    Err(Simple::custom(span, format!("'{}' is a keyword", s)))
                } else {
                    Ok(s)
                }
            });

        // Number: decimal, hex, binary, octal (including z/x for high-Z/unknown)
        let number = filter(|c: &char| {
            c.is_ascii_digit()
                || matches!(
                    c,
                    '\'' | 'x'
                        | 'b'
                        | 'o'
                        | 'd'
                        | 'h'
                        | 'z'
                        | '_'
                        | 'X'
                        | 'B'
                        | 'O'
                        | 'D'
                        | 'H'
                        | 'Z'
                )
        })
        .repeated()
        .at_least(1)
        .collect::<String>();

        // String literal: "..."
        let string_literal = just('"')
            .ignore_then(
                filter(|c: &char| *c != '"' && *c != '\\')
                    .or(just('\\').ignore_then(any()))
                    .repeated()
                    .collect::<String>(),
            )
            .then_ignore(just('"'));

        // Expression parser (recursive)
        let expr = recursive(|expr| {
            // System function call: $display(...), $sin(...), etc.
            let system_function = just('$')
                .ignore_then(identifier.clone())
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(ws.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not()
                        .map(|args| args.unwrap_or_default()),
                )
                .map_with_span(
                    |(name, arguments), span| ParsedExpression::SystemFunctionCall {
                        name,
                        arguments,
                        span: (span.start, span.end),
                    },
                );

            // New expression: new or new(args)
            let new_expr = text::keyword("new")
                .then(
                    just('(')
                        .padded_by(ws.clone())
                        .ignore_then(expr.clone().separated_by(just(',').padded_by(ws.clone())))
                        .then_ignore(just(')').padded_by(ws.clone()))
                        .or_not(),
                )
                .map(|(_new, arguments)| ParsedExpression::New {
                    arguments: arguments.unwrap_or_default(),
                    span: (0, 0),
                });

            let atom = choice((
                new_expr,
                system_function,
                string_literal
                    .clone()
                    .map(|s| ParsedExpression::StringLiteral(s, (0, 0))),
                identifier
                    .clone()
                    .map(|name| ParsedExpression::Identifier(name, (0, 0))),
                number
                    .clone()
                    .map(|num| ParsedExpression::Number(num, (0, 0))),
                expr.clone().delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws.clone()),
                ),
            ));

            // Unary operators - order matters for multi-char operators!
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
            ));

            // Unary expression: !a, ~b, +c, -d
            let unary_expr =
                unary_op
                    .then_ignore(ws.clone())
                    .then(atom.clone())
                    .map(|(op, operand)| ParsedExpression::Unary {
                        op,
                        operand: Box::new(operand),
                        span: (0, 0),
                    });

            // Member access: obj.field, obj.field.subfield
            let member_access = choice((unary_expr.clone(), atom.clone()))
                .then(just('.').ignore_then(identifier.clone()).repeated())
                .foldl(|object, member| ParsedExpression::MemberAccess {
                    object: Box::new(object),
                    member,
                    member_span: (0, 0),
                    span: (0, 0),
                });

            // Function call: func(), obj.method()
            let function_call = member_access
                .clone()
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(ws.clone()))
                        .delimited_by(just('('), just(')'))
                        .or_not(),
                )
                .map(|(function, maybe_args)| {
                    if let Some(args) = maybe_args {
                        ParsedExpression::FunctionCall {
                            function: Box::new(function),
                            arguments: args,
                            span: (0, 0),
                        }
                    } else {
                        function
                    }
                });

            let primary = function_call;

            // Binary operators - split into groups to avoid tuple size limits
            let binary_op_multi = choice((
                just("<->").to(BinaryOp::LogicalEquiv),
                just("->").to(BinaryOp::LogicalImpl),
                just("<<<").to(BinaryOp::ArithmeticShiftLeft),
                just(">>>").to(BinaryOp::ArithmeticShiftRight),
                just("<<").to(BinaryOp::LogicalShiftLeft),
                just(">>").to(BinaryOp::LogicalShiftRight),
                just("<=").to(BinaryOp::LessEqual),
                just(">=").to(BinaryOp::GreaterEqual),
                just("===").to(BinaryOp::CaseEqual),
                just("!==").to(BinaryOp::CaseNotEqual),
                just("==?").to(BinaryOp::WildcardEqual),
                just("!=?").to(BinaryOp::WildcardNotEqual),
                just("==").to(BinaryOp::Equal),
                just("!=").to(BinaryOp::NotEqual),
                just("&&").to(BinaryOp::LogicalAnd),
                just("||").to(BinaryOp::LogicalOr),
            ));

            let binary_op_single = choice((
                just("**").to(BinaryOp::Power),
                just("~^").to(BinaryOp::BitwiseXnor),
                just("<").to(BinaryOp::LessThan),
                just(">").to(BinaryOp::GreaterThan),
                just("+").to(BinaryOp::Add),
                just("-").to(BinaryOp::Sub),
                just("*").to(BinaryOp::Mul),
                just("/").to(BinaryOp::Div),
                just("%").to(BinaryOp::Modulo),
                just("&").to(BinaryOp::And),
                just("|").to(BinaryOp::Or),
                just("^").to(BinaryOp::Xor),
            ));

            let binary_op = choice((binary_op_multi, binary_op_single));

            primary
                .clone()
                .then(
                    binary_op
                        .padded_by(ws.clone())
                        .then(primary.clone())
                        .or_not(),
                )
                .map(|(left, maybe_right)| {
                    if let Some((op, right)) = maybe_right {
                        ParsedExpression::Binary {
                            op,
                            left: Box::new(left),
                            right: Box::new(right),
                            span: (0, 0),
                        }
                    } else {
                        left
                    }
                })
        });

        // Delay: #number
        let delay = just('#').ignore_then(number.clone()).map(Delay::Value);

        // Range: [3:0]
        let range = just('[')
            .padded_by(ws.clone())
            .ignore_then(choice((number.clone(), identifier.clone())))
            .then_ignore(ws.clone())
            .then_ignore(just(':'))
            .then_ignore(ws.clone())
            .then(choice((number.clone(), identifier.clone())))
            .then_ignore(ws.clone())
            .then_ignore(just(']'))
            .map(|(msb, lsb)| Range { msb, lsb });

        // Concurrent assertion
        let concurrent_assertion = text::keyword("assert")
            .padded_by(ws.clone())
            .ignore_then(text::keyword("property"))
            .then_ignore(ws.clone())
            .ignore_then(
                filter(|c| *c != ';')
                    .repeated()
                    .then_ignore(just(';').padded_by(ws.clone())),
            )
            .map_with_span(|_, span| ParsedModuleItem::ConcurrentAssertion {
                statement: ParsedStatement::ExpressionStatement {
                    expr: ParsedExpression::Identifier("placeholder".to_string(), (0, 0)),
                },
                span: (span.start, span.end),
            });

        // Type keywords - order matters! Longer keywords first
        let type_keyword = choice((
            text::keyword("shortint").to("shortint".to_string()),
            text::keyword("longint").to("longint".to_string()),
            text::keyword("integer").to("integer".to_string()),
            text::keyword("supply0").to("supply0".to_string()),
            text::keyword("supply1").to("supply1".to_string()),
            text::keyword("triand").to("triand".to_string()),
            text::keyword("trior").to("trior".to_string()),
            text::keyword("logic").to("logic".to_string()),
            text::keyword("uwire").to("uwire".to_string()),
            text::keyword("wire").to("wire".to_string()),
            text::keyword("wand").to("wand".to_string()),
            text::keyword("wor").to("wor".to_string()),
            text::keyword("byte").to("byte".to_string()),
            text::keyword("time").to("time".to_string()),
            text::keyword("tri0").to("tri0".to_string()),
            text::keyword("tri1").to("tri1".to_string()),
            text::keyword("tri").to("tri".to_string()),
            text::keyword("int").to("int".to_string()),
            text::keyword("bit").to("bit".to_string()),
            text::keyword("reg").to("reg".to_string()),
        ));

        // Port direction
        let port_direction = choice((
            text::keyword("input").to(PortDirection::Input),
            text::keyword("output").to(PortDirection::Output),
            text::keyword("inout").to(PortDirection::Inout),
        ));

        // Preprocessor directives
        let define_directive = ws
            .clone()
            .ignore_then(just('`'))
            .ignore_then(text::keyword("define"))
            .ignore_then(ws.clone())
            .ignore_then(
                identifier
                    .clone()
                    .map_with_span(|n, s| (n, (s.start, s.end))),
            )
            .then_ignore(ws.clone())
            .then(
                just('(')
                    .ignore_then(
                        identifier
                            .clone()
                            .separated_by(just(',').padded_by(ws.clone())),
                    )
                    .then_ignore(just(')'))
                    .then_ignore(ws.clone())
                    .or_not(),
            )
            .then(filter(|c: &char| *c != '\n').repeated().collect::<String>())
            .map_with_span(|(((name, name_span), params), value), span| {
                ParsedModuleItem::DefineDirective {
                    name,
                    name_span,
                    parameters: params.unwrap_or_default(),
                    value: value.trim().to_string(),
                    span: (span.start, span.end),
                }
            });

        let include_directive = ws
            .clone()
            .ignore_then(just('`'))
            .ignore_then(text::keyword("include"))
            .then_ignore(ws.clone())
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
                )),
            )
            .map_with_span(|path, span| ParsedModuleItem::IncludeDirective {
                path,
                path_span: (span.start, span.end),
                span: (span.start, span.end),
            });

        // Port declaration
        let port_decl = ws
            .clone()
            .ignore_then(port_direction.clone())
            .then_ignore(ws.clone())
            .then(type_keyword.clone()) // port type (wire, reg, logic, etc.)
            .then_ignore(ws.clone())
            .then(
                identifier
                    .clone()
                    .map_with_span(|n, s| (n, (s.start, s.end))),
            ) // port name
            .then_ignore(ws.clone())
            .then_ignore(just(';'))
            .map_with_span(|((direction, port_type), (name, name_span)), span| {
                ParsedModuleItem::PortDeclaration {
                    direction,
                    port_type,
                    name,
                    name_span,
                    span: (span.start, span.end),
                }
            });

        // Port: input [3:0] a, output b, output reg data, or just "clk" (non-ANSI)
        let port = port_direction
            .clone()
            .then_ignore(ws.clone())
            .then(
                // Optional type keyword (e.g., 'reg', 'wire')
                type_keyword.clone().then_ignore(ws.clone()).or_not(),
            )
            .then(range.clone().or_not())
            .then_ignore(ws.clone())
            .then(identifier.clone())
            .map(|(((direction, _type), range), name)| Port {
                name: name.clone(),
                name_span: (0, 0),
                direction: Some(direction),
                range,
                span: (0, 0),
            })
            .or(
                // Non-ANSI style: just port name without direction
                identifier.clone().map(|name| Port {
                    name: name.clone(),
                    name_span: (0, 0),
                    direction: None,
                    range: None,
                    span: (0, 0),
                }),
            );

        // Port list: (input a, input b) or ()
        let port_list = port
            .separated_by(just(',').padded_by(ws.clone()))
            .allow_trailing()
            .delimited_by(
                just('(').padded_by(ws.clone()),
                just(')').padded_by(ws.clone()),
            );

        // Statement parser (for inside initial/always blocks)
        let statement = recursive(|_statement| {
            // Assignment operators - order matters! Longest first
            let assign_op = choice((
                just(">>>=").to(AssignmentOp::AShrAssign),
                just("<<<=").to(AssignmentOp::AShlAssign),
                just(">>=").to(AssignmentOp::ShrAssign),
                just("<<=").to(AssignmentOp::ShlAssign),
                just("^=").to(AssignmentOp::XorAssign),
                just("+=").to(AssignmentOp::AddAssign),
                just("-=").to(AssignmentOp::SubAssign),
                just("*=").to(AssignmentOp::MulAssign),
                just("/=").to(AssignmentOp::DivAssign),
                just("%=").to(AssignmentOp::ModAssign),
                just("&=").to(AssignmentOp::AndAssign),
                just("|=").to(AssignmentOp::OrAssign),
                just("=").to(AssignmentOp::Assign),
            ));

            // Statement-level assignment: a ^= b;
            let stmt_assignment = ws
                .clone()
                .ignore_then(expr.clone())
                .then_ignore(ws.clone())
                .then(assign_op)
                .then_ignore(ws.clone())
                .then(expr.clone())
                .then_ignore(ws.clone())
                .then_ignore(just(';'))
                .map(|((target, op), expr)| ParsedStatement::Assignment { target, op, expr });

            // System call: $display(...);
            let system_call = ws
                .clone()
                .ignore_then(just('$'))
                .ignore_then(identifier.clone())
                .then(
                    expr.clone()
                        .separated_by(just(',').padded_by(ws.clone()))
                        .delimited_by(
                            just('(').padded_by(ws.clone()),
                            just(')').padded_by(ws.clone()),
                        )
                        .or_not()
                        .map(|args| args.unwrap_or_default()),
                )
                .then_ignore(just(';').padded_by(ws.clone()))
                .map_with_span(|(name, args), span| ParsedStatement::SystemCall {
                    name,
                    args,
                    span: (span.start, span.end),
                });

            // Case statement modifiers
            let case_modifier = choice((
                text::keyword("unique0").to("unique0".to_string()),
                text::keyword("unique").to("unique".to_string()),
                text::keyword("priority").to("priority".to_string()),
            ))
            .padded_by(ws.clone())
            .or_not();

            // Case type
            let case_type = choice((
                text::keyword("casez").to("casez".to_string()),
                text::keyword("casex").to("casex".to_string()),
                text::keyword("case").to("case".to_string()),
            ))
            .padded_by(ws.clone());

            // Case statement (simplified - just skip to endcase)
            let case_stmt = case_modifier
                .then(case_type)
                .then(expr.clone().delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws.clone()),
                ))
                .then_ignore(
                    filter(|c| *c != 'e')
                        .repeated()
                        .then(text::keyword("endcase"))
                        .padded_by(ws.clone()),
                )
                .map(
                    |((modifier, case_type), case_expr)| ParsedStatement::CaseStatement {
                        modifier,
                        case_type,
                        expr: case_expr,
                    },
                );

            // Assert property statement
            let assert_property = text::keyword("assert")
                .padded_by(ws.clone())
                .ignore_then(text::keyword("property").padded_by(ws.clone()))
                .ignore_then(expr.clone().delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws.clone()),
                ))
                .then(
                    text::keyword("else")
                        .padded_by(ws.clone())
                        .ignore_then(
                            just('$')
                                .ignore_then(identifier.clone())
                                .then(
                                    expr.clone()
                                        .separated_by(just(',').padded_by(ws.clone()))
                                        .delimited_by(just('('), just(')'))
                                        .or_not()
                                        .map(|args| args.unwrap_or_default()),
                                )
                                .map_with_span(|(name, args), span| ParsedStatement::SystemCall {
                                    name,
                                    args,
                                    span: (span.start, span.end),
                                }),
                        )
                        .or_not(),
                )
                .then_ignore(just(';').padded_by(ws.clone()))
                .map(
                    |(property_expr, action_block)| ParsedStatement::AssertProperty {
                        property_expr,
                        action_block: action_block.map(Box::new),
                    },
                );

            // Variable declaration statement: logic a = $tan(1);
            let var_decl_stmt = choice((
                text::keyword("logic").to("logic".to_string()),
                text::keyword("bit").to("bit".to_string()),
                text::keyword("int").to("int".to_string()),
                text::keyword("byte").to("byte".to_string()),
                text::keyword("reg").to("reg".to_string()),
                text::keyword("integer").to("integer".to_string()),
                text::keyword("time").to("time".to_string()),
                text::keyword("shortint").to("shortint".to_string()),
                text::keyword("longint").to("longint".to_string()),
                text::keyword("real").to("real".to_string()),
                text::keyword("realtime").to("realtime".to_string()),
            ))
            .padded_by(ws.clone())
            .then(
                identifier
                    .clone()
                    .map_with_span(|name, span| (name, (span.start, span.end))),
            )
            .then(
                just('=')
                    .padded_by(ws.clone())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .then_ignore(just(';').padded_by(ws.clone()))
            .map_with_span(|((data_type, (name, name_span)), initial_value), span| {
                ParsedStatement::VariableDeclaration {
                    data_type,
                    name,
                    name_span,
                    initial_value,
                    span: (span.start, span.end),
                }
            });

            // Expression statement (for function calls)
            let expr_stmt = expr
                .clone()
                .then_ignore(just(';').padded_by(ws.clone()))
                .map(|expr| ParsedStatement::ExpressionStatement { expr });

            choice((
                assert_property,
                case_stmt,
                system_call,
                var_decl_stmt,
                stmt_assignment,
                expr_stmt,
            ))
        });

        // Unpacked dimension: [10] or []
        let unpacked_dim = just('[')
            .padded_by(ws.clone())
            .ignore_then(choice((number.clone(), identifier.clone())).or_not())
            .then_ignore(ws.clone())
            .then_ignore(just(']'))
            .map(|dim| match dim {
                None => UnpackedDimension::Dynamic,
                Some(size) => UnpackedDimension::FixedSize(size),
            });

        // Class qualifier
        let class_qualifier = choice((
            text::keyword("local").to(ClassQualifier::Local),
            text::keyword("protected").to(ClassQualifier::Protected),
        ));

        // Class item parser
        let class_item = recursive(|_class_item| {
            // Class property
            let class_property = ws
                .clone()
                .ignore_then(class_qualifier.clone().or_not())
                .then_ignore(ws.clone())
                .then(choice((type_keyword.clone(), identifier.clone())))
                .then_ignore(ws.clone())
                .then(identifier.clone())
                .then_ignore(ws.clone())
                .then(unpacked_dim.clone().repeated())
                .then_ignore(ws.clone())
                .then(
                    just('=')
                        .padded_by(ws.clone())
                        .ignore_then(expr.clone())
                        .or_not(),
                )
                .then_ignore(ws.clone())
                .then_ignore(just(';'))
                .map(
                    |((((qualifier, data_type), name), unpacked), initial_value)| {
                        ParsedClassItem::Property {
                            qualifier,
                            data_type,
                            name,
                            unpacked_dimensions: unpacked,
                            initial_value,
                        }
                    },
                );

            // Class method
            let class_method = ws
                .clone()
                .ignore_then(class_qualifier.clone().or_not())
                .then_ignore(ws.clone())
                .then_ignore(text::keyword("function"))
                .then_ignore(ws.clone())
                .then(choice((type_keyword.clone(), identifier.clone())).or_not()) // return type (optional)
                .then_ignore(ws.clone())
                .then(identifier.clone()) // method name
                .then_ignore(ws.clone())
                .then(
                    // parameter list
                    just('(')
                        .padded_by(ws.clone())
                        .ignore_then(just(')'))
                        .to(Vec::new())
                        .or(just('(')
                            .padded_by(ws.clone())
                            .ignore_then(
                                identifier
                                    .clone()
                                    .separated_by(just(',').padded_by(ws.clone())),
                            )
                            .then_ignore(just(')').padded_by(ws.clone()))),
                )
                .then_ignore(just(';').padded_by(ws.clone()))
                .then(
                    // function body - statements until endfunction
                    statement.clone().repeated(),
                )
                .then_ignore(ws.clone())
                .then_ignore(text::keyword("endfunction"))
                .map(|((((qualifier, return_type), name), parameters), body)| {
                    ParsedClassItem::Method {
                        qualifier,
                        return_type,
                        name,
                        parameters,
                        body,
                    }
                });

            choice((class_property, class_method))
        });

        // Class declaration
        let class_decl = ws
            .clone()
            .ignore_then(text::keyword("class"))
            .then_ignore(ws.clone())
            .ignore_then(
                identifier
                    .clone()
                    .map_with_span(|n, s| (n, (s.start, s.end))),
            )
            .then_ignore(ws.clone())
            .then(
                text::keyword("extends")
                    .ignore_then(ws.clone())
                    .ignore_then(identifier.clone())
                    .or_not(),
            )
            .then_ignore(ws.clone())
            .then_ignore(just(';'))
            .then_ignore(ws.clone())
            .then(class_item.repeated())
            .then_ignore(ws.clone())
            .then_ignore(text::keyword("endclass"))
            .then_ignore(ws.clone())
            .map_with_span(|(((name, name_span), extends), items), span| {
                ParsedModuleItem::ClassDeclaration {
                    name,
                    name_span,
                    extends,
                    items,
                    span: (span.start, span.end),
                }
            });

        // Module item parser (recursive for module body)
        let module_item = recursive(|_module_item| {
            // Signing keyword
            let signing = choice((
                text::keyword("signed").to("signed"),
                text::keyword("unsigned").to("unsigned"),
            ));

            // Drive strength: (supply0, supply1), (strong0, strong1), etc.
            let strength_keyword = choice((
                text::keyword("supply0").to("supply0"),
                text::keyword("supply1").to("supply1"),
                text::keyword("strong0").to("strong0"),
                text::keyword("strong1").to("strong1"),
                text::keyword("pull0").to("pull0"),
                text::keyword("pull1").to("pull1"),
                text::keyword("weak0").to("weak0"),
                text::keyword("weak1").to("weak1"),
                text::keyword("highz0").to("highz0"),
                text::keyword("highz1").to("highz1"),
            ));

            let drive_strength = just('(')
                .padded_by(ws.clone())
                .ignore_then(strength_keyword.clone())
                .then_ignore(just(',').padded_by(ws.clone()))
                .then(strength_keyword.clone())
                .then_ignore(just(')').padded_by(ws.clone()))
                .map(|(s0, s1)| DriveStrength {
                    strength0: s0.to_string(),
                    strength1: s1.to_string(),
                });

            // Union/struct type
            let union_struct_type = choice((
                text::keyword("union").to("union".to_string()),
                text::keyword("struct").to("struct".to_string()),
            ))
            .then_ignore(ws.clone())
            .then(text::keyword("packed").or_not())
            .then_ignore(ws.clone())
            .then_ignore(just('{'))
            .then_ignore(ws.clone())
            .then(
                // Parse struct/union members: type name;
                type_keyword
                    .clone()
                    .or(identifier.clone())
                    .then_ignore(ws.clone())
                    .then(range.clone().or_not())
                    .then_ignore(ws.clone())
                    .then(identifier.clone())
                    .then_ignore(ws.clone())
                    .then_ignore(just(';'))
                    .then_ignore(ws.clone())
                    .repeated()
                    .at_least(1),
            )
            .then_ignore(ws.clone())
            .then_ignore(just('}'))
            .map(|((union_or_struct, _packed), _members)| {
                // For now, just return "union" or "struct" as the type name
                // A full implementation would store the member information
                union_or_struct
            });

            // Variable declaration: wire w; or int unsigned a = 12; or bit [7:0] arr[10]; or logic a, b, c;
            // or union { ... } un;
            let var_decl = ws
                .clone()
                .ignore_then(choice((
                    union_struct_type.clone(),
                    type_keyword.clone(),
                    identifier.clone(),
                )))
                .then_ignore(ws.clone())
                .then(signing.or_not())
                .then_ignore(ws.clone())
                .then(drive_strength.or_not())
                .then_ignore(ws.clone())
                .then(range.clone().or_not()) // Packed dimension [7:0]
                .then_ignore(ws.clone())
                .then(delay.clone().or_not())
                .then_ignore(ws.clone())
                .then(
                    identifier
                        .clone()
                        .map_with_span(|n, s| (n, (s.start, s.end)))
                        .then_ignore(ws.clone())
                        .then(unpacked_dim.clone().repeated()) // Unpacked dimensions [10][20]
                        .then_ignore(ws.clone())
                        .then(
                            just('=')
                                .padded_by(ws.clone())
                                .ignore_then(expr.clone())
                                .or_not(),
                        )
                        .separated_by(just(',').padded_by(ws.clone()))
                        .at_least(1),
                )
                .then_ignore(ws.clone())
                .then_ignore(just(';'))
                .map_with_span(
                    |(
                        ((((data_type, signing), drive_strength), packed_range), delay),
                        variables,
                    ),
                     span| {
                        // For now, return only the first variable as VariableDeclaration
                        // In a real implementation, we'd need to handle multiple declarations
                        let (((name, name_span), unpacked), initial_value) = &variables[0];
                        ParsedModuleItem::VariableDeclaration {
                            data_type: data_type.to_string(),
                            signing: signing.map(|s| s.to_string()),
                            drive_strength,
                            delay,
                            range: packed_range,
                            name: name.clone(),
                            name_span: *name_span,
                            unpacked_dimensions: unpacked.clone(),
                            initial_value: initial_value.clone(),
                            span: (span.start, span.end),
                        }
                    },
                );

            // Continuous assignment: assign #delay? target = expr;
            let assignment = ws
                .clone()
                .ignore_then(text::keyword("assign"))
                .then_ignore(ws.clone())
                .ignore_then(delay.clone().or_not())
                .then_ignore(ws.clone())
                .then(expr.clone())
                .then_ignore(ws.clone())
                .then_ignore(just('='))
                .then_ignore(ws.clone())
                .then(expr.clone())
                .then_ignore(ws.clone())
                .then_ignore(just(';'))
                .map_with_span(
                    |((delay, target), expr), span| ParsedModuleItem::Assignment {
                        delay,
                        target,
                        expr,
                        span: (span.start, span.end),
                    },
                );

            // Procedural block type
            let block_type = choice((
                text::keyword("always_comb").to(ProceduralBlockType::AlwaysComb),
                text::keyword("always_ff").to(ProceduralBlockType::AlwaysFF),
                text::keyword("always").to(ProceduralBlockType::Always),
                text::keyword("initial").to(ProceduralBlockType::Initial),
                text::keyword("final").to(ProceduralBlockType::Final),
            ));

            // Procedural block: initial/always/always_comb/always_ff/final begin...end
            let procedural_block = ws
                .clone()
                .ignore_then(block_type)
                .then_ignore(ws.clone())
                .then_ignore(
                    // Optional event control like @(posedge clk)
                    just('@')
                        .padded_by(ws.clone())
                        .ignore_then(
                            just('(')
                                .ignore_then(filter(|c| *c != ')').repeated())
                                .then_ignore(just(')')),
                        )
                        .or_not(),
                )
                .then_ignore(ws.clone())
                .then(choice((
                    // Multiple statements with begin/end
                    text::keyword("begin")
                        .ignore_then(ws.clone())
                        .ignore_then(statement.clone().repeated())
                        .then_ignore(ws.clone())
                        .then_ignore(text::keyword("end")),
                    // Single statement without begin/end
                    statement.clone().map(|s| vec![s]),
                )))
                .map_with_span(|(block_type, statements), span| {
                    ParsedModuleItem::ProceduralBlock {
                        block_type,
                        statements,
                        span: (span.start, span.end),
                    }
                });

            // Global clocking (needs to be before var_decl to avoid conflicts)
            let global_clocking_item = text::keyword("global")
                .padded_by(ws.clone())
                .ignore_then(text::keyword("clocking"))
                .then_ignore(ws.clone())
                .ignore_then(
                    identifier
                        .clone()
                        .map_with_span(|n, s| (n, (s.start, s.end)))
                        .or_not(),
                )
                .then_ignore(ws.clone())
                .then(
                    // Event control @(...)
                    just('@')
                        .padded_by(ws.clone())
                        .ignore_then(
                            just('(')
                                .ignore_then(filter(|c| *c != ')').repeated().collect::<String>())
                                .then_ignore(just(')')),
                        )
                        .map(|s| ParsedExpression::Identifier(format!("@({})", s), (0, 0))),
                )
                .then_ignore(ws.clone())
                .then_ignore(just(';'))
                .then_ignore(ws.clone())
                .then_ignore(text::keyword("endclocking"))
                .then_ignore(ws.clone())
                .then(
                    just(':')
                        .padded_by(ws.clone())
                        .ignore_then(identifier.clone())
                        .or_not(),
                )
                .map_with_span(|((identifier, clocking_event), end_label), span| {
                    ParsedModuleItem::GlobalClocking {
                        identifier: identifier.as_ref().map(|(n, _)| n.clone()),
                        identifier_span: identifier.map(|(_, s)| s),
                        clocking_event,
                        end_label,
                        span: (span.start, span.end),
                    }
                });

            choice((
                define_directive.clone(),
                include_directive.clone(),
                global_clocking_item,
                concurrent_assertion.clone(),
                port_decl.clone(),
                class_decl.clone(),
                var_decl,
                assignment,
                procedural_block,
            ))
        });

        // Global clocking (for top-level)
        let global_clocking = text::keyword("global")
            .padded_by(ws.clone())
            .ignore_then(text::keyword("clocking"))
            .then_ignore(ws.clone())
            .ignore_then(
                identifier
                    .clone()
                    .map_with_span(|n, s| (n, (s.start, s.end)))
                    .or_not(),
            )
            .then_ignore(ws.clone())
            .then(
                // Event control @(...)
                just('@')
                    .padded_by(ws.clone())
                    .ignore_then(
                        just('(')
                            .ignore_then(filter(|c| *c != ')').repeated().collect::<String>())
                            .then_ignore(just(')')),
                    )
                    .map(|s| ParsedExpression::Identifier(format!("@({})", s), (0, 0))),
            )
            .then_ignore(ws.clone())
            .then_ignore(just(';'))
            .then_ignore(ws.clone())
            .then_ignore(text::keyword("endclocking"))
            .then_ignore(ws.clone())
            .then(
                just(':')
                    .padded_by(ws.clone())
                    .ignore_then(identifier.clone())
                    .or_not(),
            )
            .map_with_span(|((identifier, clocking_event), end_label), span| {
                ParsedModuleItem::GlobalClocking {
                    identifier: identifier.as_ref().map(|(n, _)| n.clone()),
                    identifier_span: identifier.map(|(_, s)| s),
                    clocking_event,
                    end_label,
                    span: (span.start, span.end),
                }
            });

        // Module declaration: module <name> (ports); items endmodule
        let module_decl = ws
            .clone()
            .ignore_then(text::keyword("module"))
            .then_ignore(ws.clone())
            .ignore_then(
                identifier
                    .clone()
                    .map_with_span(|n, s| (n, (s.start, s.end))),
            )
            .then_ignore(ws.clone())
            .then(port_list.or_not())
            .then_ignore(ws.clone())
            .then_ignore(just(';'))
            .then_ignore(ws.clone())
            .then(module_item.repeated())
            .then_ignore(ws.clone())
            .then_ignore(text::keyword("endmodule"))
            .then_ignore(ws.clone())
            .map_with_span(|(((name, name_span), ports), items), span| {
                ParsedModuleItem::ModuleDeclaration {
                    name,
                    name_span,
                    ports: ports.unwrap_or_default(),
                    items,
                    span: (span.start, span.end),
                }
            });

        // Top-level items (modules, classes, preprocessor directives)
        let top_level = choice((
            define_directive,
            include_directive,
            class_decl,
            module_decl,
            global_clocking,
            concurrent_assertion,
            port_decl,
        ));

        ws.clone()
            .ignore_then(top_level.repeated())
            .then_ignore(ws.clone())
            .then_ignore(end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_module() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = "module top(); endmodule";
        let result = parser.parse_content(content);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let source_unit = result.unwrap();
        assert_eq!(source_unit.items.len(), 1);
        let item = source_unit.module_item_arena.get(source_unit.items[0]);
        match item {
            ModuleItem::ModuleDeclaration { name, .. } => {
                assert_eq!(name, "top");
            }
            _ => panic!("Expected ModuleDeclaration"),
        }
    }

    #[test]
    fn test_module_with_assignment() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = "module top(input a, input b);
            wire w;
            assign w = a & b;
            endmodule";
        let result = parser.parse_content(content);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let source_unit = result.unwrap();
        assert_eq!(source_unit.items.len(), 1);

        let item = source_unit.module_item_arena.get(source_unit.items[0]);
        match item {
            ModuleItem::ModuleDeclaration {
                name, ports, items, ..
            } => {
                assert_eq!(name, "top");
                assert_eq!(ports.len(), 2);
                assert_eq!(items.len(), 2);

                // Check wire declaration - items are now refs into the arena
                let item0 = source_unit.module_item_arena.get(items[0]);
                match item0 {
                    ModuleItem::VariableDeclaration {
                        data_type, name, ..
                    } => {
                        assert_eq!(data_type, "wire");
                        assert_eq!(name, "w");
                    }
                    _ => panic!("Expected VariableDeclaration"),
                }

                // Check assignment
                let item1 = source_unit.module_item_arena.get(items[1]);
                match item1 {
                    ModuleItem::Assignment { target, expr, .. } => {
                        // Target should be 'w'
                        let target_expr = source_unit.expr_arena.get(*target);
                        assert!(matches!(target_expr, Expression::Identifier(n, _) if n == "w"));

                        // Expr should be 'a & b'
                        let expr_expr = source_unit.expr_arena.get(*expr);
                        assert!(matches!(
                            expr_expr,
                            Expression::Binary {
                                op: BinaryOp::And,
                                ..
                            }
                        ));
                    }
                    _ => panic!("Expected Assignment"),
                }
            }
            _ => panic!("Expected ModuleDeclaration"),
        }
    }

    #[test]
    fn test_assignment_with_delay() {
        let parser = SystemVerilogParser::new(vec![], HashMap::new());
        let content = "module top(input a, input b);
            wire w;
            assign #10 w = a & b;
            endmodule";
        let result = parser.parse_content(content);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let source_unit = result.unwrap();

        let item = source_unit.module_item_arena.get(source_unit.items[0]);
        match item {
            ModuleItem::ModuleDeclaration { items, .. } => {
                let item1 = source_unit.module_item_arena.get(items[1]);
                match item1 {
                    ModuleItem::Assignment { delay, .. } => {
                        assert!(delay.is_some(), "Expected delay");
                        match delay {
                            Some(Delay::Value(v)) => assert_eq!(v, "10"),
                            _ => panic!("Expected Delay::Value"),
                        }
                    }
                    _ => panic!("Expected Assignment"),
                }
            }
            _ => panic!("Expected ModuleDeclaration"),
        }
    }
}
