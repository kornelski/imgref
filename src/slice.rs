use crate::Container;
use crate::iter;
use crate::Img;
use crate::ImgMut;
use core::ops::Index;
use core::ops::RangeBounds;
use core::ops::Deref;
use core::ptr::NonNull;
use core::slice::ChunksExact;
use std::mem::MaybeUninit;
use crate::size::Size;
use core::slice;
use std::borrow::Cow;

mod inner;

/// A contiguous 2D slice.
///
/// `width * height` large, with items laid out consecutively row by row.
///
/// The dimensions are generic, and can be u32, u16, or u8 for tiny slices.
pub struct Slice2D<C: Container, Usize = usize> {
    s: inner::RawSlice<C, Usize>,
}

/// A 2D slice of mutable memory.
///
/// Mutable equivalent of `Slice2D`.
#[repr(transparent)]
pub struct Slice2DMut<C: Container, Usize = usize> {
    s: inner::RawSlice<C, Usize>,
}

/// A 2D slice where memory can be non-contiguous.
///
/// It can represent a rectangle cut out from a larger slice.
#[repr(transparent)]
pub struct Stride2D<C: Container, Usize = usize> {
    s: inner::RawStride<C, Usize>,
}

/// A mutable 2D slice where memory can be non-contiguous.
///
/// It can represent a rectangle cut out from a larger slice.
#[repr(transparent)]
pub struct Stride2DMut<C: Container, Usize = usize> {
    s: inner::RawStride<C, Usize>,
}

impl<C: Container, Usize: Size> Img<C, Usize> for Slice2D<C, Usize> {
    type ItemIter<'data> = slice::Iter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIter<'data> = ChunksExact<'data, C::Item> where Self: 'data, C::Item: 'data;
    type Stride<'data> = Stride2D<C::Borrowed<'data>, Usize> where Self: 'data;
    type Slice<'data> = Slice2D<C::Borrowed<'data>, Usize> where Self: 'data;

    #[inline]
    fn width(&self) -> usize {
        self.s.width.usize()
    }

    #[inline]
    fn height(&self) -> usize {
        self.s.height.usize()
    }

    #[inline]
    fn area(&self) -> usize {
        // TODO: should it guarantee no overflow?
        self.s.width.usize().checked_mul(self.s.height.usize()).unwrap()
    }

    #[inline]
    fn iter(&self) -> Self::ItemIter<'_> {
        self.buf().iter()
    }

    #[inline]
    fn rows(&self) -> Self::RowsIter<'_> {
        let w = self.width();
        if w > 0 {
            self.buf().chunks_exact(w)
        } else {
            [].chunks_exact(1)
        }
    }

    #[inline(always)]
    fn row(&self, row_index: usize) -> Option<&[C::Item]> {
        if row_index < self.s.height.usize() {
            Some(unsafe {
                self.row_unchecked(row_index)
            })
        } else {
            None
        }
    }

    #[inline(always)]
    unsafe fn row_unchecked(&self, row_index: usize) -> &[C::Item] {
        debug_assert!(row_index < self.s.height.usize());
        let offset = row_index.checked_mul(self.s.width.usize()).unwrap_unchecked();
        std::slice::from_raw_parts(C::ptr(&self.s.data).as_ptr().add(offset), self.s.width.usize())
    }

    #[inline]
    fn as_stride(&self) -> Self::Stride<'_> {
        unsafe {
            Self::Stride {
                s: self.s.borrowed().into(),
            }
        }
    }

    #[inline]
    fn split_at_col(&self, columns: Usize) -> Option<(Self::Stride<'_>, Self::Stride<'_>)> {
        let (c1, c2) = self.s.split_at_col(columns)?;
        Some((
            Self::Stride { s: c1 },
            Self::Stride { s: c2 },
        ))
    }

    #[inline]
    fn split_at_row(&self, rows: Usize) -> Option<(Self::Slice<'_>, Self::Slice<'_>)> {
        let (r1, r2) = self.s.split_at_row(rows)?;
        Some((
            Self::Slice { s: r1 },
            Self::Slice { s: r2 },
        ))
    }

    #[inline]
    fn is_contiguous(&self) -> bool {
        true
    }

    #[inline]
    fn to_contiguous_buf(&self) -> (Cow<'_, [C::Item]>, usize, usize) where [C::Item]: ToOwned {
        (Cow::Borrowed(self.buf()), self.width(), self.height())
    }

    #[inline]
    fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::Stride<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        let s = self.s.slice(horizontal, vertical)?;
        Some(Self::Stride { s })
    }
}

