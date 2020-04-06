//! In graphics code it's very common to pass `width` and `height` along with a `Vec` of pixels,
//! all as separate arguments. This is tedious, and can lead to errors.
//!
//! This crate is a simple struct that adds dimensions to the underlying buffer. This makes it easier to correctly keep track
//! of the image size and allows passing images with just one function argument instead three or four.
//!
//! Additionally, it has a concept of a `stride`, which allows defining sub-regions of images without copying,
//! as well as handling padding (e.g. buffers for video frames may require to be a multiple of 8, regardless of logical image size).
//!
//! For convenience, there are iterators over rows or all pixels of a (sub)image and
//! pixel-based indexing directly with `img[(x,y)]` (where `x`/`y` can be `u32` as well as `usize`).
//!
//! `Img<Container>` type has aliases for common uses:
//!
//! * Owned: `ImgVec<T>` → `Img<Vec<T>>`  (use it in `struct`s and return types)
//! * Reference: `ImgRef<T>` → `Img<&[T]>` (use it in function arguments)
//! * Mutable reference: `ImgRefMut<T>` → `Img<&mut [T]>`
//!
//! It is assumed that the container is [one element per pixel](https://crates.io/crates/rgb/), e.g. `Vec<RGBA>`,
//! and _not_ a `Vec<u8>` where 4 `u8` elements are interpreted as one pixel.
//!
use std::slice;

mod ops;
pub use ops::*;

mod iter;
pub use iter::*;

/// Image owning its pixels.
///
/// A 2D array of pixels. The pixels are oriented top-left first and rows are `stride` pixels wide.
///
/// If size of the `buf` is larger than `width`*`height`, then any excess space is a padding (see `width_padded()`/`height_padded()`).
pub type ImgVec<Pixel> = Img<Vec<Pixel>>;

/// Reference to pixels inside another image.
/// Pass this structure by value (i.e. `ImgRef`, not `&ImgRef`).
///
/// Only `width` of pixels of every `stride` can be modified. The `buf` may be longer than `height`*`stride`, but the extra space should be ignored.
pub type ImgRef<'a, Pixel> = Img<&'a [Pixel]>;

/// Same as `ImgRef`, but mutable
/// Pass this structure by value (i.e. `ImgRef`, not `&ImgRef`).
///
pub type ImgRefMut<'a, Pixel> = Img<&'a mut [Pixel]>;

/// Additional methods that depend on buffer size
///
/// To use these methods you need:
///
/// ```rust
/// use imgref::*;
/// ```
pub trait ImgExt<Pixel> {
    /// Maximum possible width of the data, including the stride.
    ///
    /// This method may panic if the underlying buffer is not at least `height()*stride()` pixels large.
    fn width_padded(&self) -> usize;

    /// Height in number of full strides.
    /// If the underlying buffer is not an even multiple of strides, the last row is ignored.
    ///
    /// This method may panic if the underlying buffer is not at least `height()*stride()` pixels large.
    fn height_padded(&self) -> usize;

    /// Iterate over the entire buffer as rows, including all padding
    ///
    /// Rows will have up to `stride` width, but the last row may be shorter.
    fn rows_padded(&self) -> slice::Chunks<'_, Pixel>;
}

/// Additional methods that depend on buffer size
///
/// To use these methods you need:
///
/// ```rust
/// use imgref::*;
/// ```
pub trait ImgExtMut<Pixel> {
    /// Iterate over the entire buffer as rows, including all padding
    ///
    /// Rows will have up to `stride` width, but the last row may be shorter.
    fn rows_padded_mut(&mut self) -> slice::ChunksMut<'_, Pixel>;
}

/// Basic struct used for both owned (alias `ImgVec`) and borrowed (alias `ImgRef`) image fragments.
///
/// Note: the fields are `pub` only because of borrow checker limitations. Please consider them as read-only.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Img<Container> {
    /// Storage for the pixels. Usually `Vec<Pixel>` or `&[Pixel]`. See `ImgVec` and `ImgRef`.
    ///
    /// Note that future version will make this field private. Use `.rows()` and `.pixels()` iterators where possible, or `buf()`/`buf_mut()`/`into_buf()`.
    #[deprecated(note = "Don't access struct fields directly. Use buf(), buf_mut() or into_buf()")]
    pub buf: Container,

    /// Number of pixels to skip in the container to advance to the next row.
    ///
    /// Note: pixels between `width` and `stride` may not be usable, and may not even exist in the last row.
    #[deprecated(note = "Don't access struct fields directly. Use stride()")]
    pub stride: usize,
    /// Width of the image in pixels.
    ///
    /// Note that this isn't same as the width of the row in the `buf`, see `stride`
    #[deprecated(note = "Don't access struct fields directly. Use width()")]
    pub width: u32,
    /// Height of the image in pixels.
    #[deprecated(note = "Don't access struct fields directly. Use height()")]
    pub height: u32,
}

