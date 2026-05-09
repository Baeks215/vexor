use crate::evaluator::EResult;

/// Linked List node
#[derive(Debug, Clone)]
pub enum ListNode<T: Clone> {
    Nil,
    Cons(T, Box<ListNode<T>>),
}

impl<T: Clone> ListNode<T> {
    pub fn map<F: Fn(T) -> EResult<T>>(self, f: F) -> EResult<ListNode<T>> {
        match self {
            ListNode::Nil => Ok(ListNode::Nil),
            ListNode::Cons(head, tail) => Ok(ListNode::Cons(f(head)?, Box::new(tail.map(f)?))),
        }
    }
}
