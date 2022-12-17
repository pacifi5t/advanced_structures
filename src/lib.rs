use std::ptr::NonNull;

pub mod lists;

type MaybeNone<T> = Option<NonNull<T>>;
