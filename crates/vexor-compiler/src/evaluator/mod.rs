//! Evaluator: Typed AST -> Scene

use crate::evaluator::expr::Evaluable;
use crate::ir::scene::marker;
use crate::ir::{Number, ast};
use std::collections::HashMap;

mod expr;
mod list;
mod program;

pub use program::eval_program;

/// Evaluation error
type EError = String;

/// Result type for evaluation
type EResult<O> = Result<O, EError>;

/// Literal value types
#[derive(Debug, Clone)]
pub enum Value {
    Number(<marker::Number as Evaluable>::Output),
    String(<marker::String as Evaluable>::Output),
    Bool(<marker::Bool as Evaluable>::Output),
    Color(<marker::Color as Evaluable>::Output),
    Graphic(<marker::Graphic as Evaluable>::Output),
    List(<marker::List as Evaluable>::Output),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub scope: Vec<ast::Assignment>,
    pub return_expr: ast::Expr,
}

/// Context for evaluation
#[derive(Debug, Clone)]
pub struct Context {
    pub functions: HashMap<String, Function>,
    pub vars: HashMap<String, Value>,
}
impl Context {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            vars: HashMap::new(),
        }
    }

    /// Get a variable
    fn get_var(&self, name: &str) -> EResult<Value> {
        self.vars
            .get(name)
            .cloned()
            .ok_or("Unknown variable".to_string())
    }

    /// Set a variable
    fn set_var(&mut self, name: String, value: Value) -> Option<Value> {
        self.vars.insert(name, value)
    }

    /// Create a new scope with the given variables
    ///   Adds the arguments to the variables scope
    fn new_scope_function(&self, args: Vec<(String, Value)>) -> Self {
        let mut this = self.clone();
        for (name, arg) in args {
            this.vars.insert(name.clone(), arg);
        }
        this
    }

    /// Add a function to the context
    fn add_function(&mut self, func: ast::Function) {
        let ast::Function {
            name,
            params,
            scope,
            return_expr,
        } = func;
        let func = Function {
            params,
            scope,
            return_expr,
        };
        self.functions.insert(name, func);
    }

    /// Get a function
    fn get_function(&self, name: &str) -> EResult<&Function> {
        self.functions
            .get(name)
            .ok_or("Unknown function".to_string())
    }
}

/// --- Helpers ---

const EPS: f64 = 1e-9;
fn to_int(n: Number) -> EResult<i64> {
    let rounded = n.round();
    if (n - rounded).abs() > EPS {
        return Err(format!("Expected integer, got {}", n));
    }
    Ok(n as i64)
}