impl<C: Container, Usize: Size> Img<C, Usize> for Slice2DMut<C, Usize> {
    type ItemIter<'data> = slice::Iter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIter<'data> = ChunksExact<'data, C::Item> where Self: 'data, C::Item: 'data;
    type Stride<'data> = Stride2D<C::Borrowed<'data>, Usize> where Self: 'data;
    type Slice<'data> = Slice2D<C::Borrowed<'data>, Usize> where Self: 'data;

    #[inline]
    fn width(&self) -> usize {
        self.s.width.usize()
    }

    #[inline]
    fn height(&self) -> usize {
        self.s.width.usize()
    }

    #[inline]
    fn area(&self) -> usize {
        self.s.area()
    }

    #[inline]
    fn iter(&self) -> Self::ItemIter<'_> {
        self.buf().iter()
    }

    #[inline]
    fn rows(&self) -> Self::RowsIter<'_> {
        self.deref().rows()
    }

    #[inline]
    fn row(&self, row_index: usize) -> Option<&[C::Item]> {
        self.deref().row(row_index)
    }

    #[inline]
    unsafe fn row_unchecked(&self, row_index: usize) -> &[C::Item] {
        self.deref().row_unchecked(row_index)
    }

    #[inline]
    fn as_stride(&self) -> Self::Stride<'_> {
        self.deref().as_stride()
    }

    #[inline]
    fn split_at_col(&self, columns: Usize) -> Option<(Self::Stride<'_>, Self::Stride<'_>)> {
        self.deref().split_at_col(columns)
    }

    #[inline]
    fn split_at_row(&self, rows: Usize) -> Option<(Self::Slice<'_>, Self::Slice<'_>)> {
        self.deref().split_at_row(rows)
    }

    #[inline]
    fn is_contiguous(&self) -> bool {
        true
    }

    #[inline]
    fn to_contiguous_buf(&self) -> (Cow<'_, [C::Item]>, usize, usize) where [C::Item]: ToOwned {
        self.deref().to_contiguous_buf()
    }

    #[inline]
    fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::Stride<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        self.deref().slice(horizontal, vertical)
    }
}

impl<C: Container, Usize: Size> ImgMut<C, Usize> for Slice2DMut<C, Usize> {
    type ItemIterMut<'data> = slice::IterMut<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIterMut<'data> = slice::ChunksExactMut<'data, C::Item> where Self: 'data, C::Item: 'data;
    type SliceMut<'data> = Slice2DMut<C::Borrowed<'data>, Usize> where Self: 'data;
    type StrideMut<'data> = Stride2DMut<C::Borrowed<'data>, Usize> where Self: 'data;

