//! Semantic analysis for SystemVerilog AST
//!
//! This module provides semantic validation that goes beyond syntax checking.
//! It validates things like:
//! - System function names
//! - Variable declarations and usage
//! - Type checking
//! - Scope resolution

use crate::{
    ExprArena, ExprRef, Expression, ModuleItem, ModuleItemArena, SourceUnit, Statement, StmtArena,
};

/// Represents a semantic error found during analysis
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub error_type: SemanticErrorType,
    pub message: String,
    pub span: (usize, usize),
}

/// Types of semantic errors
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorType {
    /// Unknown system function (e.g., $fel instead of $fell)
    UnknownSystemFunction,
    /// Undeclared identifier
    UndeclaredIdentifier,
    /// Type mismatch
    TypeMismatch,
    /// Invalid operation
    InvalidOperation,
}

/// Semantic analyzer that validates an AST
pub struct SemanticAnalyzer {
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Analyze a source unit and return any semantic errors found
    pub fn analyze(&mut self, source_unit: &SourceUnit) -> Vec<SemanticError> {
        self.errors.clear();

        // Walk the AST and validate - items is now Vec<ModuleItemRef>
        for item_ref in &source_unit.items {
            let item = source_unit.module_item_arena.get(*item_ref);
            self.analyze_module_item(
                item,
                &source_unit.expr_arena,
                &source_unit.stmt_arena,
                &source_unit.module_item_arena,
            );
        }

        self.errors.clone()
    }

    /// Analyze a module item
    fn analyze_module_item(
        &mut self,
        item: &ModuleItem,
        expr_arena: &ExprArena,
        stmt_arena: &StmtArena,
        module_item_arena: &ModuleItemArena,
    ) {
        match item {
            ModuleItem::ModuleDeclaration { items, .. } => {
                // Recursively analyze nested items - items are now refs into the arena
                for item_ref in items {
                    let sub_item = module_item_arena.get(*item_ref);
                    self.analyze_module_item(sub_item, expr_arena, stmt_arena, module_item_arena);
                }
            }
            ModuleItem::ProceduralBlock { statements, .. } => {
                // statements is now Vec<StmtRef>
                for stmt_ref in statements {
                    let statement = stmt_arena.get(*stmt_ref);
                    self.analyze_statement(statement, expr_arena, stmt_arena);
                }
            }
            ModuleItem::VariableDeclaration {
                initial_value: Some(expr),
                ..
            } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            ModuleItem::Assignment { expr, .. } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            ModuleItem::ConcurrentAssertion { statement, .. } => {
                // statement is now StmtRef
                let stmt = stmt_arena.get(*statement);
                self.analyze_statement(stmt, expr_arena, stmt_arena);
            }
            ModuleItem::ClassDeclaration { items, .. } => {
                for class_item in items {
                    self.analyze_class_item(class_item, expr_arena, stmt_arena);
                }
            }
            _ => {}
        }
    }

    /// Analyze a class item
    fn analyze_class_item(
        &mut self,
        item: &crate::ClassItem,
        expr_arena: &ExprArena,
        stmt_arena: &StmtArena,
    ) {
        match item {
            crate::ClassItem::Property {
                initial_value: Some(expr),
                ..
            } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            crate::ClassItem::Method { body, .. } => {
                // body is now Vec<StmtRef>
                for stmt_ref in body {
                    let statement = stmt_arena.get(*stmt_ref);
                    self.analyze_statement(statement, expr_arena, stmt_arena);
                }
            }
            _ => {}
        }
    }

    /// Analyze a statement
    fn analyze_statement(
        &mut self,
        statement: &Statement,
        expr_arena: &ExprArena,
        stmt_arena: &StmtArena,
    ) {
        match statement {
            Statement::Assignment { expr, .. } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            Statement::SystemCall { name, args, span } => {
                // Validate system task name
                if !self.is_valid_system_task(name) {
                    self.errors.push(SemanticError {
                        error_type: SemanticErrorType::UnknownSystemFunction,
                        message: format!("Unknown system task: ${}", name),
                        span: *span,
                    });
                }
                // Analyze arguments
                for arg in args {
                    self.analyze_expression_ref(*arg, expr_arena);
                }
            }
            Statement::CaseStatement { expr, .. } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            Statement::ExpressionStatement { expr, .. } => {
                self.analyze_expression_ref(*expr, expr_arena);
            }
            Statement::AssertProperty {
                property_expr,
                action_block,
                ..
            } => {
                self.analyze_expression_ref(*property_expr, expr_arena);
                if let Some(action_ref) = action_block {
                    let action_stmt = stmt_arena.get(*action_ref);
                    self.analyze_statement(action_stmt, expr_arena, stmt_arena);
                }
            }
        }
    }

