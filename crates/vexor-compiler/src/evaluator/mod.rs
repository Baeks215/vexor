//! Evaluator: Typed AST -> Scene

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::evaluator::expr::{Value, ty};
use crate::ir::{Ident, Number, ast};

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
    Unevaluated(Rc<ast::SpanExpr>),
    Evaluating,
    Evaluated(Value),
}

/// Bindings of a single scope.
#[derive(Debug, Clone)]
enum Scope {
    /// Top-level scope can hold many bindings, so it is hashed for O(1) lookup.
    Global(HashMap<Ident, Thunk>),
    /// Local scopes are often tiny (function params, `where` bindings), flat vector more optimal
    Local(Vec<(Ident, Thunk)>),
}

impl Scope {
    fn global() -> Self {
        Scope::Global(HashMap::new())
    }
    fn local() -> Self {
        Scope::Local(Vec::new())
    }
    /// Collects into a `Local` scope
    fn collect_local<I: IntoIterator<Item = (Ident, Thunk)>>(iter: I) -> Self {
        Scope::Local(iter.into_iter().collect())
    }

    /// Returns the thunk bound to `name` in this scope, if any.
    fn get(&self, name: &str) -> Option<&Thunk> {
        match self {
            Scope::Global(m) => m.get(name),
            Scope::Local(v) => v.iter().find(|(k, _)| k.as_ref() == name).map(|(_, t)| t),
        }
    }
    /// Returns a mutable reference to the thunk bound to `name`, if any.
    fn get_mut(&mut self, name: &str) -> Option<&mut Thunk> {
        match self {
            Scope::Global(m) => m.get_mut(name),
            Scope::Local(v) => v
                .iter_mut()
                .find(|(k, _)| k.as_ref() == name)
                .map(|(_, t)| t),
        }
    }
    /// Returns `true` if `name` is bound in this scope.
    fn contains(&self, name: &str) -> bool {
        match self {
            Scope::Global(m) => m.contains_key(name),
            Scope::Local(v) => v.iter().any(|(k, _)| k.as_ref() == name),
        }
    }
    /// Adds a binding.
    /// PRECONDITION: Caller must ensure `name` is not already present.
    fn push(&mut self, name: Ident, thunk: Thunk) {
        match self {
            Scope::Global(m) => {
                m.insert(name, thunk);
            }
            Scope::Local(v) => v.push((name, thunk)),
        }
    }
}

/// Environment: Stores variable bindings
#[derive(Debug, Clone)]
pub struct Env {
    /// Parent environment for nested scopes
    parent: Option<Rc<RefCell<Env>>>,
    /// Values in the current scope
    scope: Scope,
}
trait EnvExt {
    /// Create an empty top-level environment
    fn top_level() -> Self;
    /// Create a child scope
    fn child_scope(&self) -> Self;
    /// Get a value, forces evaluation if stored as lazy expression
    fn get_var(&self, name: &str) -> EResult<Value>;
    /// Set a value as an unevaluated expression, errors if it already exists
    fn set_var_lazy(&self, name: Ident, ast_expr: Rc<ast::SpanExpr>) -> EResult<()>;
    /// Set a value as an evaluated value, errors if it already exists
    fn set_var(&self, name: Ident, value: Value) -> EResult<()>;
    /// Create a new scope with the given values
    ///   Adds the arguments to the value scope
    fn new_scope_function(&self, args: Vec<(Ident, Value)>) -> Self;
}

type EnvRef = Rc<RefCell<Env>>;
impl EnvExt for EnvRef {
    fn top_level() -> Self {
        Rc::new(RefCell::new(Env {
            parent: None,
            scope: Scope::global(),
        }))
    }
    fn child_scope(&self) -> Self {
        Rc::new(RefCell::new(Env {
            parent: Some(Rc::clone(self)),
            scope: Scope::local(),
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
            let thunk = env.scope.get_mut(name).unwrap(); // Some from match above
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
            let thunk = env.scope.get_mut(name).unwrap(); // Some from match above
            *thunk = Thunk::Evaluated(val.clone());
        }
        Ok(val)
    }
    fn set_var_lazy(&self, name: Ident, ast_expr: Rc<ast::SpanExpr>) -> EResult<()> {
        let mut env = self.borrow_mut();
        if env.scope.contains(&name) {
            return Err(format!("`{name}` already exists in scope").into());
        }
        env.scope.push(name, Thunk::Unevaluated(ast_expr));
        Ok(())
    }
    fn set_var(&self, name: Ident, value: Value) -> EResult<()> {
        let mut env = self.borrow_mut();
        if env.scope.contains(&name) {
            return Err(format!("`{name}` already exists in scope").into());
        }
        env.scope.push(name, Thunk::Evaluated(value));
        Ok(())
    }
    fn new_scope_function(&self, args: Vec<(Ident, Value)>) -> Self {
        let child = self.child_scope();
        {
            // Borrow in scope, RAII ensures drop. Params are pre-validated unique, so the
            // scope is built directly from the args in one allocation.
            let mut env = child.borrow_mut();
            env.scope = Scope::collect_local(
                args.into_iter()
                    .map(|(name, arg)| (name, Thunk::Evaluated(arg))),
            )
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
