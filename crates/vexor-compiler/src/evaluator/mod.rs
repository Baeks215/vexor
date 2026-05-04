//! Evaluator: Typed AST -> Scene

use crate::evaluator::expr::Evaluable;
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT};
use std::collections::HashMap;

mod expr;
mod program;

pub use program::eval_program;

/// Evaluation error
type EError = String;

/// Result type for evaluation
type EResult<O> = Result<O, EError>;

/// Literal value types
#[derive(Debug, Clone)]
enum Value {
    Number(<NumberT as Evaluable>::Output),
    String(<StringT as Evaluable>::Output),
    Bool(<BoolT as Evaluable>::Output),
    Color(<ColorT as Evaluable>::Output),
    Graphic(<GraphicT as Evaluable>::Output),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub scope: Vec<typed::Assignment>,
    pub return_expr: typed::expr::ExprGeneric,
}

/// Context for evaluation
struct Context {
    functions: HashMap<String, Function>,
    vars: HashMap<String, Value>,
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
    fn new_scope_function(&self, names: &[String], args: Vec<Value>) -> Self {
        let mut this = Self {
            functions: self.functions.clone(),
            vars: self.vars.clone(),
        };
        debug_assert_eq!(names.len(), args.len());
        for (name, arg) in names.iter().zip(args) {
            this.vars.insert(name.clone(), arg);
        }
        this
    }

    /// Clone context with one extra variable bound.
    fn with_var(&self, name: String, value: Value) -> Self {
        let mut vars = self.vars.clone();
        vars.insert(name, value);
        Self {
            functions: self.functions.clone(),
            vars,
        }
    }

    /// Add a function to the context
    fn add_function(&mut self, func: typed::Function) {
        let typed::Function {
            name,
            scope,
            params,
            return_expr,
        } = func;
        self.functions.insert(
            name,
            Function {
                params: params.into_iter().map(|(n, _)| n).collect(),
                scope,
                return_expr,
            },
        );
    }

    /// Get a function
    fn get_function(&self, name: &str) -> EResult<&Function> {
        self.functions
            .get(name)
            .ok_or("Unknown function".to_string())
    }
}
