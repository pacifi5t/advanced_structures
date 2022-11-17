pub use linked_list::LinkedList;
pub use multilist::MultiList;
use std::ptr::NonNull;

mod linked_list;
mod multilist;

#[derive(Clone)]
struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    child: Option<NonNull<LinkedList<T>>>,
    elem: T,
}

impl<T> Node<T> {
    fn new(elem: T) -> Self {
        Node {
            next: None,
            child: None,
            elem,
        }
    }

    fn into_elem(self: Box<Self>) -> T {
        self.elem
    }
}
