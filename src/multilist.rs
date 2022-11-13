use std::fmt::{Display, Formatter};
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
        let node = Box::new(Node::new(elem, self.level + 1));
        let child: Option<NonNull<Node<T>>> = Some(Box::leak(node).into());
        self.child = child;
    }

    fn add_next(&mut self, elem: T) {
        let node = Box::new(Node::new(elem, self.level));
        let next: Option<NonNull<Node<T>>> = Some(Box::leak(node).into());
        self.next = next;
    }
}

impl<T> Multilist<T> {
    pub fn new() -> Self {
        Multilist { head: None, max_level: 0, len: 0 }
    }

    pub fn push_front(&mut self, elem: T) {
        let mut node = Box::new(Node::new(elem, 0));
        node.next = self.head;

        self.head = Some(Box::leak(node).into());
        self.len += 1;
    }

    pub fn push_back(&mut self, elem: T) {
        let node = Box::new(Node::new(elem, 0));
        if self.len == 0 {
            self.head = Some(Box::leak(node).into());
        } else {
            unsafe {
                let mut iter = self.head.unwrap();
                while !iter.as_ref().next.is_none() {
                    iter = iter.as_ref().next.unwrap();
                }
                iter.as_mut().next = Some(Box::leak(node).into());
            }
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| node.into_elem())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(|node| node.into_elem())
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        unsafe {
            let mut ctr: isize = 0;
            let mut iter = self.head;
            while ctr < self.len as isize - 2 && !iter?.as_ref().next.is_none() {
                iter = iter?.as_ref().next;
                ctr += 1;
            }

            let prev = iter?.as_mut();
            let tail = Box::from_raw(prev.next.unwrap_or(iter?).as_ptr());
            prev.next = None;

            self.len -= 1;

            if self.len == 0 {
                self.head = None;
            }

            Some(tail)
        }
    }

    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            self.len -= 1;
            node
        })
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

impl<T> Display for Multilist<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.head;
        while !iter.is_none() {
            unsafe {
                write!(f, "{}, ", iter.unwrap().as_ref().elem)?;
                iter = iter.unwrap().as_ref().next;
            }
        }
        // \x08 == \b (backspace), but the latter is unsupported
        write!(f, "\x08\x08]")
    }
}

impl<T> Drop for Multilist<T> {
    fn drop(&mut self) {
        while let Some(mut node) = self.pop_front_node() {
            drop(node);
        }
    }
}
