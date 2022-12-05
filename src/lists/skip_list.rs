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

impl<V> SkipList<V> {
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
        let mut update = vec![None; self.max_level + 1];
        let mut current = Some(self.head);

        unsafe {
            for i in (0..=self.cur_level).rev() {
                while let Some(next) = current.unwrap().as_ref().next[i] {
                    if next.as_ref().key < key {
                        current = Some(next);
                    } else {
                        break;
                    }
                }

                update[i] = current;
            }

            current = current.unwrap().as_ref().next[0];

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

impl<V> Default for SkipList<V> {
    fn default() -> Self {
        Self::new(0.5, usize::MAX)
    }
}
