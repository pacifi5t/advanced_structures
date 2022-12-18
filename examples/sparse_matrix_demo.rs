use advanced_structures::sparse_matrix::SparseMatrix;

fn main() {
    let a = vec![
        vec![1, 2],
        vec![3],
        vec![0, 0, 6],
    ];

    let m = SparseMatrix::from_2d_vec(a);
    print!("{:?}", m);
}
