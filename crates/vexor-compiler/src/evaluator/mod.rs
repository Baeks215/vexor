//! Evaluator: Typed AST -> Scene

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::evaluator::expr::{Value, ty};
use crate::ir::{Number, ast};

mod expr;
mod program;

pub use program::eval_program;

/// Evaluation error with a source span.
pub type EError = ast::Spanned<String>;

/// Result type for evaluation
pub type EResult<O> = Result<O, EError>;

/// Inject a span on errors that carry the placeholder `0..0` span.
pub trait WithSpan {
    fn with_span_if_missing(self, span: Option<ast::Span>) -> Self;
}
impl<T> WithSpan for EResult<T> {
    fn with_span_if_missing(self, span: Option<ast::Span>) -> Self {
        self.map_err(|mut e| {
            if e.span.is_none() {
                e.span = span;
            }
            e
        })
    }
}

#[derive(Debug, Clone)]
enum Thunk {
    Unevaluated(ast::SpanExpr),
    Evaluating,
    Evaluated(Value),
}

/// Environment: Stores variable bindings
#[derive(Debug, Clone)]
pub struct Env {
    /// Parent environment for nested scopes
    parent: Option<Rc<RefCell<Env>>>,
    /// Values in the current scope
    scope: HashMap<String, Thunk>,
}
trait EnvExt {
    /// Create an empty environment
    fn empty() -> Self;
    /// Create a child scope
    fn child_scope(&self) -> Self;
    /// Get a value, forces evaluation if stored as lazy expression
    fn get_var(&self, name: &str) -> EResult<Value>;
    /// Set a value as an unevaluated expression, errors if it already exists
    fn set_var_lazy(&self, name: String, expr: ast::SpanExpr) -> EResult<()>;
    /// Set a value as an evaluated value, errors if it already exists
    fn set_var(&self, name: String, value: Value) -> EResult<()>;
    /// Create a new scope with the given values
    ///   Adds the arguments to the value scope
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
        {
            let env = self.borrow();
            match env.scope.get(name) {
                None => {
                    // Doesn't exist, fetch from parent instead
                    let Some(parent) = &env.parent else {
                        // Parent doesn't exist
                        return Err(format!("`{name}` not in scope").into());
                    };
                    return parent.get_var(name);
                }
                Some(Thunk::Evaluating) => {
                    return Err(
                        format!("circular dependency detected while evaluating `{name}`").into(),
                    );
                }
                Some(Thunk::Evaluated(val)) => {
                    return Ok(val.clone());
                }
                Some(Thunk::Unevaluated(_)) => {
                    // Need to evaluate, but we can't while immutably borrowed
                }
            }
            // Env ref is dropped
        }
        // Need to evaluate the deferred expression
        let ast_expr = {
            let mut env = self.borrow_mut();
            let thunk = env.scope.get_mut(name).unwrap(); // Must be Some from match above
            // Replace with evaluating to prevent circular dependencies
            let Thunk::Unevaluated(ast_expr) = std::mem::replace(thunk, Thunk::Evaluating) else {
                // Must be unevaluated from match above
                unreachable!()
            };
            ast_expr
        };

        // Evaluate and set in scope.
        let val = expr::eval::<ty::Any>(self, &ast_expr)?;
        {
            let mut env = self.borrow_mut();
            let thunk = env.scope.get_mut(name).unwrap(); // Must be Some from match above
            *thunk = Thunk::Evaluated(val.clone());
        }
        Ok(val)
    }
    fn set_var_lazy(&self, name: String, e: ast::SpanExpr) -> EResult<()> {
        let mut env = self.borrow_mut();
        let old = env.scope.insert(name.clone(), Thunk::Unevaluated(e));
        match old {
            Some(_) => Err(format!("`{name}` already exists in scope").into()),
            None => Ok(()),
        }
    }
    fn set_var(&self, name: String, value: Value) -> EResult<()> {
        let mut env = self.borrow_mut();
        let old = env.scope.insert(name.clone(), Thunk::Evaluated(value));
        match old {
            Some(_) => Err(format!("`{name}` already exists in scope").into()),
            None => Ok(()),
        }
    }
    fn new_scope_function(&self, args: Vec<(String, Value)>) -> Self {
        let child = self.child_scope();
        {
            // Borrow in scope, RAII ensures drop
            let mut env = child.borrow_mut();
            for (name, arg) in args {
                env.scope.insert(name.clone(), Thunk::Evaluated(arg));
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
        return Err(format!("expected integer, got {}", n).into());
    }
    Ok(n as i64)
}
fn to_usize(n: Number) -> EResult<usize> {
    let i = to_int(n)?;
    if i < 0 {
        return Err(format!("expected non-negative integer, got {}", n).into());
    }
    Ok(i as usize)
}
