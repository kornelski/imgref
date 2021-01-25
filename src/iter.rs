use core::num::NonZeroUsize;
use core::iter::FusedIterator;
use std::marker::PhantomData;
use std::slice;

/// Rows of the image. Call `Img.rows()` to create it.
///
/// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
#[derive(Debug)]
#[must_use]
pub struct RowsIter<'a, T> {
    pub(crate) inner: slice::Chunks<'a, T>,
    pub(crate) width: usize,
}

impl<'a, T: 'a> Iterator for RowsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(s) => {
                // guaranteed during creation of chunks iterator
                debug_assert!(s.len() >= self.width);
                unsafe {
                    Some(s.get_unchecked(0..self.width))
                }
            },
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
            Some(s) => {
                // guaranteed during creation of chunks iterator
                debug_assert!(s.len() >= self.width);
                unsafe {
                    Some(s.get_unchecked(0..self.width))
                }
            },
            None => None,
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<'a, T> ExactSizeIterator for RowsIter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, T> FusedIterator for RowsIter<'a, T> {}

impl<'a, T: 'a> DoubleEndedIterator for RowsIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner.next_back() {
            Some(s) => {
                // guaranteed during creation of chunks iterator
                debug_assert!(s.len() >= self.width);
                unsafe {
                    Some(s.get_unchecked(0..self.width))
                }
            },
            None => None,
        }
    }
}

/// Rows of the image. Call `Img.rows_mut()` to create it.
///
/// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
#[derive(Debug)]
#[must_use]
pub struct RowsIterMut<'a, T> {
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
impl<'a, T> FusedIterator for RowsIterMut<'a, T> {}

impl<'a, T: 'a> DoubleEndedIterator for RowsIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.inner.next_back() {
            Some(s) => Some(&mut s[0..self.width]),
            None => None,
        }
    }
}


/// Iterates over pixels in the (sub)image. Call `Img.pixels()` to create it.
///
/// Ignores padding, if there's any.
#[derive(Debug)]
#[must_use]
pub struct PixelsIter<'a, T: Copy> {
    current: *const T,
    current_line_end: *const T,
    rows_left: usize,
    width: NonZeroUsize,
    pad: usize,
    _dat: PhantomData<&'a [T]>,
}

impl<'a, T: Copy + 'a> PixelsIter<'a, T> {
    #[inline]
    pub(crate) fn new(img: super::ImgRef<'a, T>) -> Self {
        let width = NonZeroUsize::new(img.width()).expect("width > 0");
        let stride = img.stride();
        debug_assert!(img.buf().len() + stride >= stride * img.height() + width.get());
        Self {
            current: img.buf().as_ptr(),
            current_line_end: img.buf()[width.get()..].as_ptr(),
            width,
            rows_left: img.height(),
            pad: stride - width.get(),
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
                if self.rows_left <= 1 {
                    return None;
                }
                self.rows_left -= 1;
                self.current = self.current_line_end.add(self.pad);
                self.current_line_end = self.current.add(self.width.get());
            }
            let px = *self.current;
            self.current = self.current.add(1);
            Some(px)
        }
    }
}

/// Iterates over pixels in the (sub)image. Call `Img.pixels_mut()` to create it.
///
/// Ignores padding, if there's any.
#[derive(Debug)]
#[must_use]
pub struct PixelsIterMut<'a, T: Copy> {
    current: *mut T,
    current_line_end: *mut T,
    y: usize,
    width: NonZeroUsize,
    pad: usize,
    _dat: PhantomData<&'a mut [T]>,
}

impl<'a, T: Copy + 'a> PixelsIterMut<'a, T> {
    #[inline]
    pub(crate) fn new(img: &mut super::ImgRefMut<'a, T>) -> Self {
        let width = NonZeroUsize::new(img.width()).expect("width > 0");
        let stride = img.stride();
        debug_assert!(!img.buf().is_empty() && img.buf().len() + stride >= stride * img.height() + width.get());
        Self {
            current: img.buf_mut().as_mut_ptr(),
            current_line_end: img.buf_mut()[width.get()..].as_mut_ptr(),
            width,
            y: img.height(),
            pad: stride - width.get(),
            _dat: PhantomData,
        }
    }
}

impl<'a, T: Copy + 'a> Iterator for PixelsIterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current >= self.current_line_end {
                self.y -= 1;
                if self.y == 0 {
                    return None;
                }
                self.current = self.current_line_end.add(self.pad);
                self.current_line_end = self.current.add(self.width.get());
            }
            let px = &mut *self.current;
            self.current = self.current.add(1);
            Some(px)
        }
    }
}

#[test]
fn iter() {
    let img = super::Img::new(vec![1u8, 2], 1, 2);
    let mut it = img.pixels();
    assert_eq!(Some(1), it.next());
    assert_eq!(Some(2), it.next());
    assert_eq!(None, it.next());

    let buf = vec![1u8; (16 + 3) * (8 + 1)];
    for width in 1..16 {
        for height in 1..8 {
            for pad in 0..3 {
                let img = super::Img::new_stride(&buf[..], width, height, width + pad);
                assert_eq!(width * height, img.pixels().map(|a| a as usize).sum(), "{}x{}", width, height);
                assert_eq!(width * height, img.pixels().count(), "{}x{}", width, height);
                assert_eq!(height, img.rows().count());

                let mut iter1 = img.pixels();
                match iter1.next() {
                    Some(_) => assert_eq!(width * height - 1, iter1.filter(|_| true).count()),
                    None => assert_eq!(width * height, 0),
                };

                let mut iter2 = img.rows();
                match iter2.next() {
                    Some(_) => {
                        assert_eq!(height - 1, iter2.size_hint().0);
                        assert_eq!(height - 1, iter2.filter(|_| true).count());
                    },
                    None => {
                        assert_eq!(height, 0);
                    },
                };
            }
        }
    }
}