impl<Container> Img<Container> {
     /// Width of the image in pixels.
    ///
    /// Note that this isn't same as the width of the row in image data, see `stride()`
    #[inline(always)]
    #[allow(deprecated)]
    pub fn width(&self) -> usize {self.width as usize}

    /// Height of the image in pixels.
    #[inline(always)]
    #[allow(deprecated)]
    pub fn height(&self) -> usize {self.height as usize}

    /// Number of pixels to skip in the container to advance to the next row.
    ///
    /// Note the last row may have fewer pixels than the stride.
    #[inline(always)]
    #[allow(deprecated)]
    pub fn stride(&self) -> usize {self.stride}

    /// Immutable reference to the pixel storage.
    #[inline(always)]
    #[allow(deprecated)]
    pub fn buf(&self) -> &Container {&self.buf}

    /// Mutable reference to the pixel storage.
    #[inline(always)]
    #[allow(deprecated)]
    pub fn buf_mut(&mut self) -> &mut Container {&mut self.buf}

    /// Get the pixel storage by consuming the image.
    #[inline(always)]
    #[allow(deprecated)]
    pub fn into_buf(self) -> Container {self.buf}

    #[inline]
    pub fn rows_buf<'a, T: 'a>(&self, buf: &'a [T]) -> RowsIter<'a, T> {
        let stride = self.stride();
        let non_padded = &buf[0..buf.len().min(stride * self.height())];
        RowsIter {
            width: self.width(),
            inner: non_padded.chunks(stride),
        }
    }
}

impl<Pixel,Container> ImgExt<Pixel> for Img<Container> where Container: AsRef<[Pixel]> {
    #[inline(always)]
    fn width_padded(&self) -> usize {
        self.stride()
    }

    #[inline(always)]
    fn height_padded(&self) -> usize {
        let len = self.buf().as_ref().len();
        assert_eq!(0, len % self.stride());
        len/self.stride()
    }

    /// Iterate over the entire buffer as rows, including all padding
    ///
    /// Rows will have up to `stride` width, but the last row may be shorter.
    #[inline(always)]
    fn rows_padded(&self) -> slice::Chunks<'_, Pixel> {
        self.buf().as_ref().chunks(self.stride())
    }
}

impl<Pixel,Container> ImgExtMut<Pixel> for Img<Container> where Container: AsMut<[Pixel]> {
    /// Iterate over the entire buffer as rows, including all padding
    ///
    /// Rows will have up to `stride` width, but the last row may be shorter.
    #[inline]
    #[must_use]
    fn rows_padded_mut(&mut self) -> slice::ChunksMut<'_, Pixel> {
        let stride = self.stride();
        self.buf_mut().as_mut().chunks_mut(stride)
    }
}

impl<'a, T> ImgRef<'a, T> {
    /// Make a reference for a part of the image, without copying any pixels.
    #[inline]
    #[must_use]
    pub fn sub_image(&self, left: usize, top: usize, width: usize, height: usize) -> Self {
        assert!(top+height <= self.height());
        assert!(left+width <= self.width());
        let stride = self.stride();
        let start = stride * top + left;
        let full_strides_end = start + stride * height;
        // when left > 0 and height is full, the last line is shorter than the stride
        let end = if self.buf().len() >= full_strides_end {
            full_strides_end
        } else {
            debug_assert!(height > 0);
            let min_strides_len = full_strides_end + width - stride;
            debug_assert!(self.buf().len() >= min_strides_len, "the buffer is too small to fit the subimage");
            // if can't use full buffer, then shrink to min required (last line having exact width)
            min_strides_len
        };
        let buf = &self.buf()[start .. end];
        Self::new_stride(buf, width, height, stride)
    }

    #[inline]
    #[must_use]
    pub fn rows(&self) -> RowsIter<'_, T> {
        self.rows_buf(self.buf())
    }

    /// Deprecated
    ///
    /// Note: it iterates **all** pixels in the underlying buffer, not just limited by width/height.
    #[deprecated(note="Size of this buffer is unpredictable. Use .rows() instead")]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.buf().iter()
    }
}

