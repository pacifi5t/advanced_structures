pub use linked_list::LinkedList;
pub use multilist::MultiList;
use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

mod linked_list;
pub mod multilist;

#[derive(Clone)]
struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    child: Option<Rc<RefCell<LinkedList<T>>>>,
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

    fn into_elem(self) -> T {
        self.elem
    }
}
