use super::linked_list::LinkedList;

pub struct MultiList<T> {
    head: Box<LinkedList<T>>,
    len: usize,
    levels_count: usize,
}

impl<T> MultiList<T> {
    pub fn new() -> Self {
        MultiList {
            head: Box::new(LinkedList::new()),
            len: 0,
            levels_count: 0,
        }
    }
}