impl<'a, T> ImgRefMut<'a, T> {
    /// Turn this into immutable reference, and slice a subregion of it
    #[inline]
    #[allow(deprecated)]
    #[must_use]
    pub fn sub_image(&'a mut self, left: usize, top: usize, width: usize, height: usize) -> ImgRef<'a, T> {
        self.as_ref().sub_image(left, top, width, height)
    }

    /// Trim this image without copying.
    /// Note that mutable borrows are exclusive, so it's not possible to have more than
    /// one mutable subimage at a time.
    #[inline]
    #[allow(deprecated)]
    #[must_use]
    pub fn sub_image_mut(&mut self, left: usize, top: usize, width: usize, height: usize) -> ImgRefMut<'_, T> {
        assert!(top+height <= self.height());
        assert!(left+width <= self.width());
        let stride = self.stride();
        let start = stride * top + left;
        let full_strides_end = start + stride * height;
        // when left > 0 and height is full, the last line is shorter than the stride
        let end = if self.buf().len() >= full_strides_end {
            full_strides_end
        } else {
            debug_assert!(height > 0);
            let min_strides_len = full_strides_end + width - stride;
            debug_assert!(self.buf().len() >= min_strides_len, "the buffer is too small to fit the subimage");
            // if can't use full buffer, then shrink to min required (last line having exact width)
            min_strides_len
        };
        let buf = &mut self.buf[start .. end];
        ImgRefMut::new_stride(buf, width, height, stride)
    }

    /// Make mutable reference immutable
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> ImgRef<'_, T> {
        self.new_buf(self.buf().as_ref())
    }
}

impl<'a, T: Copy> ImgRef<'a, T> {
    #[inline]
    #[must_use]
    pub fn pixels(&self) -> PixelsIter<'_, T> {
        PixelsIter::new(*self)
    }
}

impl<'a, T: Copy> ImgRefMut<'a, T> {
    #[inline]
    #[must_use]
    pub fn pixels(&self) -> PixelsIter<'_, T> {
        PixelsIter::new(self.as_ref())
    }

    #[inline]
    #[must_use]
    pub fn pixels_mut(&mut self) -> PixelsIterMut<'_, T> {
        PixelsIterMut::new(self)
    }
}

impl<'a, T: Copy> ImgVec<T> {
    #[inline]
    #[must_use]
    pub fn pixels(&self) -> PixelsIter<'_, T> {
        PixelsIter::new(self.as_ref())
    }

    #[inline]
    #[must_use]
    pub fn pixels_mut(&mut self) -> PixelsIterMut<'_, T> {
        PixelsIterMut::new(&mut self.as_mut())
    }
}

impl<'a, T> ImgRefMut<'a, T> {
    #[inline]
    #[must_use]
    pub fn rows(&self) -> RowsIter<'_, T> {
        self.rows_buf(&self.buf()[..])
    }

    #[inline]
    #[must_use]
    pub fn rows_mut(&mut self) -> RowsIterMut<'_, T> {
        let stride = self.stride();
        let width = self.width();
        let height = self.height();
        let len = self.buf().len();
        let non_padded = &mut self.buf_mut()[0..len.min(stride * height)];
        RowsIterMut {
            width,
            inner: non_padded.chunks_mut(stride),
        }
    }
}

#[deprecated(note = "use .rows() or .pixels() iterators which are more predictable")]
impl<Container> IntoIterator for Img<Container> where Container: IntoIterator {
    type Item = Container::Item;
    type IntoIter = Container::IntoIter;
    fn into_iter(self) -> Container::IntoIter {
        self.into_buf().into_iter()
    }
}