    #[inline]
    fn iter_mut(&mut self) -> Self::ItemIterMut<'_> {
        self.buf_mut().iter_mut()
    }

    #[inline]
    fn rows_mut(&mut self) -> Self::RowsIterMut<'_> {
        let w = self.width();
        if w > 0 {
            self.buf_mut().chunks_exact_mut(w)
        } else {
            [].chunks_exact_mut(1)
        }
    }

    #[inline]
    fn row_mut(&mut self, row_index: usize) -> Option<&mut [C::Item]> {
        if row_index < self.s.height.usize() {
            Some(unsafe {
                let offset = row_index.checked_mul(self.s.width.usize()).unwrap_unchecked();
                std::slice::from_raw_parts_mut(C::ptr(&mut self.s.data).as_ptr().add(offset), self.s.width.usize())
            })
        } else {
            None
        }
    }

    #[inline]
    fn as_stride_mut(&mut self) -> Self::StrideMut<'_> {
        unsafe {
            Self::StrideMut {
                s: self.s.borrowed().into(),
            }
        }
    }

    #[inline]
    fn split_at_col_mut(&mut self, columns: Usize) -> Option<(Self::StrideMut<'_>, Self::StrideMut<'_>)> {
        let (c1, c2) = self.s.split_at_col(columns)?;
        Some((
            Self::StrideMut { s: c1 },
            Self::StrideMut { s: c2 },
        ))
    }

    #[inline]
    fn split_at_row_mut(&mut self, rows: Usize) -> Option<(Self::SliceMut<'_>, Self::SliceMut<'_>)> {
        let (r1, r2) = self.s.split_at_row(rows)?;
        Some((
            Self::SliceMut {s: r1, },
            Self::SliceMut {s: r2, },
        ))
    }

    #[inline]
    fn slice_mut<X1X2, Y1Y2>(&mut self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::StrideMut<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        let s = self.s.slice(horizontal, vertical)?;
        Some(Self::StrideMut { s})
    }
}

impl<C: Container, Usize: Size> Img<C, Usize> for Stride2D<C, Usize> {
    type ItemIter<'data> = iter::StrideItemIter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIter<'data> = iter::StrideRowsIter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type Slice<'data> = Stride2D<C::Borrowed<'data>, Usize> where Self: 'data;
    type Stride<'data> = Stride2D<C::Borrowed<'data>, Usize> where Self: 'data;

    fn width(&self) -> usize {
        self.s.width.usize()
    }

    fn height(&self) -> usize {
        self.s.height.usize()
    }

    fn area(&self) -> usize {
        // TODO: check for overflow?
        self.s.width.usize().checked_mul(self.s.height.usize()).unwrap()
    }

    fn iter(&self) -> Self::ItemIter<'_> {
        unsafe {
            Self::ItemIter::new(self.s.width, self.s.height, self.s.stride_bytes, C::ptr(&self.s.data))
        }
    }

    fn rows(&self) -> Self::RowsIter<'_> {
        unsafe {
            Self::RowsIter::new(self.s.width, self.s.height, self.s.stride_bytes, C::ptr(&self.s.data))
        }
    }

    #[inline(always)]
    fn row(&self, row_index: usize) -> Option<&[C::Item]> {
        if row_index < self.s.height.usize() {
            Some(unsafe { self.row_unchecked(row_index) })
        } else {
            None
        }
    }

    #[inline]
    unsafe fn row_unchecked(&self, row_index: usize) -> &[C::Item] {
        debug_assert!(row_index < self.s.height.usize());
        let offset_bytes = row_index.checked_mul(self.s.stride_bytes).unwrap_unchecked();
        std::slice::from_raw_parts(C::ptr(&self.s.data).as_ptr().byte_add(offset_bytes), self.s.width.usize())
    }

    fn as_stride(&self) -> Self::Stride<'_> {
        unsafe {
            Self::Stride {
                s: self.s.borrowed(),
            }
        }
    }

    #[inline]
    fn split_at_col(&self, columns: Usize) -> Option<(Self::Stride<'_>, Self::Stride<'_>)> {
        let (c1, c2) = self.s.split_at_col(columns)?;
        Some((
            Self::Stride { s: c1 },
            Self::Stride { s: c2 },
        ))
    }

    #[inline]
    fn split_at_row(&self, rows: Usize) -> Option<(Self::Slice<'_>, Self::Slice<'_>)> {
        let (r1, r2) = self.s.split_at_row(rows)?;
        Some((
            Self::Stride { s: r1 },
            Self::Stride { s: r2 },
        ))
    }

    #[inline]
    fn is_contiguous(&self) -> bool {
        self.s.width.mul_size_of::<C::Item>().unwrap() == self.s.stride_bytes
    }

    fn to_contiguous_buf(&self) -> (Cow<'_, [C::Item]>, usize, usize) where [C::Item]: ToOwned {
        todo!()
    }

    #[inline]
    fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::Stride<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        let s = self.s.slice(horizontal, vertical)?;
        Some(Self::Stride { s })
    }
}

