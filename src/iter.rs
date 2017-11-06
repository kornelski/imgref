use std::slice;

/// Rows of the image. Call `Img.rows()` to create it.
///
/// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
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

/// Rows of the image. Call `Img.rows_mut()` to create it.
///
/// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
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

/// Iterates over pixels in the (sub)image. Call `Img.pixels()` to create it.
///
/// Ignores padding, if there's any.
pub struct PixelsIter<'a, T: Copy + 'a> {
    buf_left: &'a [T],
    current_row: &'a [T],
    width: usize,
    stride: usize,
}

impl<'a, T: Copy + 'a> PixelsIter<'a, T> {
    pub(crate) fn new(img: super::ImgRef<'a, T>) -> Self {
        let end = img.stride() * img.height() + img.width() - img.stride();
        Self {
           buf_left: &img.buf[0..end],
           current_row: &img.buf[0..0],
           stride: img.stride(),
           width: img.width(),
       }
    }
}

impl<'a, T: Copy + 'a> Iterator for PixelsIter<'a, T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row.is_empty() {
            if self.buf_left.len() < self.width {
                return None;
            }
            self.current_row = &self.buf_left[..self.width];
            self.buf_left = &self.buf_left[self.stride.min(self.buf_left.len()-1)..];
        }
        let px = self.current_row[0];
        self.current_row = &self.current_row[1..];
        Some(px)
    }
}
