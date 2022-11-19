use super::linked_list::LinkedList;
use super::Node;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ptr::NonNull;
use std::rc::Rc;

// 0 - level num, 1 - local node index
pub struct Index {
    level: usize,
    node: usize,
}

impl Index {
    pub fn new(level: usize, node: usize) -> Self {
        Index { level, node }
    }
}

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

    pub fn size(&self) -> usize {
        self.len
    }

    pub fn level_size(&self, level: usize) -> Option<usize> {
        let lists = self.index_map.get(&level)?;
        Some(lists.iter().map(|ls| ls.borrow().len()).sum())
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

    pub fn insert_alt(&mut self, at: Index, elem: T) -> Result<(), &str> {
        if at.node == 0 {
            return Err("wrong local node index, should be at least 1");
        }

        let stub_index = Index::new(at.level, at.node - 1);
        match self.get_sublist(&stub_index) {
            None => Err("can't find list at this index"),
            Some((list, index)) => {
                (*list).borrow_mut().insert(elem, index + 1);
                self.len += 1;
                Ok(())
            }
        }
    }

    pub fn attach_child(&mut self, at: Index, elem: T) -> Result<(), &str> {
        match self.get_sublist_node(&at) {
            None => Err("can't find node at this index"),
            Some(mut node) => {
                let node = unsafe { node.as_mut() };
                let mut list = LinkedList::new();
                list.push_back(elem);
                node.child = Some(Rc::from(Box::new(RefCell::new(list))));

                self.update_level_index(at.level + 1);
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
        let lists = self.index_map.get(&at.level)?;
        let mut local_index = at.node;
        for (i, list) in lists.iter().enumerate() {
            let list_len = list.borrow().len();
            let list_is_last = i == lists.len() - 1;
            if local_index < list_len || list_is_last && local_index == list_len {
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

impl<T> Display for MultiList<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lv0 - {}", self.index_map.get(&0).unwrap()[0].borrow())?;
        for level in 0..(self.index_map.len() - 1) {
            let mut vec: Vec<(usize, Rc<RefCell<LinkedList<T>>>)> = Vec::new();
            let pointers = self.index_map.get(&level).unwrap();
            let mut index_offset = 0;

            for list in pointers.iter().map(|r| (*r).borrow()) {
                for (i, node) in list.node_iter().enumerate() {
                    match &node.child {
                        Some(child) => vec.push((index_offset + i, child.clone())),
                        None => {}
                    }
                }
                index_offset += list.len();
            }

            writeln!(f, "Lv{} - {}", level + 1, MultiList::level_to_string(vec))?
        }
        Ok(())
    }
}

impl<T> MultiList<T>
where
    T: Display,
{
    fn level_to_string(vec: Vec<(usize, Rc<RefCell<LinkedList<T>>>)>) -> String {
        let mut string = String::new();
        for (i, each) in vec.iter().map(|pair| (pair.0, pair.1.borrow())) {
            string.push_str(format!("{}:{}  ", i, each).as_str())
        }
        string.trim().to_string()
    }
}
