//! Type resolver: AST -> Typed AST

use crate::ir::typed::Type;
use std::collections::HashMap;

mod expr;
mod program;

pub use program::*;

/// Type error
type TError = String;

/// Result type for type checking
type TResult<O> = Result<O, TError>;

// --- Constraints ---

/// Constraint for type checking
#[derive(Debug, Clone, Copy, PartialEq)]
enum Constraint {
    Is(Type),
}

impl Type {
    /// Check if type satisfies constraint
    fn satisfies(self, constraint: Constraint) -> TResult<Type> {
        match constraint {
            Constraint::Is(ty) => (self == ty)
                .then_some(ty)
                .ok_or("Type mismatch".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FunctionType {
    args: Vec<Type>,
    return_type: Type,
}

struct Context {
    functions: HashMap<String, FunctionType>,
    var_types: HashMap<String, Type>,
}
impl Context {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            var_types: HashMap::new(),
        }
    }

    /// Get a variable's type from the context.
    fn check_var(&self, name: &str, constraint: Constraint) -> TResult<Type> {
        self.var_types
            .get(name)
            .ok_or("Unknown variable".to_string())
            // Check against constraint
            .and_then(|ty| ty.satisfies(constraint))
    }

    /// Set a variable's type in the context.
    ///   Returns the previous type, if any.
    fn set_var(&mut self, name: String, ty: Type) -> Option<Type> {
        self.var_types.insert(name, ty)
    }

    /// Create a new scope for a function
    ///   Same functions, but fresh variable table of params.
    fn new_scope_function(&self, params: &Vec<(String, Type)>) -> Self {
        let mut var_types = HashMap::new();
        for (name, ty) in params {
            var_types.insert(name.clone(), *ty);
        }
        Self {
            functions: self.functions.clone(),
            var_types,
        }
    }

    /// Clone context with one extra variable bound.
    fn with_var(&self, name: String, ty: Type) -> Self {
        let mut var_types = self.var_types.clone();
        var_types.insert(name, ty);
        Self {
            functions: self.functions.clone(),
            var_types,
        }
    }

    /// Add a function to the context.
    fn add_function(&mut self, name: String, function: FunctionType) {
        self.functions.insert(name, function);
    }

    /// Check a function's type against a constraint.
    fn check_function(&self, name: &str, return_constraint: Constraint) -> TResult<Vec<Type>> {
        self.functions
            .get(name)
            .ok_or("Unknown function".to_string())
            // Check against constraint
            .and_then(|FunctionType { args, return_type }| {
                return_type.satisfies(return_constraint)?;
                Ok(args.clone())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_check_var() {
        let mut var_types = HashMap::new();
        var_types.insert("x".to_string(), Type::Number);
        var_types.insert("s".to_string(), Type::String);

        let functions = HashMap::new();

        let context = Context {
            var_types,
            functions,
        };

        // Test specific type constraint (success)
        assert_eq!(
            context
                .check_var("x", Constraint::Is(Type::Number))
                .unwrap(),
            Type::Number
        );

        // Test specific type constraint (failure)
        assert!(
            context
                .check_var("x", Constraint::Is(Type::String))
                .is_err()
        );

        // Test unknown variable
        assert!(
            context
                .check_var("y", Constraint::Is(Type::Number))
                .is_err()
        );
    }
}
