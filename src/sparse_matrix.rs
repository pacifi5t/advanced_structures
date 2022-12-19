use crate::MaybeNone;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Mul, MulAssign};
use std::ptr::NonNull;

#[derive(Clone)]
struct Node<T> {
    value: T,
    row: usize,
    col: usize,
    next_row: MaybeNone<Node<T>>,
    next_col: MaybeNone<Node<T>>,
}

impl<T> Node<T> {
    fn new(value: T, row: usize, col: usize) -> Self {
        Node {
            value,
            row,
            col,
            next_col: None,
            next_row: None,
        }
    }
}

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Node {
            value: T::default(),
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
    size: usize,
}

pub struct AxisIter<T>
where
    T: Default + Clone,
{
    head: MaybeNone<Node<T>>,
    axis: usize,
    len: usize,
    max_len: usize,
}

impl<T> Iterator for AxisIter<T>
where
    T: Default + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let elem = self.head.map(|node| unsafe {
            let node = node.as_ref();
            let (index, next) = match self.axis {
                0 => (node.row, node.next_col),
                1 => (node.col, node.next_row),
                _ => panic!("axis can be only 0 or 1"),
            };

            if index == self.max_len - self.len {
                self.head = next;
                node.value.clone()
            } else {
                T::default()
            }
        });

        self.len -= 1;
        Some(elem.unwrap_or_default())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> SparseMatrix<T>
where
    T: Default + Copy + PartialEq,
{
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows_vec: Self::create_empty_nodes_vec(rows),
            cols_vec: Self::create_empty_nodes_vec(cols),
            size: 0,
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

    pub fn sparsity(&self) -> f64 {
        let max_size = (self.rows_vec.len() * self.cols_vec.len()) as f64;
        (max_size - self.size as f64) / max_size
    }

    pub fn from_2d_vec(vec: Vec<Vec<T>>) -> Self {
        let cols = vec.iter().map(|v| v.len()).max_by(|x, y| x.cmp(y)).unwrap();
        let mut matrix = Self::new(vec.len(), cols);

        for (i, v) in vec.iter().enumerate() {
            let mut prev = Some(matrix.rows_vec[i]);

            for (j, value) in v.iter().enumerate().filter(|t| *t.1 != T::default()) {
                let node = Box::new(Node::new(*value, i, j));
                let current = Some(Box::leak(node).into());

                unsafe { prev.unwrap().as_mut().next_row = current };
                prev = current;
                matrix.size += 1;
            }
        }

        let mut update = matrix.cols_vec.clone();
        for i in 0..matrix.rows_vec.len() {
            for j in 0..update.len() {
                if let Some(node) = matrix.get_node_rows(i, j) {
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
            unsafe { node.as_ref().value }
        } else {
            T::default()
        }
    }

    pub fn row_iter(&self, row: usize) -> AxisIter<T> {
        AxisIter {
            head: unsafe { self.rows_vec[row].as_ref().next_row },
            axis: 1,
            len: self.cols_vec.len(),
            max_len: self.cols_vec.len(),
        }
    }

    pub fn col_iter(&self, col: usize) -> AxisIter<T> {
        AxisIter {
            head: unsafe { self.cols_vec[col].as_ref().next_col },
            axis: 0,
            len: self.rows_vec.len(),
            max_len: self.rows_vec.len(),
        }
    }

    fn get_node(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        if row > col {
            self.get_node_rows(row, col)
        } else {
            self.get_node_cols(row, col)
        }
    }

    fn get_node_rows(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
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

    fn get_node_cols(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        unsafe {
            let mut node = self.cols_vec[col].as_ref().next_col;

            while node?.as_ref().row < row {
                node = node?.as_ref().next_col
            }

            if node?.as_ref().row == row {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let shape = (self.rows_vec.len(), self.cols_vec.len());

        write!(f, "[")?;
        writeln!(f, "{:?},", self.row_iter(0).collect::<Vec<T>>())?;

        for i in 1..(shape.0 - 1) {
            writeln!(f, " {:?},", self.row_iter(i).collect::<Vec<T>>())?;
        }

        writeln!(f, " {:?}]", self.row_iter(shape.0 - 1).collect::<Vec<T>>())?;
        write!(f, "Shape: {}x{}  ", shape.0, shape.1)?;
        writeln!(f, "Sparsity: {:.2}", self.sparsity())
    }
}

impl<T> Clone for SparseMatrix<T>
where
    T: Default + Copy + PartialEq,
{
    fn clone(&self) -> Self {
        let mut vec = Vec::new();
        for row in 0..self.rows_vec.len() {
            vec.push(self.row_iter(row).collect())
        }
        SparseMatrix::from_2d_vec(vec)
    }
}

impl<T> Mul<T> for SparseMatrix<T>
where
    T: Default + Copy + PartialEq + MulAssign,
{
    type Output = Self;

    //TODO: Can be optimized with node iterator
    fn mul(self, rhs: T) -> Self::Output {
        for row in 0..self.rows_vec.len() {
            for col in 0..self.cols_vec.len() {
                if let Some(mut node) = self.get_node(row, col) {
                    unsafe { node.as_mut().value *= rhs };
                }
            }
        }

        self
    }
}

impl<T> Add for SparseMatrix<T>
where
    T: Default + Copy + PartialEq + Add<Output = T>,
{
    type Output = Self;

    //TODO: Can be done cleaner
    fn add(self, rhs: Self) -> Self::Output {
        let mut vec = Vec::new();
        for row in 0..self.rows_vec.len() {
            let mut v: Vec<T> = Vec::new();
            for col in 0..self.cols_vec.len() {
                let l = if let Some(node) = self.get_node(row, col) {
                    unsafe { node.as_ref().value }
                } else {
                    T::default()
                };

                let r = if let Some(node) = rhs.get_node(row, col) {
                    unsafe { node.as_ref().value }
                } else {
                    T::default()
                };

                v.push(l + r);
            }
            vec.push(v);
        }

        SparseMatrix::from_2d_vec(vec)
    }
}

//TODO: DESTRUCTOR