impl<C: Container, Usize: Size> Img<C, Usize> for Stride2DMut<C, Usize> {
    type ItemIter<'data> = iter::StrideItemIter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIter<'data> = iter::StrideRowsIter<'data, C::Item> where Self: 'data, C::Item: 'data;
    type Slice<'data> = Stride2DMut<C::Borrowed<'data>, Usize> where Self: 'data;
    type Stride<'data> = Stride2DMut<C::Borrowed<'data>, Usize> where Self: 'data;

    fn width(&self) -> usize {
        self.s.width.usize()
    }

    fn height(&self) -> usize {
        self.s.height.usize()
    }

    fn area(&self) -> usize {
        // TODO: overflow?
        self.s.width.usize().checked_mul(self.s.height.usize()).unwrap()
    }

    fn iter(&self) -> Self::ItemIter<'_> {
        unsafe {
            Self::ItemIter::new(self.s.width, self.s.height, self.s.stride_bytes, C::ptr(&self.s.data))
        }
    }

    fn rows(&self) -> Self::RowsIter<'_> {
        unsafe {
            Self::RowsIter::new(self.s.width, self.s.height, self.s.stride_bytes, C::ptr(&self.s.data))
        }
    }

    #[inline(always)]
    fn row(&self, row_index: usize) -> Option<&[C::Item]> {
        if row_index < self.s.height.usize() {
            Some(unsafe { self.row_unchecked(row_index) })
        } else {
            None
        }
    }

    #[inline]
    unsafe fn row_unchecked(&self, row_index: usize) -> &[C::Item] {
        debug_assert!(row_index < self.s.height.usize());
        let offset_bytes = row_index.checked_mul(self.s.stride_bytes).unwrap_unchecked();
        std::slice::from_raw_parts(C::ptr(&self.s.data).as_ptr().byte_add(offset_bytes), self.s.width.usize())
    }

    fn as_stride(&self) -> Self::Stride<'_> {
        unsafe {
            Self::Stride {
                s: self.s.borrowed(),
            }
        }
    }

    #[inline]
    fn split_at_col(&self, columns: Usize) -> Option<(Self::Stride<'_>, Self::Stride<'_>)> {
        let (c1, c2) = self.s.split_at_col(columns)?;
        Some((
            Self::Stride { s: c1 },
            Self::Stride { s: c2 },
        ))
    }

    #[inline]
    fn split_at_row(&self, rows: Usize) -> Option<(Self::Slice<'_>, Self::Slice<'_>)> {
        let (r1, r2) = self.s.split_at_row(rows)?;
        Some((
            Self::Stride { s: r1 },
            Self::Stride { s: r2 },
        ))
    }

    #[inline]
    fn is_contiguous(&self) -> bool {
        self.s.width.mul_size_of::<C::Item>().unwrap() == self.s.stride_bytes
    }

    fn to_contiguous_buf(&self) -> (Cow<'_, [C::Item]>, usize, usize) where [C::Item]: ToOwned {
        todo!()
    }

    #[inline]
    fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::Stride<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        let s = self.s.slice(horizontal, vertical)?;
        Some(Self::Stride { s })
    }
}

