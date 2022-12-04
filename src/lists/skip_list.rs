use crate::lists::SkipNode;
use rand::{thread_rng, Rng};

pub struct SkipList<V> {
    head: *mut SkipNode<usize, V>,
    fraction: f64,
    max_level: usize,
    cur_level: usize,
    len: usize,
}

impl<V> SkipList<V> {
    pub fn new(fraction: f64, max_level: usize) -> Self {
        let nil = SkipNode::<usize, V>::new(usize::MAX, None, max_level);
        let nil_ptr = Box::leak(Box::new(nil));

        SkipList {
            head: nil_ptr,
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
        let mut update = vec![std::ptr::null_mut(); self.max_level + 1];
        let mut current = self.head;

        unsafe {
            for i in (0..=self.cur_level).rev() {
                while !(*current).next[i].is_null() && (*(*current).next[i]).key < key {
                    current = (*current).next[i]
                }

                update[i] = current;
            }

            current = (*current).next[0];

            if current.is_null() || (*current).key != key {
                let rlevel = self.random_level();

                if rlevel > self.cur_level {
                    for i in (self.cur_level + 1)..(rlevel + 1) {
                        update[i] = self.head
                    }
                    self.cur_level = rlevel
                }

                let node = SkipNode::new(key, Some(value), rlevel);
                let node_ref = Box::leak(Box::new(node));

                for i in 0..rlevel + 1 {
                    node_ref.next[i] = (*update[i]).next[i];
                    (*update[i]).next[i] = node_ref;
                }
            }
        }
    }

    pub fn display(&self) {
        unsafe {
            for lvl in 0..self.cur_level + 1 {
                print!("Level {}:  ", lvl);
                let mut node = (*self.head).next[lvl];
                while !node.is_null() {
                    print!("{} ", (*node).key);
                    node = (*node).next[lvl];
                }
                println!();
            }
        }
    }
}

impl<V> Default for SkipList<V> {
    fn default() -> Self {
        Self::new(0.5, usize::MAX)
    }
}
