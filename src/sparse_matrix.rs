use crate::MaybeNone;
use num::traits::NumAssign;
use num::Num;
use std::fmt::{Debug, Formatter};
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
    T: Num,
{
    fn default() -> Self {
        Node {
            value: T::zero(),
            row: usize::MAX,
            col: usize::MAX,
            next_row: None,
            next_col: None,
        }
    }
}

pub struct SparseMatrix<T: NumAssign + Copy> {
    rows_vec: Vec<NonNull<Node<T>>>,
    cols_vec: Vec<NonNull<Node<T>>>,
    size: usize,
}

pub struct AxisIter<T>
where
    T: Num + Clone,
{
    head: MaybeNone<Node<T>>,
    axis: usize,
    len: usize,
    max_len: usize,
}

impl<T> Iterator for AxisIter<T>
where
    T: Num + Clone,
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
                T::zero()
            }
        });

        self.len -= 1;
        Some(elem.unwrap_or_else(T::zero))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

struct NodeAxisIter<T>
where
    T: Num + Clone,
{
    head: MaybeNone<Node<T>>,
    axis: usize,
    len: usize,
}

impl<T> Iterator for NodeAxisIter<T>
where
    T: Num + Clone,
{
    type Item = NonNull<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.head.map(|nd| unsafe {
            let node = nd.as_ref();
            let next = match self.axis {
                0 => node.next_col,
                1 => node.next_row,
                _ => panic!("axis can be only 0 or 1"),
            };

            self.len -= 1;
            self.head = next;
            nd
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> SparseMatrix<T>
where
    T: NumAssign + Copy,
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

    pub fn rows(&self) -> usize {
        self.rows_vec.len()
    }

    pub fn cols(&self) -> usize {
        self.cols_vec.len()
    }

    pub fn sparsity(&self) -> f64 {
        let max_size = (self.rows() * self.cols()) as f64;
        (max_size - self.size as f64) / max_size
    }

    pub fn from_2d_vec(vec: Vec<Vec<T>>) -> Self {
        let cols = vec.iter().map(|v| v.len()).max_by(|x, y| x.cmp(y)).unwrap();
        let mut matrix = Self::new(vec.len(), cols);

        for (i, v) in vec.iter().enumerate() {
            let mut prev = Some(matrix.rows_vec[i]);

            for (j, value) in v.iter().enumerate().filter(|t| *t.1 != T::zero()) {
                let node = Box::new(Node::new(*value, i, j));
                let current = Some(Box::leak(node).into());

                unsafe { prev.unwrap().as_mut().next_row = current };
                prev = current;
                matrix.size += 1;
            }
        }

        let mut update = matrix.cols_vec.clone();
        for row in 0..matrix.rows() {
            for (col, upd) in update.iter_mut().enumerate() {
                if let Some(node) = matrix.get_node_row(row, col) {
                    unsafe { upd.as_mut().next_col = Some(node) };
                    *upd = node;
                }
            }
        }

        matrix
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut vec = Vec::new();
        for row in 0..self.rows() {
            let mut v: Vec<T> = self.row_iter(row).collect();
            other.row_iter(row).enumerate().for_each(|(i, e)| v[i] += e);
            vec.push(v);
        }
        SparseMatrix::from_2d_vec(vec)
    }

    pub fn mul_by(&self, num: T) -> Self {
        let clone = self.clone();
        for row in 0..clone.rows() {
            let iter = clone.node_row_iter(row);
            iter.for_each(|mut each| unsafe { each.as_mut().value *= num });
        }
        clone
    }

    pub fn transposed(&self) -> Self {
        let range = 0..self.cols();
        Self::from_2d_vec(range.map(|c| self.col_iter(c).collect()).collect())
    }

    pub fn set(&mut self, value: T, row: usize, col: usize) {
        unsafe {
            let mut prev_col = self.get_prev_node_col(row, col);
            let mut prev_row = self.get_prev_node_row(row, col);

            let value_is_zero = value == T::zero();
            let node_exists = if let Some(node) = prev_col.as_ref().next_col {
                node.as_ref().row == row
            } else {
                false
            };

            if node_exists {
                if value_is_zero {
                    Self::remove_node(prev_col, prev_row);
                } else {
                    prev_col.as_ref().next_col.unwrap().as_mut().value = value;
                }
            } else {
                let node = Box::new(Node::new(value, row, col));
                let ptr = Some(Box::leak(node).into());
                prev_row.as_mut().next_row = ptr;
                prev_col.as_mut().next_col = ptr;
            }
        }
    }

    unsafe fn remove_node(mut prev_col: NonNull<Node<T>>, mut prev_row: NonNull<Node<T>>) {
        let _ = Box::from_raw(prev_row.as_mut().next_row.unwrap().as_ptr());
        prev_row.as_mut().next_row = prev_row.as_mut().next_row.unwrap().as_mut().next_row;
        prev_col.as_mut().next_col = prev_col.as_mut().next_col.unwrap().as_mut().next_col;
    }

    fn get_prev_node_col(&self, row: usize, col: usize) -> NonNull<Node<T>> {
        let mut prev = None;
        for each in self.node_col_iter(col) {
            if unsafe { each.as_ref().row } < row {
                prev = Some(each);
            } else {
                break;
            }
        }
        prev.unwrap_or(self.cols_vec[col])
    }

    fn get_prev_node_row(&self, row: usize, col: usize) -> NonNull<Node<T>> {
        let mut prev = None;
        for each in self.node_row_iter(row) {
            if unsafe { each.as_ref().col } < col {
                prev = Some(each);
            } else {
                break;
            }
        }
        prev.unwrap_or(self.rows_vec[row])
    }

    pub fn get(&self, row: usize, col: usize) -> T {
        if let Some(node) = self.get_node(row, col) {
            unsafe { node.as_ref().value }
        } else {
            T::zero()
        }
    }

    pub fn row_iter(&self, row: usize) -> AxisIter<T> {
        AxisIter {
            head: unsafe { self.rows_vec[row].as_ref().next_row },
            axis: 1,
            len: self.cols(),
            max_len: self.cols(),
        }
    }

    pub fn col_iter(&self, col: usize) -> AxisIter<T> {
        AxisIter {
            head: unsafe { self.cols_vec[col].as_ref().next_col },
            axis: 0,
            len: self.rows(),
            max_len: self.rows(),
        }
    }

    fn node_row_iter(&self, row: usize) -> NodeAxisIter<T> {
        NodeAxisIter {
            head: unsafe { self.rows_vec[row].as_ref().next_row },
            axis: 1,
            len: self.rows(),
        }
    }

    fn node_col_iter(&self, col: usize) -> NodeAxisIter<T> {
        NodeAxisIter {
            head: unsafe { self.cols_vec[col].as_ref().next_col },
            axis: 0,
            len: self.rows(),
        }
    }

    fn get_node(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        if row > col {
            self.get_node_row(row, col)
        } else {
            self.get_node_col(row, col)
        }
    }

    fn get_node_row(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        unsafe {
            let node = self.get_prev_node_row(row, col).as_ref().next_row;
            if node?.as_ref().col == col {
                node
            } else {
                None
            }
        }
    }

    fn get_node_col(&self, row: usize, col: usize) -> MaybeNone<Node<T>> {
        unsafe {
            let node = self.get_prev_node_col(row, col).as_ref().next_col;
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
    T: NumAssign + Copy,
{
    fn default() -> Self {
        Self::new(2, 2)
    }
}

impl<T> Debug for SparseMatrix<T>
where
    T: NumAssign + Copy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        writeln!(f, "{:?},", self.row_iter(0).collect::<Vec<T>>())?;

        for i in 1..(self.rows() - 1) {
            writeln!(f, " {:?},", self.row_iter(i).collect::<Vec<T>>())?;
        }

        writeln!(
            f,
            " {:?}]",
            self.row_iter(self.rows() - 1).collect::<Vec<T>>()
        )?;
        write!(f, "Shape: {}x{}  ", self.rows(), self.cols())?;
        writeln!(f, "Sparsity: {:.2}", self.sparsity())
    }
}

impl<T> Clone for SparseMatrix<T>
where
    T: NumAssign + Copy,
{
    fn clone(&self) -> Self {
        let range = 0..self.rows();
        Self::from_2d_vec(range.map(|r| self.row_iter(r).collect()).collect())
    }
}

impl<T> Drop for SparseMatrix<T>
where
    T: NumAssign + Copy,
{
    fn drop(&mut self) {
        unsafe {
            for row in 0..self.rows() {
                for each in self.node_row_iter(row) {
                    drop(Box::from_raw(each.as_ptr()))
                }
            }
            for each in self.cols_vec.iter() {
                drop(Box::from_raw(each.as_ptr()));
            }
            for each in self.rows_vec.iter() {
                drop(Box::from_raw(each.as_ptr()))
            }
        }
    }
}
