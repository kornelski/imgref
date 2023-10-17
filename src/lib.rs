use core::ops::RangeBounds;
use crate::size::Size;
use std::borrow::Cow;

pub mod size;
pub mod iter;

mod range;
mod slice;
mod container;

pub trait TypeEq<T, U> {}
impl<T> TypeEq<T, T> for T {}

pub use container::Container;

pub type ImgRef<'slice, T> = Stride2D<&'slice [T]>;
pub type ImgRefMut<'slice, T> = Stride2DMut<&'slice mut [T]>;
pub type ImgVec<T> = Stride2D<Vec<T>>;

pub use slice::{Slice2D, Slice2DMut, Stride2D, Stride2DMut};

/// A generic reference to a 2D slice
pub trait Img<C: Container, S: Size = usize> {
    type ItemIter<'slice>: Iterator<Item = &'slice C::Item> + ExactSizeIterator where Self: 'slice, C::Item: 'slice;
    type RowsIter<'slice>: Iterator<Item = &'slice [C::Item]> + ExactSizeIterator + DoubleEndedIterator where Self: 'slice, C::Item: 'slice;
    type Slice<'slice> where Self: 'slice;
    type Stride<'slice> where Self: 'slice;

    /// Number of items per row.
    #[must_use]
    fn width(&self) -> usize;

    /// Number of rows
    #[must_use]
    fn height(&self) -> usize;

    /// Number of items, width * height
    #[must_use]
    fn area(&self) -> usize;

    /// Iterate over every element in this slice, by reference.
    ///
    /// The iterator is width * height items long,
    /// and iterates from the start of the first row.
    ///
    /// Padding (when stride > width) is skipped over.
    #[must_use]
    fn iter(&self) -> Self::ItemIter<'_>;

    /// Iterate over every element in this slice by value.
    ///
    /// The iterator is width * height items long,
    /// and iterates from the start of the first row.
    ///
    /// Padding (when stride > width) is skipped over.
    #[inline]
    #[must_use]
    #[doc(alias = "pixels")]
    fn items(&self) -> iter::Copied<Self::ItemIter<'_>> where C::Item: Copy {
        self.iter().copied()
    }

    #[doc(hidden)]
    fn pixels(&self) -> iter::Copied<Self::ItemIter<'_>> where C::Item: Copy {
        self.items()
    }


    /// Iterate over rows of this slice.
    ///
    /// Every row is `width` items long,
    /// and the number of rows equals `height`.
    #[must_use]
    fn rows(&self) -> Self::RowsIter<'_>;

    /// Get nth (0-indexed) row as a slice
    ///
    /// `None` if the index is out of range.
    ///
    /// Indexing with `[n]` is also supported.
    #[must_use]
    fn row(&self, row_index: usize) -> Option<&[C::Item]>;

    /// Get a row without bounds checks. It is *Undefined Behavior* if the `row_index >= height`
    #[must_use]
    unsafe fn row_unchecked(&self, row_index: usize) -> &[C::Item];

    /// Converts to a 2D slice that can be non-contiguous in memory.
    ///
    /// This type is flexible enough that you can use it a concrete type,
    /// instead of a generic `ImgRef`.
    #[must_use]
    fn as_stride(&self) -> Self::Stride<'_>;

    /// Split this slice into two non-overlapping slices vertically.
    ///
    /// The first one being `columns` wide (`0..columns`),
    /// and the second one covering the rest (`columns..`).
    ///
    /// The height stays the same.
    ///
    /// Slices with width 0 are allowed.
    #[must_use]
    fn split_at_col(&self, columns: S) -> Option<(Self::Stride<'_>, Self::Stride<'_>)>;

    /// Split this slice into two non-overlapping slices horizontally.
    ///
    /// The first one being `rows` tall (`0..rows`),
    /// and the second one covering the rest (`rows..`).
    ///
    /// The width stays the same.
    ///
    /// Slices with height 0 are allowed.
    #[must_use]
    fn split_at_row(&self, rows: S) -> Option<(Self::Slice<'_>, Self::Slice<'_>)>;

    /// Make a sub-slice from this slice.
    ///
    /// `horizontal` and `vertical` arguments are ranges, e.g. `0..100`.
    ///
    /// `horizontal` specifies left and right side (the width will be `right - left`),
    /// and `vertical` specifies top and bottom side (the height will be `right - left`).
    ///
    /// Ranges `..` and `0..` mean whole width or height.
    ///
    /// 0 size is allowed.
    ///
    /// Returns `None` if right side is greater than width, or bottom greater than height.
    #[must_use]
    fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::Stride<'_>> where X1X2: RangeBounds<S>, Y1Y2: RangeBounds<S>;

    /// Check if this slice is contiguous in memory,
    /// i.e its stride is the same as its width.
    #[must_use]
    fn is_contiguous(&self) -> bool;

    /// Makes a contiguous buffer from this slice.
    ///
    /// Returns a tuple of `(buffer, width, height)`
    ///
    /// The buffer is exactly width * height items long,
    /// with items laid row by row without any gaps.
    ///
    /// This method may allocate if necessary.
    #[must_use]
    fn to_contiguous_buf(&self) -> (Cow<'_, [C::Item]>, usize, usize) where [C::Item]: ToOwned;
}

