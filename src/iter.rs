use alloc::boxed::Box;

pub trait BoxedIter {
    type Item;
    fn boxed(self) -> Box<dyn Iterator<Item = Self::Item>>;
}

impl<I: Iterator<Item = T> + 'static, T> BoxedIter for I {
    type Item = T;
    fn boxed(self) -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(self)
    }
}
