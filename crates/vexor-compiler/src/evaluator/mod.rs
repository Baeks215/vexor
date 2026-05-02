//! Evaluator: Typed AST -> Scene

use crate::ir::{Number, scene, typed};
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
    Number(Number),
    String(String),
    Bool(bool),
    Color(scene::Color),
    Graphic(scene::Graphic),
}

impl Value {
    fn as_number(self) -> EResult<Number> {
        match self {
            Value::Number(n) => Ok(n),
            _ => Err("Expected a number".to_string()),
        }
    }
    fn as_string(self) -> EResult<String> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err("Expected a string".to_string()),
        }
    }
    fn as_bool(self) -> EResult<bool> {
        match self {
            Value::Bool(b) => Ok(b),
            _ => Err("Expected a boolean".to_string()),
        }
    }
    fn as_color(self) -> EResult<scene::Color> {
        match self {
            Value::Color(c) => Ok(c),
            _ => Err("Expected a color".to_string()),
        }
    }
    fn as_graphic(self) -> EResult<scene::Graphic> {
        match self {
            Value::Graphic(g) => Ok(g),
            _ => Err("Expected a graphic".to_string()),
        }
    }
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

        let graphic = Value::Graphic(scene::Graphic::Circle {
            x: 0.0,
            y: 0.0,
            radius: 10.0,
            color: scene::Color::Rgba {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        });
        context.set_var("ball".to_string(), graphic.clone());

        if let Value::Graphic(g) = context.get_var("ball").unwrap() {
            assert_eq!(
                g,
                scene::Graphic::Circle {
                    x: 0.0,
                    y: 0.0,
                    radius: 10.0,
                    color: scene::Color::Rgba {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0
                    },
                }
            );
        } else {
            panic!("Expected Graphic");
        }
    }
}