impl<T> ImgVec<T> {
    /// Create a mutable view into a region within the image. See `sub_image()` for read-only views.
    #[allow(deprecated)]
    #[must_use]
    pub fn sub_image_mut(&mut self, left: usize, top: usize, width: usize, height: usize) -> ImgRefMut<'_, T> {
        assert!(top+height <= self.height());
        assert!(left+width <= self.width());
        let start = self.stride * top + left;
        let min_buf_size = if self.height > 0 {self.stride * height + width - self.stride} else {0};
        let buf = &mut self.buf[start .. start + min_buf_size];
        Img::new_stride(buf, width, height, self.stride)
    }

    #[inline]
    #[must_use]
    /// Make a reference for a part of the image, without copying any pixels.
    pub fn sub_image(&self, left: usize, top: usize, width: usize, height: usize) -> ImgRef<'_, T> {
        self.as_ref().sub_image(left, top, width, height)
    }

    /// Make a reference to this image to pass it to functions without giving up ownership
    ///
    /// The reference should be passed by value (`ImgRef`, not `&ImgRef`).
    ///
    /// If you need a mutable reference, see `as_mut()` and `sub_image_mut()`
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> ImgRef<'_, T> {
        self.new_buf(self.buf().as_ref())
    }

    /// Make a mutable reference to the entire image
    ///
    /// The reference should be passed by value (`ImgRefMut`, not `&mut ImgRefMut`).
    ///
    /// See also `sub_image_mut()` and `rows_mut()`
    #[inline]
    pub fn as_mut(&mut self) -> ImgRefMut<'_, T> {
        let width = self.width();
        let height = self.height();
        let stride = self.stride();
        Img::new_stride(self.buf_mut().as_mut(), width, height, stride)
    }

    #[deprecated(note = "Size of this buffer may be unpredictable. Use .rows() instead")]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.buf().iter()
    }

    /// Iterate over rows of the image as slices
    ///
    /// Each slice is guaranteed to be exactly `width` pixels wide.
    #[inline]
    #[must_use]
    pub fn rows(&self) -> RowsIter<'_, T> {
        self.rows_buf(self.buf())
    }

    /// Iterate over rows of the image as mutable slices
    ///
    /// Each slice is guaranteed to be exactly `width` pixels wide.
    #[inline]
    #[must_use]
    pub fn rows_mut(&mut self) -> RowsIterMut<'_, T> {
        let stride = self.stride();
        let width = self.width();
        let height = self.height();
        let len = self.buf().len();
        let non_padded = &mut self.buf_mut()[0..len.min(stride * height)];
        RowsIterMut {
            width,
            inner: non_padded.chunks_mut(stride),
        }
    }
}

impl<Container> Img<Container> {
    /// Same as `new()`, except each row is located `stride` number of pixels after the previous one.
    ///
    /// Stride can be equal to `width` or larger. If it's larger, then pixels between end of previous row and start of the next are considered a padding, and may be ignored.
    ///
    /// The `Container` is usually a `Vec` or a slice.
    #[inline]
    #[allow(deprecated)]
    pub fn new_stride(buf: Container, width: usize, height: usize, stride: usize) -> Self {
        assert!(stride > 0);
        assert!(stride >= width as usize);
        debug_assert!(height < <u32>::max_value() as usize);
        debug_assert!(width < <u32>::max_value() as usize);
        Img {
            buf,
            width: width as u32,
            height: height as u32,
            stride,
        }
    }

    /// Create new image with `Container` (which can be `Vec`, `&[]` or something else) with given `width` and `height` in pixels.
    ///
    /// Assumes the pixels in container are contiguous, layed out row by row with `width` pixels per row and at least `height` rows.
    ///
    /// If the container is larger than `width`×`height` pixels, the extra rows are a considered a padding and may be ignored.
    #[inline]
    pub fn new(buf: Container, width: usize, height: usize) -> Self {
        Self::new_stride(buf, width, height, width)
    }
}

impl<T: Copy> Img<Vec<T>> {
    /// Returns buffer, width, height. Guarantees that the buffer is contiguous,
    /// i.e. it's `width*height` elements long, and `[x + y*width]` addresses each pixel.
    ///
    /// Efficiently performs operation in-place. For other containers use `pixels().collect()`.
    #[allow(deprecated)]
    #[must_use]
    pub fn into_contiguous_buf(mut self) -> (Vec<T>, usize, usize) {
        let width = self.width();
        let height = self.height();
        let stride = self.stride();
        if width != stride {
            unsafe {
                let buf = self.buf.as_mut_ptr();
                for row in 1..height {
                    std::ptr::copy(buf.add(row * stride), buf.add(row * width), width);
                }
            }
        }
        self.buf.truncate(width * height);
        (self.buf, width, height)
    }
}

