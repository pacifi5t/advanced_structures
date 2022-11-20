use crate::lists::Node;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
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

pub(super) struct NodeIter<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            0 => None,
            _ => self.head.map(|node| unsafe {
                let node = node.as_ref();
                self.len -= 1;
                self.head = node.next;
                node
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub(super) struct NodeIterMut<'a, T: 'a> {
    head: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for NodeIterMut<'a, T> {
    type Item = &'a mut Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.len {
            0 => None,
            _ => self.head.map(|mut node| unsafe {
                let node = node.as_mut();
                self.len -= 1;
                self.head = node.next;
                node
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
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

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_front(&mut self, elem: T) {
        self.push_front_node(Box::new(Node::new(elem)));
    }

    pub fn push_back(&mut self, elem: T) {
        self.push_back_node(Box::new(Node::new(elem)))
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(|node| node.into_elem())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(|node| node.into_elem())
    }

    fn push_back_node(&mut self, node: Box<Node<T>>) {
        let node = Some(Box::leak(node).into());
        match self.tail {
            None => self.head = node,
            Some(tail) => unsafe { (*tail.as_ptr()).next = node },
        }

        self.tail = node;
        self.len += 1;
    }

    fn push_front_node(&mut self, mut node: Box<Node<T>>) {
        node.next = self.head;
        let node = Some(Box::leak(node).into());

        if self.head.is_none() {
            self.tail = node
        }

        self.head = node;
        self.len += 1;
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        let new_tail = match self.len {
            0 | 1 => self.head,
            _ => self.get_node(self.len - 2),
        };

        unsafe {
            let node = Some(Box::from_raw(self.tail?.as_ptr()));
            self.tail = new_tail;

            self.len -= 1;
            if self.is_empty() {
                self.head = None;
                self.tail = None;
            }

            new_tail?.as_mut().next = None;
            node
        }
    }

    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            if self.head.is_none() {
                self.tail = None
            }

            self.len -= 1;
            node
        })
    }

    pub(super) fn get_node(&self, at: usize) -> Option<NonNull<Node<T>>> {
        if self.is_empty() || at >= self.len {
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

    pub fn insert(&mut self, elem: T, at: usize) {
        self.insert_node(Box::new(Node::new(elem)), at)
    }

    pub(super) fn insert_node(&mut self, mut node: Box<Node<T>>, at: usize) {
        assert!(
            (0..=self.len).contains(&at),
            "Index is out of bounds 0..=len"
        );

        if at == 0 {
            return self.push_front_node(node);
        } else if at == self.len {
            return self.push_back_node(node);
        }

        unsafe {
            let mut node_before = self.get_node(at - 1).unwrap();
            node.next = node_before.as_ref().next;
            node_before.as_mut().next = Some(Box::leak(node).into());
            self.len += 1;
        }
    }

    pub fn pop(&mut self, at: usize) -> Option<T> {
        Some(self.pop_node(at)?.into_elem())
    }

    pub(super) fn pop_node(&mut self, at: usize) -> Option<Box<Node<T>>> {
        assert!((0..self.len).contains(&at), "Index is out of bounds");

        if at == 0 {
            return self.pop_front_node();
        } else if at == self.len {
            return self.pop_back_node();
        }

        unsafe {
            let mut node_before = self.get_node(at - 1)?;
            let node = Box::from_raw(node_before.as_ref().next?.as_ptr());
            node_before.as_mut().next = node.next;
            self.len -= 1;
            Some(node)
        }
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

    pub(super) fn node_iter(&self) -> NodeIter<T> {
        NodeIter {
            head: self.head,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub(super) fn node_iter_mut(&self) -> NodeIterMut<T> {
        NodeIterMut {
            head: self.head,
            len: self.len,
            marker: PhantomData,
        }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for each in iter {
            self.push_back(each);
        }
    }
}

impl<T> Clone for LinkedList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T> Display for LinkedList<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for each in self.iter() {
            write!(f, "{}, ", each)?;
        }

        if !self.is_empty() {
            write!(f, "\x08\x08")?
        }
        write!(f, "]")
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(node) = self.pop_front_node() {
            drop(node);
        }
    }
}
