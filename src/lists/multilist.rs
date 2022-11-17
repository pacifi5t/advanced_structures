use super::linked_list::LinkedList;
use crate::lists::Node;
use std::collections::HashMap;
use std::ptr::NonNull;

// 0 - level num, 1 - local node index
pub struct Index(pub usize, pub usize);

pub struct MultiList<T> {
    head: Box<LinkedList<T>>,
    len: usize,
    levels: HashMap<usize, usize>,
}

impl<T> MultiList<T> {
    pub fn new() -> Self {
        MultiList {
            head: Box::new(LinkedList::new()),
            len: 0,
            levels: HashMap::from([(0, 0)]),
        }
    }

    pub fn insert(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_sublist(&at) {
            Some((mut list, index)) => unsafe {
                list.as_mut().insert(elem, index);
                (*self.levels.get_mut(&at.0).unwrap()) += 1;
                self.len += 1;
                Ok(())
            },
            None => Err("can't find list at this index"),
        }
    }

    pub fn insert_child(at: Index, elem: T) {
        todo!()
    }

    fn get_node(&mut self, at: &Index) -> Option<NonNull<Node<T>>> {
        let (list, index) = self.get_sublist(at)?;
        unsafe { list.as_ref().get_node(index) }
    }

    fn get_sublist(&mut self, at: &Index) -> Option<(NonNull<LinkedList<T>>, usize)> {
        if !(0..self.levels.len()).contains(&at.0) {
            return None;
        }

        let mut local_index = at.1;
        let mut list_map = self.build_index_map(at.0);
        for list in list_map.get_mut(&at.0).unwrap() {
            if list.len() <= local_index {
                return Some(((*list).into(), local_index));
            } else {
                local_index -= list.len();
            }
        }

        None
    }

    fn build_index_map(&mut self, max_level: usize) -> HashMap<usize, Vec<&mut LinkedList<T>>> {
        let mut list_map = HashMap::from([(0, vec![self.head.as_mut()])]);
        for level in 0..max_level {
            let mut vec = Vec::new();
            for list in list_map.get(&level).unwrap() {
                let mut v = Vec::new();
                for n in list.node_iter() {
                    if n.child.is_some() {
                        unsafe { v.push(n.child.unwrap().as_mut()) }
                    }
                }
                vec.append(&mut v);
            }
            list_map.insert(level + 1, vec);
        }
        list_map
    }
}
