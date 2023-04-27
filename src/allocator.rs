pub trait Allocator {
    fn allocate<T>(&mut self) -> Vec<T>;

    fn size(&self) -> usize;

    fn new() -> Self;
}