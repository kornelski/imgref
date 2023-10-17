//! Implementations of iterators for `.rows()`, `.items()`, etc.

use crate::size::Size;
use std::marker::PhantomData;
use std::ptr::NonNull;
use core::iter::FusedIterator;

pub use core::iter::Copied;

/// Iterator over items in non-contiguous 2D slices
#[repr(transparent)]
pub struct StrideItemIter<'a, T> {
    iter: ItemIterInner<T>,
    _data_lifetime: PhantomData<&'a [T]>,
}

impl<'a, T> StrideItemIter<'a, T> {
    #[inline]
    pub(crate) unsafe fn new<S: Size>(width: S, height: S, stride_bytes: usize, data: NonNull<T>) -> Self {
        Self {
            iter: ItemIterInner::new(width, stride_bytes, height, data),
            _data_lifetime: PhantomData,
        }
    }
}

/// Iterator over mutable items in non-contiguous 2D slices
#[repr(transparent)]
pub struct StrideItemIterMut<'a, T> {
    iter: ItemIterInner<T>,
    _data_lifetime: PhantomData<&'a mut [T]>,
}

impl<'a, T> StrideItemIterMut<'a, T> {
    #[inline]
    pub(crate) unsafe fn new<S: Size>(width: S, stride_bytes: usize, height: S, data: NonNull<T>) -> Self {
        Self {
            iter: ItemIterInner::new(width, stride_bytes, height, data),
            _data_lifetime: PhantomData,
        }
    }
}

struct ItemIterInner<T> {
    data: NonNull<T>,
    /// pointer to int cast
    row_end_ptr: IntPtr,
    strides_end_ptr: IntPtr,
    stride_minus_width_bytes: usize,
    stride_bytes: usize,
}

impl<T> ItemIterInner<T> {
    #[inline]
    pub fn next(&mut self) -> Option<*mut T> {
        unsafe {
            if (self.data.as_ptr() as IntPtr) < self.row_end_ptr {
                let res = self.data.as_ptr();
                self.data = NonNull::new_unchecked(res.add(1));
                return Some(res)
            }
            self.row_end_ptr += self.stride_bytes;
            if self.row_end_ptr > self.strides_end_ptr { // when equal, it's the last row
                return None;
            }
            self.data = NonNull::new_unchecked(self.data.as_ptr().byte_add(self.stride_minus_width_bytes));
            let res = self.data.as_ptr();
            self.data = NonNull::new_unchecked(res.add(1));
            Some(res)
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if (self.data.as_ptr() as IntPtr) >= self.strides_end_ptr {
            return 0;
        }

        // can it be negative?
        let left_in_this_row = (self.row_end_ptr as isize - self.data.as_ptr() as isize).max(0) as usize;
        let rows = (self.strides_end_ptr + self.stride_minus_width_bytes - self.row_end_ptr) / self.stride_bytes;
        let width_bytes = self.stride_bytes - self.stride_minus_width_bytes;
        left_in_this_row + rows * (width_bytes/std::mem::size_of::<T>())
    }
}

impl<T> ItemIterInner<T> {
    #[inline]
    pub unsafe fn new<S: Size>(width: S, stride_bytes: usize, height: S, data: NonNull<T>) -> Self {
        // TODO: should this fail? make dummy iter? fix the width?
        assert!(width.mul_size_of::<T>().unwrap() <= stride_bytes);

        debug_assert!(height.usize().checked_mul(stride_bytes).is_some());
        let width_bytes = width.mul_size_of::<T>().unwrap();
        Self {
            stride_minus_width_bytes: stride_bytes - width_bytes,
            stride_bytes,
            data,
            row_end_ptr: if height > S::ZERO { data.as_ptr() as IntPtr + width.usize() } else { data.as_ptr() as IntPtr },
            // TODO: this could overflow?
            strides_end_ptr: data.as_ptr() as IntPtr + (height.usize() * stride_bytes),
        }
    }
}

impl<'a, T> Iterator for StrideItemIter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Some(&*self.iter.next()?) }
    }
}

impl<T> ExactSizeIterator for StrideItemIter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T> Iterator for StrideItemIterMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Some(&mut *self.iter.next()?) }
    }
}