impl<C: Container, Usize: Size> ImgMut<C, Usize> for Stride2DMut<C, Usize> {
    type ItemIterMut<'data> = iter::StrideItemIterMut<'data, C::Item> where Self: 'data, C::Item: 'data;
    type RowsIterMut<'data> = slice::ChunksExactMut<'data, C::Item> where Self: 'data, C::Item: 'data;
    type SliceMut<'data> = Stride2DMut<C::Borrowed<'data>, Usize> where Self: 'data;
    type StrideMut<'data> = Stride2DMut<C::Borrowed<'data>, Usize> where Self: 'data;

    #[inline]
    #[doc(alias = "items_mut")]
    fn iter_mut(&mut self) -> Self::ItemIterMut<'_> {
        // SAFETY: this must be a valid slice
        unsafe { iter::StrideItemIterMut::new(self.s.width, self.s.stride_bytes, self.s.height, C::ptr(&self.s.data)) }
    }

    #[inline]
    fn rows_mut(&mut self) -> Self::RowsIterMut<'_> {
        todo!()
    }

    #[inline]
    fn row_mut(&mut self, row_index: usize) -> Option<&mut [C::Item]> {
        if row_index < self.s.height.usize() {
            Some(unsafe {
                let offset_bytes = row_index.checked_mul(self.s.stride_bytes).unwrap_unchecked();
                std::slice::from_raw_parts_mut(C::ptr(&self.s.data).as_ptr().byte_add(offset_bytes), self.s.width.usize())
            })
        } else {
            None
        }
    }

    #[inline]
    fn as_stride_mut(&mut self) -> Self::StrideMut<'_> {
        unsafe {
            Self::StrideMut {
                s: self.s.borrowed(),
            }
        }
    }

    #[inline]
    fn split_at_col_mut(&mut self, columns: Usize) -> Option<(Self::StrideMut<'_>, Self::StrideMut<'_>)> {
        let (c1, c2) = self.s.split_at_col(columns)?;
        Some((
            Self::StrideMut { s: c1 },
            Self::StrideMut { s: c2 },
        ))
    }

    #[inline]
    fn split_at_row_mut(&mut self, rows: Usize) -> Option<(Self::SliceMut<'_>, Self::SliceMut<'_>)> {
        let (r1, r2) = self.s.split_at_row(rows)?;
        Some((
            Self::SliceMut { s: r1 },
            Self::SliceMut { s: r2 },
        ))
    }

    #[inline]
    fn slice_mut<X1X2, Y1Y2>(&mut self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::StrideMut<'_>> where X1X2: RangeBounds<Usize>, Y1Y2: RangeBounds<Usize> {
        let s = self.s.slice(horizontal, vertical)?;
        Some(Self::StrideMut { s })
    }
}

///////////////////////////

impl<'slice, T, Usize: Size> Slice2D<&'slice [T], Usize> {
    /// Make a 2D slice from a 1D slice.
    ///
    /// Returns `None` if `width * height` is larger than the slice.
    ///
    /// Zero size is allowed. Excess slice elements are ignored.
    ///
    /// The generic `Size` argument allows use of `usize`, `u32`, `u16` or `u8` types
    /// for the dimensions. This can make `Slice2D` smaller, and in some cases
    /// eliminate bounds checks and integer overflow checks.
    #[inline(always)]
    #[track_caller]
    pub fn from_slice(slice: &'slice [T], width: Usize, height: Usize) -> Option<Self> {
        Some(Self {
            s: inner::RawSlice::from_slice(slice, width, height)?,
        })
    }

    /// Make a 2D slice from a 1D slice.
    ///
    /// The height is deduced from the length of the slice.
    /// The height is capped to max value of integer type `S`.
    /// Excess length is ignored.
    #[inline(always)]
    #[track_caller]
    pub fn new_ref(slice: &'slice [T], width: Usize) -> Self {
        let height = if width > Usize::ZERO {
            let h = slice.len() / width.usize();
            h.try_into().unwrap_or(Usize::MAX)
        } else {
            Usize::ZERO
        };

        Self {
            s: inner::RawSlice::from_slice(slice, width, height).unwrap(),
        }
    }
}

