/// Linked List node
#[derive(Debug, Clone)]
pub enum ListNode<T: Clone> {
    Nil,
    Cons(T, Box<ListNode<T>>),
}
