use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    child: Option<NonNull<Node<T>>>,
    level: usize,
    elem: T,
}

pub struct Multilist<T> {
    head: Option<NonNull<Node<T>>>,
    max_level: usize,
    len: usize,
}

pub struct Iter<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let node = node.as_ref();
                self.len -= 1;
                self.head = node.next;
                &node.elem
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IterMut<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.head.map(|node| unsafe {
                let node = &mut *node.as_ptr();
                self.len -= 1;
                self.head = node.next;
                &mut node.elem
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IntoIter<T> {
    list: Multilist<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> Node<T> {
    fn new(elem: T, level: usize) -> Self {
        Node {
            next: None,
            child: None,
            level,
            elem,
        }
    }

    fn into_elem(self: Box<Self>) -> T {
        self.elem
    }

    // TODO: maybe useless, uncomment later
    // fn add_child(&mut self, elem: T) {
    //     let node = Box::new(Node::new(elem, self.level + 1));
    //     let child: Option<NonNull<Node<T>>> = Some(Box::leak(node).into());
    //     self.child = child;
    // }
    //
    // fn add_next(&mut self, elem: T) {
    //     let node = Box::new(Node::new(elem, self.level));
    //     let next: Option<NonNull<Node<T>>> = Some(Box::leak(node).into());
    //     self.next = next;
    // }
}

impl<T> Multilist<T> {
    pub fn new() -> Self {
        Multilist {
            head: None,
            max_level: 0,
            len: 0,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let mut node = Box::new(Node::new(elem, 0));
        node.next = self.head;

        self.head = Some(Box::leak(node).into());
        self.len += 1;
    }

    pub fn push_back(&mut self, elem: T) {
        let node = Box::new(Node::new(elem, 0));
        if self.len == 0 {
            self.head = Some(Box::leak(node).into());
        } else {
            unsafe {
                let mut iter = self.head.unwrap();
                while !iter.as_ref().next.is_none() {
                    iter = iter.as_ref().next.unwrap();
                }
                iter.as_mut().next = Some(Box::leak(node).into());
            }
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| node.into_elem())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(|node| node.into_elem())
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        let new_last_node = if self.len <= 1 {
            self.head
        } else {
            self.get_node(self.len - 2)
        };

        unsafe {
            let pop_node = new_last_node?.as_ref().next.unwrap_or(new_last_node?);

            self.len -= 1;
            if self.len == 0 {
                self.head = None
            }

            new_last_node?.as_mut().next = None;
            Some(Box::from_raw(pop_node.as_ptr()))
        }
    }

    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            self.len -= 1;
            node
        })
    }

    fn get_node(&self, at: usize) -> Option<NonNull<Node<T>>> {
        if self.len == 0 || at >= self.len {
            return None;
        }

        let mut iter_elem = self.head?;
        let mut counter = 0;
        while counter < at {
            unsafe { iter_elem = iter_elem.as_ref().next? }
            counter += 1;
        }

        Some(iter_elem)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            head: self.head,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub fn iter_mut(&self) -> IterMut<T> {
        IterMut {
            head: self.head,
            len: self.len,
            marker: PhantomData,
        }
    }

    //TODO: Implement those:
    // fn insert_next (elem, at)
    // fn insert_child (elem, at)
    // fn pop (elem, at)
    // fn move (src, dst)
    // fn remove_levels_from (l)
    // fn clone
    // fn clear
    // fn display
}

impl<T> IntoIterator for Multilist<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T> IntoIterator for &'a Multilist<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Multilist<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> FromIterator<T> for Multilist<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T> Extend<T> for Multilist<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for each in iter {
            self.push_back(each);
        }
    }
}

impl<T> Clone for Multilist<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T> Display for Multilist<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for each in self.iter() {
            write!(f, "{}, ", each)?;
        }
        // \x08 == \b (backspace), but the latter is unsupported
        write!(f, "\x08\x08]")
    }
}

impl<T> Drop for Multilist<T> {
    fn drop(&mut self) {
        while let Some(node) = self.pop_front_node() {
            drop(node);
        }
    }
}
