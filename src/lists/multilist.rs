use super::linked_list::LinkedList;
use super::Node;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

// 0 - level num, 1 - local node index
pub struct Index(pub usize, pub usize);

pub struct MultiList<T> {
    len: usize,
    index_map: HashMap<usize, Vec<Rc<RefCell<LinkedList<T>>>>>,
}

impl<T> MultiList<T> {
    pub fn new() -> Self {
        let head = Rc::from(RefCell::new(LinkedList::new()));
        let index_map = HashMap::from([(0, vec![head.clone()])]);
        MultiList { len: 0, index_map }
    }

    pub fn insert(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_sublist(&at) {
            None => Err("can't find list at this index"),
            Some((list, index)) => {
                (*list).borrow_mut().insert(elem, index);
                self.len += 1;
                Ok(())
            }
        }
    }

    pub fn insert_child(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_sublist_node(&at) {
            None => Err("can't find node at this index"),
            Some(mut node) => {
                let node = unsafe { node.as_mut() };
                let mut list = LinkedList::new();
                list.push_back(elem);
                node.child = Some(Rc::from(Box::new(RefCell::new(list))));

                self.update_level_index(at.0 + 1);
                self.len += 1;
                Ok(())
            }
        }
    }

    fn get_sublist_node(&self, at: &Index) -> Option<NonNull<Node<T>>> {
        let (list, index) = self.get_sublist(at)?;
        let node = list.borrow().get_node(index);
        node
    }

    fn get_sublist(&self, at: &Index) -> Option<(Rc<RefCell<LinkedList<T>>>, usize)> {
        let lists = self.index_map.get(&at.0)?;
        let mut local_index = at.1;
        for list in lists {
            let list_len = list.borrow().len();
            if local_index <= list_len {
                return Some((list.clone(), local_index));
            } else {
                local_index -= list_len;
            }
        }

        None
    }

    fn update_level_index(&mut self, level: usize) {
        let mut vec = Vec::new();
        let lists = self.index_map.get(&(level - 1)).unwrap();

        for list in lists {
            let mut v = Vec::new();
            for n in (*list).borrow().node_iter() {
                match &n.child {
                    Some(child) => v.push(child.clone()),
                    None => {}
                };
            }
            vec.append(&mut v);
        }

        self.index_map.insert(level, vec);
    }
}
