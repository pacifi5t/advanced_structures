use std::cell::RefCell;
use super::linked_list::LinkedList;
use crate::lists::Node;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

// 0 - level num, 1 - local node index
pub struct Index(pub usize, pub usize);

pub struct MultiList<T> {
    head: Rc<RefCell<LinkedList<T>>>,
    len: usize,
    levels: HashMap<usize, usize>,
}

impl<T> MultiList<T> {
    pub fn new() -> Self {
        MultiList {
            head: Rc::from(RefCell::new(LinkedList::new())),
            len: 0,
            levels: HashMap::from([(0, 0)]),
        }
    }

    pub fn insert(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_sublist(&at) {
            None => Err("can't find list at this index"),
            Some((list, index)) => {
                (*list).borrow_mut().insert(elem, index);
                (*self.levels.get_mut(&at.0).unwrap()) += 1;
                self.len += 1;
                Ok(())
            },
        }
    }

    pub fn insert_child(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_node(&at) {
            None => Err("can't find node at this index"),
            Some(mut node) => {
                let node = unsafe { node.as_mut() };
                let mut list = LinkedList::new();
                list.push_back(elem);
                node.child = Some(Rc::from(Box::new(RefCell::new(list))));

                let next_level_size = self.levels.get(&(at.0 + 1)).unwrap_or(&0);
                self.levels.insert(at.0 + 1, next_level_size + 1);
                self.len += 1;
                Ok(())
            }
        }
    }

    fn get_node(&self, at: &Index) -> Option<NonNull<Node<T>>> {
        let (list, index) = self.get_sublist(at)?;
        let node = list.borrow().get_node(index);
        node
    }

    fn get_sublist(&self, at: &Index) -> Option<(Rc<RefCell<LinkedList<T>>>, usize)> {
        if !(0..self.levels.len()).contains(&at.0) {
            return None;
        }

        let mut local_index = at.1;
        let list_map = self.build_index_map(at.0);
        for list in list_map.get(&at.0).unwrap() {
            let list_len = list.borrow().len();
            if local_index <= list_len {
                return Some((list.clone(), local_index));
            } else {
                local_index -= list_len;
            }
        }

        None
    }

    //TODO: Can be removed later
    fn build_index_map(&self, max_level: usize) -> HashMap<usize, Vec<Rc<RefCell<LinkedList<T>>>>> {
        let mut list_map = HashMap::from([(0, vec![self.head.clone()])]);
        for level in 0..max_level {
            let mut vec = Vec::new();
            for list in list_map.get(&level).unwrap() {
                let mut v = Vec::new();
                for n in (*list).borrow().node_iter() {
                    match &n.child {
                        Some(child) => v.push(child.clone()),
                        None => {},
                    };
                }
                vec.append(&mut v);
            }
            list_map.insert(level + 1, vec);
        }
        list_map
    }
}
