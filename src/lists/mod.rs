use linked_list::LinkedList;
pub use multilist::MultiList;
pub use skip_list::SkipList;
use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

mod linked_list;
pub mod multilist;
pub mod skip_list;

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

#[derive(Clone)]
struct SkipNode<K, V> {
    next: Vec<Option<NonNull<SkipNode<K, V>>>>,
    key: K,
    value: V,
}

impl<K, V> SkipNode<K, V> {
    fn new(key: K, value: V, level: usize) -> Self {
        SkipNode {
            next: vec![None; level + 1],
            key,
            value,
        }
    }
}