impl<OldContainer> Img<OldContainer> {
    /// A convenience method for creating an image of the same size and stride, but with a new buffer.
    #[inline]
    pub fn new_buf<NewContainer, OldPixel, NewPixel>(&self, new_buf: NewContainer) -> Img<NewContainer>
        where NewContainer: AsRef<[NewPixel]>, OldContainer: AsRef<[OldPixel]> {
        assert_eq!(self.buf().as_ref().len(), new_buf.as_ref().len());
        Img::new_stride(new_buf, self.width(), self.height(), self.stride())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod with_opinionated_container {
        use super::*;

        struct IDontDeriveAnything;

        #[test]
        fn compiles() {
            let _ = Img::new(IDontDeriveAnything, 1, 1);
        }
    }

    #[test]
    fn with_vec() {
        let bytes = vec![0u8;20];
        let old = Img::new_stride(bytes, 10,2,10);
        let _ = old.new_buf(vec![6u16;20]);
    }

    #[test]
    fn zero() {
        let bytes = vec![0u8];
        let mut img = Img::new_stride(bytes,0,0,1);
        let _ = img.sub_image(0,0,0,0);
        let _ = img.sub_image_mut(0,0,0,0);
        let _ = img.as_ref();
    }

    #[test]
    fn zero_width() {
        let bytes = vec![0u8];
        let mut img = Img::new_stride(bytes,0,1,1);
        let _ = img.sub_image(0,1,0,0);
        let _ = img.sub_image_mut(0,0,0,1);
    }

    #[test]
    fn zero_height() {
        let bytes = vec![0u8];
        let mut img = Img::new_stride(bytes,1,0,1);
        let _ = img.sub_image(1,0,0,0);
        let _ = img.sub_image_mut(0,0,1,0);
    }

    #[test]
    #[allow(deprecated)]
    fn with_slice() {
        let bytes = vec![0u8;20];
        let _ = Img::new_stride(bytes.as_slice(), 10,2,10);
        let vec = ImgVec::new_stride(bytes, 10,2,10);
        for _ in vec.iter() {}
        assert_eq!(2, vec.rows().count());
        for _ in vec.as_ref().buf().iter() {}
        for _ in vec {}
    }
    #[test]
    fn sub() {
        let img = Img::new_stride(vec![1,2,3,4,
                       5,6,7,8,
                       9], 3, 2, 4);
        assert_eq!(img.buf()[img.stride()], 5);
        assert_eq!(img.buf()[img.stride() + img.width()-1], 7);

        assert_eq!(img.pixels().count(), img.width() * img.height());
        assert_eq!(img.pixels().sum::<i32>(), 24);

        {
        let refimg = img.as_ref();
        let refimg2 = refimg; // Test is Copy

        // sub-image with stride hits end of the buffer
        let s1 = refimg.sub_image(1, 0, refimg.width()-1, refimg.height());
        let _ = s1.sub_image(1, 0, s1.width()-1, s1.height());

        let subimg = refimg.sub_image(1, 1, 2, 1);
        assert_eq!(subimg.pixels().count(), subimg.width() * subimg.height());

        assert_eq!(subimg.buf()[0], 6);
        assert_eq!(subimg.stride(), refimg2.stride());
        assert!(subimg.stride() * subimg.height() + subimg.width() - subimg.stride() <= subimg.buf().len());
        assert_eq!(refimg.buf()[0], 1);
        assert_eq!(1, subimg.rows().count());
        }

        let mut img = img;
        let mut subimg = img.sub_image_mut(1, 1, 2, 1);
        assert_eq!(1, subimg.rows().count());
        assert_eq!(1, subimg.rows_mut().count());
        assert_eq!(subimg.buf()[0], 6);
    }

    #[test]
    fn rows() {
        let img = ImgVec::new_stride(vec![0u8; 10000], 10, 15, 100);
        assert_eq!(img.height(), img.rows().count());
    }

    #[test]
    fn mut_pixels() {
        let mut img = ImgVec::new_stride(vec![0u8; 10000], 10, 15, 100);
        assert_eq!(10*15, img.pixels_mut().count());
        assert_eq!(10*15, img.as_mut().pixels().count());
        assert_eq!(10*15, img.as_mut().pixels_mut().count());
        assert_eq!(10*15, img.as_mut().as_ref().pixels().count());
    }

    #[test]
    fn into_contiguous_buf() {
        let img = ImgVec::new_stride((0..10000).map(|x| x as u8).collect(), 121, 39, 166);
        let pixels: Vec<_> = img.pixels().collect();
        let (buf, w, h) = img.into_contiguous_buf();
        assert_eq!(pixels, buf);
        assert_eq!(121*39, buf.len());
        assert_eq!(10000, buf.capacity());
        assert_eq!(121, w);
        assert_eq!(39, h);

        let img = ImgVec::new((0..55*33).map(|x| x as u8).collect(), 55, 33);
        let pixels: Vec<_> = img.pixels().collect();
        let (buf, ..) = img.into_contiguous_buf();
        assert_eq!(pixels, buf);
    }
}
