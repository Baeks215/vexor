use std::fmt::Debug;

use crate::evaluator::EResult;
use crate::evaluator::expr::Callable;
use crate::evaluator::expr::list::ListNode;
use crate::ir::{Number, scene};

pub trait Evaluable {
    type Output: Debug + Clone;
    /// Converts a [`Value`] to an evaluated output
    fn expect(value: Value) -> EResult<Self::Output>;
}

// Used for generic eval where type is not known at compile time.
impl Evaluable for ty::Any {
    type Output = Value;
    fn expect(value: Value) -> EResult<Self::Output> {
        Ok(value)
    }
}

macro_rules! define_value_types {
    (
        $(
            $variant:ident($out_type:ty)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum Value {
            $(
                $variant(<ty::$variant as Evaluable>::Output),
            )*
        }

        /// Marker Types used to expect type at compile time
        pub mod ty {
            pub struct Any;
            $(
                pub struct $variant;
            )*
        }

        $(
            impl Evaluable for ty::$variant {
                type Output = $out_type;
                fn expect(value: Value) -> EResult<Self::Output> {
                    match value {
                        Value::$variant(x) => Ok(x),
                        _ => Err(format!("expected a {}", stringify!($variant).to_lowercase())),
                    }
                }
            }

            // From conversion to construct Values from output types
            impl From<<ty::$variant as Evaluable>::Output> for Value {
                fn from(value: <ty::$variant as Evaluable>::Output) -> Self {
                    Value::$variant(value)
                }
            }
        )*
    };
}

// --- Define types ---
// Value enum variant -> evaluated output
define_value_types! {
    Number(Number),
    String(String),
    Bool(bool),
    Color(scene::Color),
    Graphic(scene::Graphic),
    List(Box<ListNode<Value>>),
    Function(Callable),
}