impl<C: Container, Usize: Size> Slice2D<C, Usize> {
    /// Make a 2D slice from a 1D slice.
    ///
    /// The height is deduced from the length of the slice.
    /// The height is capped to max value of integer type `Usize`.
    /// Excess length is ignored.
    #[inline(always)]
    #[track_caller]
    pub fn new(container: C, width: Usize, height: Usize) -> Self {

        Self {
            s: inner::RawSlice::new(container, width, height).unwrap(),
        }
    }

    /// Direct access to the underlying buffer.
    ///
    /// The buffer is exactly `width * height` long.
    ///
    /// Elements are at `buf()[x + y * width]`.
    #[must_use]
    fn buf(&self) -> &[C::Item] {
        // returning &'container would block future use of 'static
        unsafe {
            NonNull::slice_from_raw_parts(C::ptr(&self.s.data), self.s.area()).as_ref()
        }
    }
}

impl<'slice, T, Usize: Size> Slice2DMut<&'slice mut [T], Usize> {
    /// Make a mutable 2D slice from a 1D slice.
    ///
    /// Returns `None` if `width * height` is larger than the slice.
    ///
    /// Zero size is allowed. Excess slice elements are ignored.
    ///
    /// The generic `Size` argument allows use of `usize`, `u32`, `u16` or `u8` types
    /// for the dimensions. This can make `Slice2D` smaller, and in some cases
    /// eliminate bounds checks and integer overflow checks.
    #[inline(always)]
    #[track_caller]
    pub fn from_slice_mut(slice: &'slice mut [T], width: Usize, height: Usize) -> Option<Self> {
        Some(Self {
            s: inner::RawSlice::from_slice_mut(slice, width, height)?,
        })
    }

    /// Make a mutable 2D slice from a 1D slice.
    ///
    /// The height is deduced from the length of the slice. Excess length is ignored.
    #[inline(always)]
    #[track_caller]
    pub fn new_mut(slice: &'slice mut [T], width: Usize) -> Self {
        let height = if width > Usize::ZERO {
            let h = slice.len() / width.usize();
            h.try_into().unwrap_or(Usize::MAX)
        } else {
            Usize::ZERO
        };

        Self {
            s: inner::RawSlice::from_slice_mut(slice, width, height).unwrap(),
        }
    }
}

impl<C: Container, Usize: Size> Slice2DMut<C, Usize> {
    /// Direct access to the underlying mutable buffer
    ///
    /// The buffer is exactly `width * height` long.
    ///
    /// NB: you will need to make a copy of `width()` and `height()` first if needed,
    /// because getters borrow entire object exclusively.
    ///
    /// Elements are at `buf()[x + y * width]`.
    #[must_use]
    fn buf_mut(&mut self) -> &mut [C::Item] {
        // don't return &'slice mut! Needs exclusive loan of self.
        unsafe {
            NonNull::slice_from_raw_parts(C::ptr(&self.s.data), self.s.area()).as_mut()
        }
    }
}

impl<C: Container, Usize: Size> Stride2D<C, Usize> {
    pub fn new(container: C, width: Usize, height: Usize, stride_bytes: Usize) -> Option<Self> {
        Some(Self {
            s: unsafe {
                inner::RawStride::from_raw_parts_type_erased(C::into_raw(container), width, height, stride_bytes.usize())?
            },
        })
    }
}

