//! Type resolver: AST -> Typed AST

use crate::ir::typed::Type;
use std::collections::HashMap;

mod expr;

/// Type error
type TError = String;

/// Result type for type checking
type TResult<O> = Result<O, TError>;

struct Context {
    var_types: HashMap<String, Type>,
}

// --- Constraints ---

/// Constraint for type checking
#[derive(Debug, Clone, Copy, PartialEq)]
enum Constraint {
    Is(Type),
    Any,
}

impl Type {
    /// Check if type satisfies constraint
    fn satisfies(self, constraint: Constraint) -> TResult<Type> {
        match constraint {
            Constraint::Any => Ok(self),
            Constraint::Is(ty) => (self == ty)
                .then_some(ty)
                .ok_or("Type mismatch".to_string()),
        }
    }
}

// --- Common check functions ---

fn check_identifier(context: &Context, name: &str, constraint: Constraint) -> TResult<Type> {
    context
        .var_types
        .get(name)
        // Ensure the variable exists
        .ok_or("Unknown variable".to_string())
        // Check against constraint
        .and_then(|ty| ty.satisfies(constraint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_identifier() {
        let mut var_types = HashMap::new();
        var_types.insert("x".to_string(), Type::Number);
        var_types.insert("s".to_string(), Type::String);

        let context = Context { var_types };

        // Test Any constraint
        assert_eq!(
            check_identifier(&context, "x", Constraint::Any).unwrap(),
            Type::Number
        );
        assert_eq!(
            check_identifier(&context, "s", Constraint::Any).unwrap(),
            Type::String
        );

        // Test specific type constraint (success)
        assert_eq!(
            check_identifier(&context, "x", Constraint::Is(Type::Number)).unwrap(),
            Type::Number
        );

        // Test specific type constraint (failure)
        assert!(check_identifier(&context, "x", Constraint::Is(Type::String)).is_err());

        // Test unknown variable
        assert!(check_identifier(&context, "y", Constraint::Any).is_err());
    }
}
