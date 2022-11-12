use std::ptr::NonNull;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    child: Option<NonNull<Node<T>>>,
    level: usize,
    elem: T,
}

pub struct Multilist<T> {
    head: Option<NonNull<Node<T>>>,
    max_level: usize,
    len: usize,
}

impl<T> Node<T> {
    fn new(elem: T, level: usize) -> Self {
        Node { next: None, child: None, level, elem }
    }

    fn into_elem(self: Box<Self>) -> T {
        self.elem
    }

    fn add_child(&mut self, elem: T) {
        let node = Node::new(elem, self.level + 1);
        let child: Option<NonNull<Node<T>>> = Some((&node).into());
        self.child = child;
    }

    fn add_next(&mut self, elem: T) {
        let node = Node::new(elem, self.level);
        let next: Option<NonNull<Node<T>>> = Some((&node).into());
        self.next = next;
    }
}

impl<T> Multilist<T> {
    fn new() -> Self {
        Multilist { head: None, max_level: 0, len: 0 }
    }

    //TODO: Implement those:
    // fn insert_next (elem, at)
    // fn insert_child (elem, at)
    // fn pop (elem, at)
    // fn move (src, dst)
    // fn remove_levels_from (l)
    // fn clone
    // fn clear
    // fn display

}
