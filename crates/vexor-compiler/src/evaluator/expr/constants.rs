use std::f64::consts::PI;

use crate::evaluator::expr::Value;
use crate::ir::ast::Const;

pub fn get_constant(c: Const) -> Value {
    match c {
        Const::Pi => Value::from(PI),
    }
}
