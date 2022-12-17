use crate::lists::SkipNode;
use crate::MaybeNone;
use rand::{thread_rng, Rng};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct SkipList<V>
where
    V: Default,
{
    head: NonNull<SkipNode<usize, V>>,
    fraction: f64,
    max_level: usize,
    cur_level: usize,
    len: usize,
}

type UpdateVec<V> = Vec<MaybeNone<SkipNode<usize, V>>>;

const MAX_LEVEL: usize = u16::MAX as usize;

struct NodeIter<'a, V: 'a> {
    current: MaybeNone<SkipNode<usize, V>>,
    len: usize,
    marker: PhantomData<&'a SkipNode<usize, V>>,
}

impl<'a, V> Iterator for NodeIter<'a, V> {
    type Item = &'a SkipNode<usize, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.current.map(|node| unsafe {
                let node = node.as_ref();
                self.len -= 1;
                self.current = node.next[0];
                node
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<V> SkipList<V>
where
    V: Default,
{
    pub fn new(fraction: f64, max_level: usize) -> Self {
        let nil = SkipNode::<usize, V>::new(usize::MAX, V::default(), max_level);

        SkipList {
            head: NonNull::from(Box::leak(Box::new(nil))),
            fraction,
            max_level,
            cur_level: 0,
            len: 0,
        }
    }

    pub fn with_fraction(fraction: f64) -> Self {
        Self::new(fraction, MAX_LEVEL)
    }

    pub fn clear(&mut self) {
        *self = Self::new(self.fraction, self.max_level)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cur_level(&self) -> usize {
        self.cur_level
    }

    pub fn max_level(&self) -> usize {
        self.max_level
    }

    pub fn insert(&mut self, key: usize, value: V) -> Result<(), &str> {
        unsafe {
            let (current, mut update) = self.find_node_update(key);
            if current.is_none() || current.unwrap().as_ref().key != key {
                let level = self.random_level();

                if level > self.cur_level {
                    for i in (self.cur_level + 1)..=level {
                        update[i] = Some(self.head)
                    }
                    self.cur_level = level
                }

                let node = SkipNode::new(key, value, level);
                let mut node_ptr = NonNull::from(Box::leak(Box::new(node)));

                for i in 0..=level {
                    let each = update[i].unwrap().as_mut();
                    node_ptr.as_mut().next[i] = each.next[i];
                    each.next[i] = Some(node_ptr);
                }

                self.len += 1;
                Ok(())
            } else {
                Err("provided key already exists")
            }
        }
    }

    fn random_level(&self) -> usize {
        let mut rng = thread_rng();
        let mut level = 0;

        while rng.gen_range(0.0..1.0) < self.fraction && level < self.max_level {
            level += 1
        }

        level
    }

    pub fn pop(&mut self, key: usize) -> Option<V> {
        Some(self.pop_node(key)?.value)
    }

    fn pop_node(&mut self, key: usize) -> Option<Box<SkipNode<usize, V>>> {
        unsafe {
            let (current, update) = self.find_node_update(key);
            let current_key = current?.as_ref().key;

            if current_key != key {
                return None;
            }

            for i in 0..=self.cur_level {
                let update_each = update[i].unwrap().as_mut();
                if update_each.next[i] != current {
                    break;
                }
                update_each.next[i] = current.unwrap().as_ref().next[i]
            }

            while self.cur_level > 0 && self.head.as_ref().next[self.cur_level].is_none() {
                self.cur_level -= 1;
            }

            self.len -= 1;
            Some(Box::from_raw(current?.as_ptr()))
        }
    }

    fn find_node_update(&self, key: usize) -> (MaybeNone<SkipNode<usize, V>>, UpdateVec<V>) {
        let mut update = vec![None; self.max_level + 1];
        let mut current = Some(self.head);

        unsafe {
            for lvl in (0..=self.cur_level).rev() {
                Self::iter_node_on_level(&mut current, key, lvl);
                update[lvl] = current;
            }

            current = current.unwrap().as_ref().next[0];
            (current, update)
        }
    }

    pub fn find(&self, key: usize) -> Option<&V> {
        unsafe {
            let mut current = Some(self.head);

            for lvl in (0..=self.cur_level).rev() {
                Self::iter_node_on_level(&mut current, key, lvl);
            }

            current = current.unwrap().as_ref().next[0];

            let current_ref = current?.as_ref();
            if current_ref.key == key {
                Some(&current_ref.value)
            } else {
                None
            }
        }
    }

    fn node_iter(&self) -> NodeIter<V> {
        NodeIter {
            current: unsafe { self.head.as_ref().next[0] },
            len: self.len,
            marker: PhantomData,
        }
    }

    unsafe fn iter_node_on_level(
        current: &mut MaybeNone<SkipNode<usize, V>>,
        search_key: usize,
        lvl: usize,
    ) {
        while let Some(next) = current.unwrap().as_ref().next[lvl] {
            if next.as_ref().key < search_key {
                *current = Some(next);
            } else {
                break;
            }
        }
    }

    pub fn node_ptrs(&self) -> usize {
        let mut node_ptrs = 0;

        for node in self.node_iter() {
            for ptr in &node.next {
                if ptr.is_some() {
                    node_ptrs += 1;
                }
            }
        }
        unsafe {
            for ptr in &self.head.as_ref().next {
                if ptr.is_some() {
                    node_ptrs += 1;
                }
            }
        }

        node_ptrs
    }
}

impl<V> Default for SkipList<V>
where
    V: Default,
{
    fn default() -> Self {
        Self::new(0.5, MAX_LEVEL)
    }
}

impl<V> Clone for SkipList<V>
where
    V: Default + Clone,
{
    fn clone(&self) -> Self {
        let mut clone = SkipList::new(self.fraction, self.max_level);

        for each in self.node_iter() {
            clone.insert(each.key, each.value.clone()).unwrap_or(());
        }

        clone
    }
}

impl<V> Debug for SkipList<V>
where
    V: Default,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for lvl in 0..self.cur_level + 1 {
            write!(f, "Lv{} - ", lvl)?;
            let mut node = unsafe { (self.head.as_ref()).next[lvl] };
            while node.is_some() {
                let node_ref = unsafe { node.unwrap().as_ref() };
                write!(f, "{} ", node_ref.key)?;
                node = node_ref.next[lvl];
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<V> Drop for SkipList<V>
where
    V: Default,
{
    fn drop(&mut self) {
        unsafe {
            let mut key = self.head.as_ref().next[0].unwrap().as_ref().key;
            while !self.is_empty() {
                if let Some(node) = self.pop_node(key) {
                    if let Some(next) = node.next[0] {
                        key = next.as_ref().key;
                    }

                    drop(node)
                }
            }

            drop(Box::from_raw(self.head.as_ptr()))
        }
    }
}
