use super::LinkedList;
use super::Node;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ptr::NonNull;
use std::rc::Rc;

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
        let index_map = HashMap::from([(0, vec![head])]);
        MultiList { len: 0, index_map }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn size(&self) -> usize {
        self.len
    }

    pub fn levels(&self) -> usize {
        self.index_map.len()
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
                if index >= list.borrow().len() {
                    return Err("index out of bounds");
                }

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

                if node.child.is_some() {
                    return Err("child already exists");
                } else {
                    let mut list = LinkedList::new();
                    list.push_back(elem);
                    node.child = Some(Rc::from(Box::new(RefCell::new(list))))
                }

                self.update_level_index(at.level + 1);
                self.len += 1;
                Ok(())
            }
        }
    }

    pub fn detach_child(&mut self, at: Index) -> Result<(), &str> {
        match self.get_sublist_node(&at) {
            None => Err("can't find node at this index"),
            Some(mut node) => {
                let node = unsafe { node.as_mut() };
                node.child = None;

                self.update_index(at.level + 1);
                self.recalc_size();
                Ok(())
            }
        }
    }

    pub fn pop(&mut self, at: Index) -> Result<T, &str> {
        match self.get_sublist(&at) {
            None => Err("can't find list at this index"),
            Some((list, index)) => {
                if index >= list.borrow().len() {
                    return Err("index out of bounds");
                }

                let elem = (*list).borrow_mut().pop(index).unwrap();
                self.update_index(at.level);
                self.recalc_size();
                Ok(elem)
            }
        }
    }

    pub fn remove_level(&mut self, level: usize) -> Result<(), &str> {
        if level >= self.levels() {
            return Err("provided level does not exist");
        } else if level == 0 {
            self.clear();
            return Ok(());
        }

        for list in self.index_map.get(&level).unwrap() {
            for node in list.borrow_mut().node_iter_mut() {
                node.child = None;
            }
        }

        (level..self.levels()).for_each(|lv| {
            self.len -= self.level_size(lv).unwrap();
            self.index_map.remove(&lv);
        });

        Ok(())
    }

    pub fn move_elem(&mut self, src: Index, dst: Index) -> Result<(), &str> {
        let maybe_src_list = self.get_sublist(&src);
        if maybe_src_list.is_none() {
            return Err("can't find list at source index");
        }

        let (list, at) = maybe_src_list.unwrap();
        let node = list.borrow_mut().pop_node(at).unwrap();

        let maybe_dst_list = self.get_sublist(&dst);
        if maybe_dst_list.is_none() {
            return Err("can't find list at destination index");
        }

        let (list, at) = maybe_dst_list.unwrap();
        list.borrow_mut().insert_node(node, at);

        self.update_index(1);
        self.recalc_size();
        Ok(())
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
        let children = self.get_children_of_level(level - 1);
        let vec: Vec<Rc<RefCell<LinkedList<T>>>> =
            children.iter().map(|(_, c)| c.clone()).collect();

        if vec.is_empty() {
            self.index_map.remove(&level);
        } else {
            self.index_map.insert(level, vec);
        }
    }

    fn update_index(&mut self, from: usize) {
        let from = if from == 0 { 1 } else { from };
        (from..self.levels()).for_each(|level| {
            self.update_level_index(level);
        });
    }

    fn recalc_size(&mut self) {
        self.len = 0;
        for level in self.index_map.values() {
            for list in level {
                self.len += list.borrow().len();
            }
        }
    }

    fn get_children_of_level(&self, level: usize) -> Vec<(usize, Rc<RefCell<LinkedList<T>>>)> {
        let mut vec: Vec<(usize, Rc<RefCell<LinkedList<T>>>)> = Vec::new();
        let mut index_offset = 0;

        let pointers = self.index_map.get(&level);
        if pointers.is_none() {
            return vec;
        }

        for list in pointers.unwrap().iter().map(|r| (*r).borrow_mut()) {
            for (i, node) in list.node_iter_mut().enumerate() {
                match &node.child {
                    Some(child) => {
                        // This is a hack: we don't need empty lists in our array,
                        // so we should remove refs to empty children
                        if child.borrow().is_empty() {
                            node.child = None;
                        } else {
                            vec.push((index_offset + i, child.clone()))
                        }
                    }
                    None => {}
                }
            }
            index_offset += list.len();
        }

        vec
    }
}

impl<T> Default for MultiList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Display for MultiList<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lv0 - {}", self.index_map.get(&0).unwrap()[0].borrow())?;

        for level in 0..(self.index_map.len() - 1) {
            let vec = self.get_children_of_level(level);
            let mut string = String::new();
            for (i, each) in vec.iter().map(|pair| (pair.0, pair.1.borrow())) {
                string.push_str(format!("{}:{}  ", i, each).as_str())
            }

            writeln!(f, "Lv{} - {}", level + 1, string.trim())?
        }

        Ok(())
    }
}

impl<T> Clone for MultiList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new = MultiList::<T>::new();
        new.len = self.len;

        let mut first_level = Vec::new();
        for list in self.index_map.get(&0).unwrap() {
            let list_clone = list.borrow().clone();
            first_level.push(Rc::from(Box::new(RefCell::new(list_clone))));
        }
        new.index_map.insert(0, first_level);

        for level in 0..(self.index_map.len() - 1) {
            let vec: Vec<(usize, Rc<RefCell<LinkedList<T>>>)> = self
                .get_children_of_level(level)
                .iter()
                .map(|(i, child)| (*i, Rc::from(Box::new(RefCell::new(child.borrow().clone())))))
                .collect();

            for (node, list) in &vec {
                let mut parent = new.get_sublist_node(&Index::new(level, *node)).unwrap();
                unsafe { parent.as_mut().child = Some(list.clone()) }
            }

            let v = vec.iter().map(|(_, ls)| ls.clone()).collect();
            new.index_map.insert(level + 1, v);
        }

        new
    }
}
