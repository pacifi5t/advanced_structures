use std::ptr::NonNull;

pub mod linked_list;
mod multilist;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    child: Option<NonNull<Node<T>>>,
    level: usize,
    elem: T,
}
