//! Evaluator: Typed AST -> Scene

use crate::evaluator::expr::Evaluable;
use crate::ir::Number;
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
    pub parent: Option<Box<Context>>,
    pub scope: HashMap<String, Value>,
}
impl Context {
    fn new() -> Self {
        Self {
            parent: None,
            scope: HashMap::new(),
        }
    }
    fn child_scope(&self) -> Self {
        Self {
            parent: Some(Box::new(self.clone())),
            scope: HashMap::new(),
        }
    }

    /// Get a variable
    fn get_var(&self, name: &str) -> EResult<Value> {
        let current = self.scope.get(name);
        if let Some(value) = current {
            return Ok(value.clone());
        }
        // Doesn't exist, fetch from parent instead
        let Some(parent) = &self.parent else {
            // Parent doesn't exist
            return Err(format!("`{name}` not in scope"));
        };
        parent.get_var(name)
    }

    /// Set a variable, errors if name already exists
    fn set_var(&mut self, name: String, value: Value) -> EResult<()> {
        let old = self.scope.insert(name.clone(), value);
        match old {
            Some(_) => Err(format!("`{name}` already exists in scope")),
            None => Ok(()),
        }
    }

    /// Create a new scope with the given variables
    ///   Adds the arguments to the variables scope
    fn new_scope_function(&self, args: Vec<(String, Value)>) -> Self {
        let mut this = self.child_scope();
        for (name, arg) in args {
            this.scope.insert(name.clone(), arg);
        }
        this
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