impl<'slice, T, Usize: Size> Stride2D<&'slice [T], Usize> {
    /// Make a 2D slice from a non-contiguous 1D slice.
    ///
    /// Rows are `stride` elements apart, but only `width` items wide.
    /// `stride >= width`.
    ///
    /// Zero sized dimensions are allowed. Excess slice elements are ignored.
    ///
    /// The slice must be large enough for all elements, but doesn't have to
    /// cover the whole stride of the last row, only width.
    ///
    /// The generic `Size` argument allows use of `usize`, `u32`, `u16` or `u8` types
    /// for the dimensions. This can make `Slice2D` smaller, and in some cases
    /// eliminate bounds checks and integer overflow checks.
    #[inline(always)]
    #[track_caller]
    pub fn from_slice(slice: &'slice [T], width: Usize, height: Usize, stride: Usize) -> Option<Self> {
        Some(Self {
            s: inner::RawStride::from_slice(slice, width, height, stride)?,
        })
    }

    /// Make a 2D slice from a raw pointer.
    ///
    /// The data is borrowed, and you must ensure the type is given a
    /// correct lifetime.
    ///
    /// There must be `height` valid rows.
    /// In each row there must be `width` valid initialized items.
    /// The items must not be mutated for the lifetime of this slice.
    ///
    /// Rows are `stride_bytes` bytes apart — the first row starts at `slice_start`,
    /// and the second row starts at `slice_start.byte_add(stride_bytes)`.
    ///
    /// `stride_bytes` must be a multiple of `align_of::<C::Item>`, i.e. elements must be
    /// aligned in all rows. For byte-aligned `T` any `stride_bytes` is fine.
    ///
    /// The data between rows (when `stride_bytes` is larger than `width*size_of<C::Item>`)
    /// is never accessed (may be uninitialized or belong to a different slice).
    ///
    /// `height` * `stride_bytes` must be less than `isize::MAX`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn from_raw_parts_ref(slice_start: NonNull<[MaybeUninit<u8>]>, width: Usize, height: Usize, stride_bytes: usize) -> Option<Self> {
        Some(Self {
            s: inner::RawStride::from_raw_parts_type_erased(todo!(), width, height, stride_bytes)?,
        })
    }
}

impl<'slice, T, Usize: Size> Stride2DMut<&'slice mut [T], Usize> {
    /// Make a mutable 2D slice from a non-contiguous 1D slice.
    ///
    /// Rows are `stride` elements apart, but only `width` items wide.
    /// `stride >= width`.
    ///
    /// Zero sized dimensions are allowed. Excess slice elements are ignored.
    ///
    /// The slice must be large enough for all elements, but doesn't have to
    /// cover the whole stride of the last row, only width.
    ///
    /// The generic `Size` argument allows use of `usize`, `u32`, `u16` or `u8` types
    /// for the dimensions. This can make `Slice2D` smaller, and in some cases
    /// eliminate bounds checks and integer overflow checks.
    #[inline(always)]
    #[track_caller]
    pub fn from_slice_mut(slice: &'slice mut [T], width: Usize, height: Usize, stride: Usize) -> Option<Self> {
        Some(Self {
            s: inner::RawStride::from_slice_mut(slice, width, height, stride)?,
        })
    }


    /// Make a 2D mutable slice from a raw pointer.
    ///
    /// The data is borrowed, and you must ensure the type is given a
    /// correct lifetime.
    ///
    /// There must be `height` valid rows.
    /// In each row there must be `width` valid initialized items that will be
    /// accessed only by this slice for the duration of its lifetime.
    ///
    /// Rows are `stride_bytes` bytes apart — the first row starts at `slice_start`,
    /// and the second row starts at `slice_start.byte_add(stride_bytes)`.
    ///
    /// `stride_bytes` must be a multiple of `align_of::<C::Item>`, i.e. elements must be
    /// aligned in all rows. For byte-aligned `T` any `stride_bytes` is fine.
    ///
    /// The data between rows (when `stride_bytes` is larger than `width*size_of<C::Item>`)
    /// is never accessed (may be uninitialized.
    ///
    /// `height` * `stride_bytes` must be less than `isize::MAX`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn from_raw_parts_mut(slice: NonNull<[MaybeUninit<u8>]>, width: Usize, height: Usize, stride_bytes: usize) -> Option<Self> {
        Some(Self {
            s: inner::RawStride::from_raw_parts_type_erased(todo!(), width, height, stride_bytes)?,
        })
    }
}

