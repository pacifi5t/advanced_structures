use crate::lists::SkipNode;
use rand::{thread_rng, Rng};
use std::ptr::NonNull;

pub struct SkipList<V> {
    head: NonNull<SkipNode<usize, V>>,
    fraction: f64,
    max_level: usize,
    cur_level: usize,
    len: usize,
}

type MaybeNone<T> = Option<NonNull<T>>;
type UpdateVec<V> = Vec<MaybeNone<SkipNode<usize, V>>>;

impl<V> SkipList<V>
where
    V: Clone,
{
    pub fn new(fraction: f64, max_level: usize) -> Self {
        let nil = SkipNode::<usize, V>::new(usize::MAX, None, max_level);

        SkipList {
            head: NonNull::from(Box::leak(Box::new(nil))),
            fraction,
            max_level,
            cur_level: 0,
            len: 0,
        }
    }

    pub fn random_level(&self) -> usize {
        let mut rng = thread_rng();
        let mut level = 0;

        while rng.gen_range(0.0..1.0) < self.fraction && level < self.max_level {
            level += 1
        }

        level
    }

    pub fn insert(&mut self, key: usize, value: V) {
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

                let node = SkipNode::new(key, Some(value), level);
                let mut node_ptr = NonNull::from(Box::leak(Box::new(node)));

                for i in 0..=level {
                    let each = update[i].unwrap().as_mut();
                    node_ptr.as_mut().next[i] = each.next[i];
                    each.next[i] = Some(node_ptr);
                }

                self.len += 1;
            }
            //TODO: return error that key exists
        }
    }

    pub fn remove(&mut self, key: usize) -> Option<V> {
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

            Box::from_raw(current?.as_ptr()).value
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

    pub fn find(&self, key: usize) -> Option<V> {
        unsafe {
            let mut current = Some(self.head);

            for lvl in (0..=self.cur_level).rev() {
                Self::iter_node_on_level(&mut current, key, lvl);
            }

            current = current.unwrap().as_ref().next[0];
            current?.as_ref().value.clone()
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

    pub fn display(&self) {
        unsafe {
            for lvl in 0..self.cur_level + 1 {
                print!("Level {}:  ", lvl);
                let mut node = (self.head.as_ref()).next[lvl];
                while node.is_some() {
                    let node_ref = node.unwrap().as_ref();
                    print!("{} ", node_ref.key);
                    node = node_ref.next[lvl];
                }
                println!();
            }
        }
    }
}

impl<V> Default for SkipList<V>
where
    V: Clone,
{
    fn default() -> Self {
        Self::new(0.5, usize::MAX)
    }
}

//TODO: IMPLEMENT DESTRUCTOR
