pub mod ast;
pub mod scene;

/// User Number type in the compiler: always a 64-bit floating point number
pub type Number = f64;

/// Linked List node
#[derive(Debug, Clone)]
pub enum ListNode<T: Clone> {
    Nil,
    Cons(T, Box<ListNode<T>>),
}