////////////////////////////////////

impl<C: Container, S: Size> AsRef<[C::Item]> for Slice2D<C, S> {
    #[inline]
    fn as_ref(&self) -> &[C::Item] {
        self.buf()
    }
}

impl<C: Container, S: Size> AsRef<[C::Item]> for Slice2DMut<C, S> {
    #[inline]
    fn as_ref(&self) -> &[C::Item] {
        self.buf()
    }
}

impl<C: Container, S: Size> AsMut<[C::Item]> for Slice2DMut<C, S> {
    #[inline]
    fn as_mut(&mut self) -> &mut [C::Item] {
        self.buf_mut()
    }
}


impl<C: Container, Usize> AsRef<Slice2D<C, Usize>> for Slice2DMut<C, Usize> {
    #[inline]
    fn as_ref(&self) -> &Slice2D<C, Usize> {
        self
    }
}

impl<C: Container, Usize> AsRef<Stride2D<C, Usize>> for Stride2DMut<C, Usize> {
    #[inline]
    fn as_ref(&self) -> &Stride2D<C, Usize> {
        self
    }
}

impl<C: Container, Usize> Deref for Slice2DMut<C, Usize> {
    type Target = Slice2D<C, Usize>;

    #[inline(always)]
    fn deref(&self) -> &Slice2D<C, Usize> {
        // Safety: both are `repr(transparent)` over the same struct type
        unsafe {
            &*std::ptr::from_ref::<Self>(self).cast()
        }
    }
}

impl<C: Container, Usize> Deref for Stride2DMut<C, Usize> {
    type Target = Stride2D<C, Usize>;

    #[inline(always)]
    fn deref(&self) -> &Stride2D<C, Usize> {
        // Safety: both are `repr(transparent)` over the same struct type
        unsafe {
            &*std::ptr::from_ref::<Self>(self).cast()
        }
    }
}

impl<C: Container + Clone, S: Copy> Clone for Slice2D<C, S> where C::RawData: Clone {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
        }
    }
}

impl<C: Container + Copy, S: Copy> Copy for Slice2D<C, S> where C::RawData: Copy {}

impl<C: Container + Clone, S: Copy> Clone for Stride2D<C, S> where C::RawData: Clone {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
        }
    }
}

// impl<C: Container + Copy, S: Copy> Copy for Stride2D<C, S> {
// }

impl<C: Container, S: Size> From<Slice2DMut<C, S>> for Stride2DMut<C, S> {
    #[inline]
    fn from(other: Slice2DMut<C, S>) -> Self {
        Self {
            s: other.s.into(),
        }
    }
}

impl<C: Container, S: Size> From<Slice2D<C, S>> for Stride2D<C, S> {
    #[inline]
    fn from(other: Slice2D<C, S>) -> Self {
        Self {
            s: other.s.into(),
        }
    }
}

macro_rules! impl_index {
    ($t:ident) => {
        impl<C: Container, S: Size> Index<S> for $t<C, S> {
            type Output = [C::Item];

            /// Get nth row. Panics if index is greater than height.
            ///
            /// Note that the index uses the same integer type as
            /// the underlying slice, and is not always `usize`.
            #[inline]
            fn index(&self, idx: S) -> &[C::Item] {
                if let Some(row) = self.rows().nth(idx.usize()) {
                    row
                } else {
                    index_out_of_bounds()
                }
            }
        }
    };
}

impl_index! { Slice2D }
impl_index! { Slice2DMut }
impl_index! { Stride2D }
impl_index! { Stride2DMut }

#[cold]
fn index_out_of_bounds() -> ! {
    panic!("row index out of bounds")
}
