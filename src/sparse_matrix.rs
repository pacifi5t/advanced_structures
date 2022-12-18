use crate::MaybeNone;
use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

#[derive(Clone)]
struct Node<T> {
    value: Option<T>,
    row: usize,
    col: usize,
    next_row: MaybeNone<Node<T>>,
    next_col: MaybeNone<Node<T>>,
}

impl<T> Node<T> {
    fn new(value: T, row: usize, col: usize) -> Self {
        Node {
            value: Some(value),
            row,
            col,
            next_col: None,
            next_row: None,
        }
    }
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Node {
            value: None,
            row: usize::MAX,
            col: usize::MAX,
            next_row: None,
            next_col: None,
        }
    }
}

pub struct SparseMatrix<T: Default> {
    rows_vec: Vec<NonNull<Node<T>>>,
    cols_vec: Vec<NonNull<Node<T>>>,
}

impl<T> SparseMatrix<T>
where
    T: Default + Copy + PartialEq,
{
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows_vec: Self::create_empty_nodes_vec(rows),
            cols_vec: Self::create_empty_nodes_vec(cols),
        }
    }

    fn create_empty_nodes_vec(size: usize) -> Vec<NonNull<Node<T>>> {
        let node: Box<Node<T>> = Box::default();
        let mut vec = Vec::new();

        for _ in 0..size {
            vec.push(Box::leak(node.clone()).into());
        }

        vec
    }

    pub fn from_2d_vec(vec: Vec<Vec<T>>) -> Self {
        let cols = vec.iter().map(|v| v.len()).max_by(|x, y| x.cmp(y)).unwrap();

        let matrix = Self::new(vec.len(), cols);

        for (i, v) in vec.iter().enumerate() {
            let mut prev = Some(matrix.rows_vec[i]);

            for (j, value) in v.iter().enumerate().filter(|t| *t.1 != T::default()) {
                let node = Box::new(Node::new(*value, i, j));
                let current = Some(Box::leak(node).into());

                unsafe { prev.unwrap().as_mut().next_row = current };
                prev = current;
            }
        }

        let mut update = matrix.cols_vec.clone();
        for i in 0..matrix.rows_vec.len() {
            for j in 0..update.len() {
                if let Some(node) = matrix.get_node(i, j) {
                    unsafe { update[j].as_mut().next_col = Some(node) };
                    update[j] = node;
                }
            }
        }

        matrix
    }

    pub fn set(&self, value: T, row: usize, col: usize) {
        todo!()
    }

    pub fn get(&self, row: usize, col: usize) -> T {
        if let Some(node) = self.get_node(row, col) {
            unsafe { node.as_ref().value.unwrap() }
        } else {
            T::default()
        }
    }

    fn get_node(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        unsafe {
            let mut node = self.rows_vec[row].as_ref().next_row;

            while node?.as_ref().col < col {
                node = node?.as_ref().next_row
            }

            if node?.as_ref().col == col {
                node
            } else {
                None
            }
        }
    }
}

impl<T> Default for SparseMatrix<T>
where
    T: Default + Copy + PartialEq,
{
    fn default() -> Self {
        Self::new(2, 2)
    }
}

impl<T> Debug for SparseMatrix<T>
where
    T: Default + Copy + PartialEq + Debug,
{
    //FIXME: Too slow
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.rows_vec.len() {
            for j in 0..self.cols_vec.len() {
                if let Some(node) = self.get_node(i, j) {
                    unsafe { write!(f, "{:?} ", node.as_ref().value.unwrap())? };
                } else {
                    write!(f, "{:?} ", T::default())?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
