//! Evaluator: Typed AST -> Scene

use crate::evaluator::expr::Evaluable;
use crate::ir::{Number, ast};
use std::collections::HashMap;

mod expr;
mod program;

pub use program::eval_program;

/// Evaluation error
type EError = String;

/// Result type for evaluation
type EResult<O> = Result<O, EError>;

/// Marker Types used to annotate generics
mod ty {
    #[derive(Debug, Clone, Copy)]
    pub struct Any;
    #[derive(Debug, Clone, Copy)]
    pub struct Number;
    #[derive(Debug, Clone, Copy)]
    pub struct String;
    #[derive(Debug, Clone, Copy)]
    pub struct Bool;
    #[derive(Debug, Clone, Copy)]
    pub struct Color;
    #[derive(Debug, Clone, Copy)]
    pub struct Graphic;
    #[derive(Debug, Clone, Copy)]
    pub struct List;
    #[derive(Debug, Clone, Copy)]
    pub struct Function;
}

/// Literal value types
#[derive(Debug, Clone)]
pub enum Value {
    Number(<ty::Number as Evaluable>::Output),
    String(<ty::String as Evaluable>::Output),
    Bool(<ty::Bool as Evaluable>::Output),
    Color(<ty::Color as Evaluable>::Output),
    Graphic(<ty::Graphic as Evaluable>::Output),
    List(<ty::List as Evaluable>::Output),
    Function(<ty::Function as Evaluable>::Output),
}

/// Context for evaluation
#[derive(Debug, Clone)]
pub struct Context {
    pub functions: HashMap<String, ast::Function>,
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
    fn add_function(&mut self, name: String, func: ast::Function) {
        self.functions.insert(name, func);
    }

    /// Get a function
    fn get_function(&self, name: &str) -> EResult<&ast::Function> {
        self.functions
            .get(name)
            .ok_or(format!("Unknown function: {}", name))
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
