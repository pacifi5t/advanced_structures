use std::ptr::NonNull;

pub mod lists;
pub mod sparse_matrix;

type MaybeNone<T> = Option<NonNull<T>>;
