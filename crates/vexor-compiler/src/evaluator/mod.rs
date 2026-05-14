//! Evaluator: Typed AST -> Scene

use crate::evaluator::expr::Evaluable;
use crate::ir::Number;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

/// Environment: Stores variable bindings
#[derive(Debug, Clone)]
pub struct Env {
    pub parent: Option<Rc<RefCell<Env>>>,
    pub scope: HashMap<String, Value>,
}
trait EnvExt {
    /// Create an empty environment
    fn empty() -> Self;
    /// Create a child scope
    fn child_scope(&self) -> Self;
    /// Get a variable
    fn get_var(&self, name: &str) -> EResult<Value>;
    /// Set a variable, errors if name already exists
    fn set_var(&self, name: String, value: Value) -> EResult<()>;
    /// Create a new scope with the given variables
    ///   Adds the arguments to the variables scope
    fn new_scope_function(&self, args: Vec<(String, Value)>) -> Self;
}

type EnvRef = Rc<RefCell<Env>>;
impl EnvExt for EnvRef {
    fn empty() -> Self {
        Rc::new(RefCell::new(Env {
            parent: None,
            scope: HashMap::new(),
        }))
    }
    fn child_scope(&self) -> Self {
        Rc::new(RefCell::new(Env {
            parent: Some(Rc::clone(self)),
            scope: HashMap::new(),
        }))
    }
    fn get_var(&self, name: &str) -> EResult<Value> {
        let env = self.borrow();

        let current = env.scope.get(name);
        if let Some(value) = current {
            return Ok(value.clone());
        }
        // Doesn't exist, fetch from parent instead
        let Some(parent) = &env.parent else {
            // Parent doesn't exist
            return Err(format!("`{name}` not in scope"));
        };
        parent.get_var(name)
    }
    fn set_var(&self, name: String, value: Value) -> EResult<()> {
        let mut env = self.borrow_mut();
        let old = env.scope.insert(name.clone(), value);
        match old {
            Some(_) => Err(format!("`{name}` already exists in scope")),
            None => Ok(()),
        }
    }
    fn new_scope_function(&self, args: Vec<(String, Value)>) -> Self {
        let child = self.child_scope();
        {
            // Borrow in scope, RAII ensures drop
            let mut env = child.borrow_mut();
            for (name, arg) in args {
                env.scope.insert(name.clone(), arg);
            }
        }
        child
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
