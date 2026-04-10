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

struct Context {
    var_types: HashMap<String, Type>,
}
impl Context {
    fn new() -> Self {
        Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_check_var() {
        let mut var_types = HashMap::new();
        var_types.insert("x".to_string(), Type::Number);
        var_types.insert("s".to_string(), Type::String);

        let context = Context { var_types };

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
