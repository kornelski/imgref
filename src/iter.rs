use std::slice;
use std::marker::PhantomData;

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.inner.nth(n) {
            Some(s) => Some(&s[0..self.width]),
            None => None,
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<'a, T> ExactSizeIterator for RowsIter<'a, T> {}

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.inner.nth(n) {
            Some(s) => Some(&mut s[0..self.width]),
            None => None,
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<'a, T> ExactSizeIterator for RowsIterMut<'a, T> {}

/// Iterates over pixels in the (sub)image. Call `Img.pixels()` to create it.
///
/// Ignores padding, if there's any.
pub struct PixelsIter<'a, T: Copy + 'a> {
    current: *const T,
    current_line_end: *const T,
    y: usize,
    width: usize,
    pad: usize,
    _dat: PhantomData<&'a [T]>,
}

impl<'a, T: Copy + 'a> PixelsIter<'a, T> {
    pub(crate) fn new(img: super::ImgRef<'a, T>) -> Self {
        let width = img.width();
        let stride = img.stride();
        debug_assert!(img.buf.len() > 0 && img.buf.len() >= stride * img.height() + width - stride);
        Self {
           current: img.buf[0..].as_ptr(),
           current_line_end: img.buf[width..].as_ptr(),
           width,
           y: img.height(),
           pad: stride - width,
           _dat: PhantomData,
       }
    }
}

impl<'a, T: Copy + 'a> Iterator for PixelsIter<'a, T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current >= self.current_line_end {
                self.y -= 1;
                if self.y == 0 {
                    return None;
                }
                self.current = self.current_line_end.offset(self.pad as isize);
                self.current_line_end = self.current.offset(self.width as isize);
            }
            let px = *self.current;
            self.current = self.current.offset(1);
            Some(px)
        }
    }
}

#[test]
fn iter() {
    let img = super::Img::new(vec![1u8,2], 1,2);
    let mut it = img.pixels();
    assert_eq!(Some(1), it.next());
    assert_eq!(Some(2), it.next());
    assert_eq!(None, it.next());

    let buf = vec![1u8; (16+3)*(8+1)];
    for width in 1..16 {
        for height in 1..8 {
            for pad in 0..3 {
                let img = super::Img::new_stride(&buf[..], width, height, width+pad);
                assert_eq!(width*height, img.pixels().count());
                assert_eq!(height, img.rows().count());
                assert_eq!(width*height, img.pixels().map(|a| a as usize).sum());

                let mut iter1 = img.pixels();
                iter1.next();
                assert_eq!(width*height - 1, iter1.filter(|_| true).count());

                let mut iter2 = img.rows();
                iter2.next();
                assert_eq!(height - 1, iter2.size_hint().0);
                assert_eq!(height - 1, iter2.filter(|_| true).count());
            }
        }
    }
}
