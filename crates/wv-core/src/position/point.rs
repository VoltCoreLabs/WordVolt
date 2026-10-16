/// A point in the document: paragraph index + UTF-8 byte offset inside it.
/// Ordering is document order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub para: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(para: usize, offset: usize) -> Self {
        Self { para, offset }
    }
}
