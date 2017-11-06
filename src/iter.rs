use std::slice;

pub struct RowsIter<'a, T: 'a> {
    pub(crate) width: usize,
    pub(crate) inner: slice::Chunks<'a, T>,
}

impl<'a, T: 'a> Iterator for RowsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(s) => Some(&s[0..self.width]),
            None => None,
        }
    }
}

pub struct RowsIterMut<'a, T: 'a> {
    pub(crate) width: usize,
    pub(crate) inner: slice::ChunksMut<'a, T>,
}

impl<'a, T: 'a> Iterator for RowsIterMut<'a, T> {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(s) => Some(&mut s[0..self.width]),
            None => None,
        }
    }
}
