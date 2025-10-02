//! Semantic analysis for SystemVerilog AST
//!
//! This module provides semantic validation that goes beyond syntax checking.
//! It validates things like:
//! - System function names
//! - Variable declarations and usage
//! - Type checking
//! - Scope resolution

use crate::{Expression, ModuleItem, SourceUnit, Statement};

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

        // Walk the AST and validate
        for item in &source_unit.items {
            self.analyze_module_item(item);
        }

        self.errors.clone()
    }

    /// Analyze a module item
    fn analyze_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDeclaration { items, .. } => {
                // Recursively analyze nested items
                for sub_item in items {
                    self.analyze_module_item(sub_item);
                }
            }
            ModuleItem::ProceduralBlock { statements, .. } => {
                for statement in statements {
                    self.analyze_statement(statement);
                }
            }
            ModuleItem::VariableDeclaration {
                initial_value: Some(expr),
                ..
            } => {
                self.analyze_expression(expr);
            }
            ModuleItem::Assignment { expr, .. } => {
                self.analyze_expression(expr);
            }
            ModuleItem::ConcurrentAssertion { statement, .. } => {
                self.analyze_statement(statement);
            }
            ModuleItem::ClassDeclaration { items, .. } => {
                for class_item in items {
                    self.analyze_class_item(class_item);
                }
            }
            _ => {}
        }
    }

    /// Analyze a class item
    fn analyze_class_item(&mut self, item: &crate::ClassItem) {
        match item {
            crate::ClassItem::Property {
                initial_value: Some(expr),
                ..
            } => {
                self.analyze_expression(expr);
            }
            crate::ClassItem::Method { body, .. } => {
                for statement in body {
                    self.analyze_statement(statement);
                }
            }
            _ => {}
        }
    }

    /// Analyze a statement
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assignment { expr, .. } => {
                self.analyze_expression(expr);
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
                    self.analyze_expression(arg);
                }
            }
            Statement::CaseStatement { expr, .. } => {
                self.analyze_expression(expr);
            }
            Statement::ExpressionStatement { expr, .. } => {
                self.analyze_expression(expr);
            }
            Statement::AssertProperty {
                property_expr,
                action_block,
                ..
            } => {
                self.analyze_expression(property_expr);
                if let Some(action) = action_block {
                    self.analyze_statement(action);
                }
            }
        }
    }

    /// Analyze an expression
    fn analyze_expression(&mut self, expr: &Expression) {
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
                    self.analyze_expression(arg);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left);
                self.analyze_expression(right);
            }
            Expression::Unary { operand, .. } => {
                self.analyze_expression(operand);
            }
            Expression::MacroUsage { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression(arg);
                }
            }
            Expression::New { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression(arg);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.analyze_expression(object);
            }
            Expression::FunctionCall {
                function,
                arguments,
                ..
            } => {
                self.analyze_expression(function);
                for arg in arguments {
                    self.analyze_expression(arg);
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