    /// Analyze an expression reference
    fn analyze_expression_ref(&mut self, expr_ref: ExprRef, arena: &ExprArena) {
        let expr = arena.get(expr_ref);
        match expr {
            Expression::SystemFunctionCall {
                name,
                arguments,
                span,
                ..
            } => {
                // Validate system function name
                if !self.is_valid_system_function(name) {
                    self.errors.push(SemanticError {
                        error_type: SemanticErrorType::UnknownSystemFunction,
                        message: format!("Unknown system function: ${}", name),
                        span: *span,
                    });
                }
                // Analyze arguments
                for arg in arguments {
                    self.analyze_expression_ref(*arg, arena);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression_ref(*left, arena);
                self.analyze_expression_ref(*right, arena);
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expression_ref(*operand, arena);
            }
            Expression::MacroUsage { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression_ref(*arg, arena);
                }
            }
            Expression::New { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression_ref(*arg, arena);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.analyze_expression_ref(*object, arena);
            }
            Expression::FunctionCall {
                function,
                arguments,
                ..
            } => {
                self.analyze_expression_ref(*function, arena);
                for arg in arguments {
                    self.analyze_expression_ref(*arg, arena);
                }
            }
            _ => {}
        }
    }

    /// Check if a system function name is valid
    fn is_valid_system_function(&self, name: &str) -> bool {
        matches!(
            name,
            // Sampled value functions (16.9.3)
            "rose" | "fell" | "stable" | "past" | "changed" | "sampled" |
            // Global clocking sampled value functions
            "future_gclk" | "rising_gclk" | "falling_gclk" | "steady_gclk" |
            "changing_gclk" | "past_gclk" | "rose_gclk" | "fell_gclk" |
            "stable_gclk" | "changed_gclk" |
            // Math functions (20.8)
            "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" |
            "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh" |
            "ln" | "log10" | "exp" | "sqrt" | "pow" | "floor" | "ceil" |
            "hypot" |
            // Conversion functions (20.5)
            "itor" | "rtoi" | "bitstoreal" | "realtobits" |
            "shortrealtobits" | "bitstoshortreal" |
            // Array query functions (20.7)
            "left" | "right" | "low" | "high" | "increment" | "size" |
            "dimensions" |
            // Bit vector functions (20.9)
            "clog2" | "bits" | "typename" |
            "isunknown" | "onehot" | "onehot0" | "countbits" | "countones" |
            // Random functions (18.13)
            "urandom" | "urandom_range" | "random" |
            // Misc
            "time" | "stime" | "realtime"
        )
    }

    /// Check if a system task name is valid
    fn is_valid_system_task(&self, name: &str) -> bool {
        matches!(
            name,
            // Display/output tasks (21.2)
            "display" | "write" | "monitor" | "strobe" |
            "displayb" | "displayh" | "displayo" |
            "writeb" | "writeh" | "writeo" |
            "monitorb" | "monitorh" | "monitoro" |
            "strobeb" | "strobeh" | "strobeo" |
            // File I/O tasks (21.3)
            "fdisplay" | "fwrite" | "fmonitor" | "fstrobe" |
            "fdisplayb" | "fdisplayh" | "fdisplayo" |
            "fwriteb" | "fwriteh" | "fwriteo" |
            "fmonitorb" | "fmonitorh" | "fmonitoro" |
            "fstrobeb" | "fstrobeh" | "fstrobeo" |
            "swrite" | "sformat" | "sformatf" |
            "fopen" | "fclose" | "fflush" | "fgetc" | "fgets" |
            "fread" | "fscanf" | "sscanf" | "fseek" | "ftell" | "rewind" |
            "ungetc" | "feof" | "ferror" |
            // Severity tasks (20.10)
            "info" | "warning" | "error" | "fatal" |
            // Simulation control (20.2)
            "finish" | "stop" | "exit" |
            // Timing (20.3, 20.4)
            "timeformat" | "printtimescale" |
            // Memory load (21.4)
            "readmemb" | "readmemh" | "writememb" | "writememh" |
            // Value change dump (21.7)
            "dumpfile" | "dumpvars" | "dumpon" | "dumpoff" | "dumpall" |
            "dumpflush" | "dumplimit" | "dumpports" | "dumpportsoff" |
            "dumpportson" | "dumpportsall" | "dumpportsflush" | "dumpportslimit" |
            // Assertion control (20.11)
            "assertoff" | "asserton" | "assertkill" | "assertcontrol" |
            "assertpasson" | "assertpassoff" | "assertfailon" | "assertfailoff" |
            "assertnonvacuouson" | "assertvacuousoff"
        )
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
