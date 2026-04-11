//! Evaluator: Typed AST -> Scene

use crate::ir::{Number, scene};
use std::collections::HashMap;

mod expr;
mod program;

pub use program::*;

/// Evaluation error
type EError = String;

/// Result type for evaluation
type EResult<O> = Result<O, EError>;

/// Literal value types
#[derive(Debug, Clone)]
enum Value {
    Number(Number),
    String(String),
    Color(scene::Color),
    Graphic(scene::Graphic),
}

/// Context for evaluation
struct Context {
    vars: HashMap<String, Value>,
}
impl Context {
    fn new() -> Self {
        Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_set_get_var() {
        let mut context = Context::new();
        let val = Value::Number(42.0);
        context.set_var("x".to_string(), val.clone());

        let retrieved = context.get_var("x").unwrap();
        if let Value::Number(n) = retrieved {
            assert_eq!(n, 42.0);
        } else {
            panic!("Expected Value::Number, got {:?}", retrieved);
        }

        // Test overwrite
        context.set_var("x".to_string(), Value::String("hello".to_string()));
        let retrieved = context.get_var("x").unwrap();
        if let Value::String(s) = retrieved {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Value::String, got {:?}", retrieved);
        }
    }

    #[test]
    fn test_context_get_unknown_var() {
        let context = Context::new();
        assert!(context.get_var("y").is_err());
    }

    #[test]
    fn test_context_complex_values() {
        let mut context = Context::new();

        let color = Value::Color(scene::Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
        context.set_var("red".to_string(), color.clone());

        if let Value::Color(c) = context.get_var("red").unwrap() {
            assert_eq!(
                c,
                scene::Color::Rgba {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0
                }
            );
        } else {
            panic!("Expected Color");
        }

        let graphic = Value::Graphic(scene::Graphic::Circle { radius: 10.0 });
        context.set_var("ball".to_string(), graphic.clone());

        if let Value::Graphic(g) = context.get_var("ball").unwrap() {
            assert_eq!(g, scene::Graphic::Circle { radius: 10.0 });
        } else {
            panic!("Expected Graphic");
        }
    }
}
