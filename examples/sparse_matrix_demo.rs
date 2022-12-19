use advanced_structures::sparse_matrix::SparseMatrix;

fn main() {
    let m = SparseMatrix::from_2d_vec(vec![
        vec![1, 2, 4],
        vec![30],
        vec![0, 0, 0],
        vec![0, 1],
    ]);

    let m2 = SparseMatrix::from_2d_vec(vec![
        vec![10],
        vec![0, 0, 1],
        vec![20, 40],
        vec![9, 0, 21],
    ]);

    println!("{:?}", m.mul_by(3));
    println!("{:?}", m.add(&m2));

}