impl<T> ExactSizeIterator for StrideItemIterMut<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

type IntPtr = usize;

/// Rows of the image. Call `Img.rows()` to create it.
///
/// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
#[derive(Debug)]
#[must_use]
pub struct StrideRowsIter<'slice, T> {
    pub(crate) width: usize,
    pub(crate) stride_bytes: usize,
    pub(crate) strides_end_ptr: IntPtr,
    pub(crate) data_start: NonNull<T>,
    pub(crate) ownership: PhantomData<&'slice [T]>,
}

impl<'slice, T> StrideRowsIter<'slice, T> {
    /// Safety: width <= stride && stride > 0
    #[inline]
    pub(crate) unsafe fn new<S: Size>(width: S, height: S, stride_bytes: usize, data: NonNull<T>) -> Self {
        debug_assert!(width.mul_size_of::<T>().unwrap() <= stride_bytes);
        Self {
            width: width.usize(),
            stride_bytes,
            strides_end_ptr: data.as_ptr() as IntPtr + (height.usize() * stride_bytes),
            data_start: data,
            ownership: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for StrideRowsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if (self.data_start.as_ptr() as IntPtr) < self.strides_end_ptr {
            unsafe {
                let res = std::slice::from_raw_parts(self.data_start.as_ptr(), self.width);
                self.data_start = NonNull::new_unchecked(self.data_start.as_ptr().byte_add(self.stride_bytes));
                Some(res)
            }
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let i = self.len();
        (i, Some(i))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            let offset_bytes = n.checked_mul(self.stride_bytes)?;
            // can add overflow?
            self.data_start = NonNull::new_unchecked(self.data_start.as_ptr().byte_add(offset_bytes));
        }
        self.next()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl<'a, T> ExactSizeIterator for StrideRowsIter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        let bytes_left = (self.strides_end_ptr as isize - self.data_start.as_ptr() as isize).max(0) as usize;
        bytes_left / self.stride_bytes

    }
}

impl<'a, T> FusedIterator for StrideRowsIter<'a, T> {}

impl<'a, T: 'a> DoubleEndedIterator for StrideRowsIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.strides_end_ptr > self.data_start.as_ptr() as IntPtr {
            let last_row_start_offset = self.strides_end_ptr - self.stride_bytes;
            self.strides_end_ptr -= self.stride_bytes;
            unsafe {
                Some(std::slice::from_raw_parts(self.data_start.as_ptr().byte_add(last_row_start_offset), self.width))
            }
        } else {
            None
        }
    }
}


// /// Rows of the image. Call `Img.rows_mut()` to create it.
// ///
// /// Each element is a slice `width` pixels wide. Ignores padding, if there's any.
// #[derive(Debug)]
// #[must_use]
// pub struct StrideRowsIterMut<'slice, T> {
//     width: usize,
//     inner: slice::ChunksMut<'slice, T>,
// }

// // impl<'slice, T> RowsIterMut<'slice, T> {
// //     pub fn new(width: usize, stride: usize, buf: &'slice mut [T]) -> Self {
// //         Self {
// //             width, inner: buf.chunks_mut(stride)
// //         }
// //     }
// // }

// impl<'slice, T: 'slice> Iterator for StrideRowsIterMut<'slice, T> {
//     type Item = &'slice mut [T];

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.inner.next() {
//             Some(s) => Some(&mut s[0..self.width]),
//             None => None,
//         }
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }

//     #[inline]
//     fn nth(&mut self, n: usize) -> Option<Self::Item> {
//         match self.inner.nth(n) {
//             Some(s) => Some(&mut s[0..self.width]),
//             None => None,
//         }
//     }

//     #[inline]
//     fn count(self) -> usize {
//         self.inner.count()
//     }
// }

// impl<'a, T> ExactSizeIterator for StrideRowsIterMut<'a, T> {}
// impl<'a, T> FusedIterator for StrideRowsIterMut<'a, T> {}

// impl<'a, T: 'a> DoubleEndedIterator for StrideRowsIterMut<'a, T> {
//     #[inline]
//     fn next_back(&mut self) -> Option<Self::Item> {
//         match self.inner.next_back() {
//             Some(s) => Some(&mut s[0..self.width]),
//             None => None,
//         }
//     }
// }