/// Mutable 2D slice with exclusive access to its elements.
pub trait ImgMut<C: Container, S: Size> where Self: Img<C, S> {
    type ItemIterMut<'slice> where Self: 'slice;
    type RowsIterMut<'slice> where Self: 'slice;
    type SliceMut<'slice> where Self: 'slice;
    type StrideMut<'slice> where Self: 'slice;

    /// Iterate over every element in this slice, by mutable reference.
    ///
    /// It guarantees to only access items within width/height of this 2D slice.
    #[must_use]
    #[doc(alias = "pixels_mut")]
    fn iter_mut(&mut self) -> Self::ItemIterMut<'_>;

    /// Iterate over rows of this 2D slice, as mutable 1D slices.
    ///
    /// It guarantees to only access rows within width/height of this 2D slice.
    #[must_use]
    fn rows_mut(&mut self) -> Self::RowsIterMut<'_>;

    /// Access nth (0-indexed) row as a mutable slice.
    ///
    /// Returns `None` if the index is out of range.
    #[must_use]
    fn row_mut(&mut self, row_index: usize) -> Option<&mut [C::Item]>;

    /// Converts to a 2D slice that can be non-contiguous in memory.
    ///
    /// The slice guarantees it won't access elements outside of its width/height.
    /// If the stride is greater than width, the elements in between might
    /// belong to other slices, and will be skipped over.
    ///
    /// This type is flexible enough that you can use it a concrete type,
    /// instead of a generic `ImgRefMut`.
    #[must_use]
    fn as_stride_mut(&mut self) -> Self::StrideMut<'_>;

    /// Divides this slice into two non-overlapping slices vertically.
    #[must_use]
    fn split_at_col_mut(&mut self, columns: S) -> Option<(Self::StrideMut<'_>, Self::StrideMut<'_>)>;

    /// Divides this slice into two non-overlapping slices horizontally.
    #[must_use]
    fn split_at_row_mut(&mut self, rows: S) -> Option<(Self::SliceMut<'_>, Self::SliceMut<'_>)>;

    /// If you need more than one sub-slice, then use `split_at_col_mut` and `split_at_row_mut` first.
    ///
    /// The `horizontal` and `vertical` arguments are ranges, e.g. `0..100`.
    ///
    /// Ranges `..` and `0..` mean whole width or height.
    ///
    /// Returns `None` if right side is greater than width, or bottom greater than height.
    #[must_use]
    fn slice_mut<X1X2, Y1Y2>(&mut self, horizontal: X1X2, vertical: Y1Y2) -> Option<Self::StrideMut<'_>> where X1X2: RangeBounds<S>, Y1Y2: RangeBounds<S>;
}
