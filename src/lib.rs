
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

pub trait ImgExt<Pixel> {
    /// Width of the image in pixels.
    ///
    /// Note that this isn't same as the width of the row in image data, see `stride()`
    fn width(&self) -> usize;

    /// Height of the image in pixels.
    fn height(&self) -> usize;

    /// Number of pixels to skip in the container to advance to the next row.
    /// Note the last row may have fewer pixels than the stride.
    fn stride(&self) -> usize;

    /// Maximum possible width of the data, including the stride.
    ///
    /// This method may panic if the underlying buffer is not at least `height()*stride()` pixels large.
    fn width_padded(&self) -> usize {self.stride()}

    /// Height in number of full strides.
    /// If the underlying buffer is not an even multiple of strides, the last row is ignored.
    ///
    /// This method may panic if the underlying buffer is not at least `height()*stride()` pixels large.
    fn height_padded(&self) -> usize;
}

/// Basic struct used for both owned (alias `ImgVec`) and borrowed (alias `ImgRef`) image fragments.
#[derive(Clone)]
pub struct Img<Container> {
    /// Storage for the pixels. Usually `Vec<Pixel>` or `&[Pixel]`. See `ImgVec` and `ImgRef`.
    pub buf: Container,

    /// Number of pixels to skip in the container to advance to the next row.
    ///
    /// Note: pixels between `width` and `stride` may not be usable, and may not even exist in the last row.
    pub stride: usize,
    /// Width of the image in pixels.
    ///
    /// Note that this isn't same as the width of the row in the `buf`, see `stride`
    pub width: u32,
    /// Height of the image in pixels.
    pub height: u32,
}

impl<Pixel,Container> ImgExt<Pixel> for Img<Container> where Container: AsRef<[Pixel]> {
    #[inline(always)]
    fn width(&self) -> usize {self.width as usize}
    #[inline(always)]
    fn height(&self) -> usize {self.height as usize}
    #[inline(always)]
    fn stride(&self) -> usize {self.stride}

    #[inline(always)]
    fn height_padded(&self) -> usize {
        let len = self.buf.as_ref().len();
        assert_eq!(0, len % self.stride);
        len/self.stride
    }
}

impl<'a, T> Copy for Img<&'a [T]> {
}

impl<'a, T> Img<&'a [T]> {
    #[inline]
    pub fn sub_image(&self, left: usize, top: usize, width: usize, height: usize) -> Self {
        assert!(height > 0);
        assert!(width > 0);
        assert!(top+height <= self.height());
        assert!(left+width <= self.width());
        debug_assert!(self.buf.len() >= self.stride * self.height());
        let start = self.stride * top + left;
        let buf = &self.buf[start .. start + self.stride * height + width - self.stride];
        Self::new_stride(buf, width, height, self.stride)
    }
}

impl<Container> IntoIterator for Img<Container> where Container: IntoIterator {
    type Item = Container::Item;
    type IntoIter = Container::IntoIter;
    fn into_iter(self) -> Container::IntoIter {
        self.buf.into_iter()
    }
}

impl<T> ImgVec<T> {
    /// Create a mutable view into a region within the image. See `sub_image()` for read-only views.
    pub fn sub_image_mut(&mut self, left: usize, top: usize, width: usize, height: usize) -> Img<&mut [T]> {
        assert!(height > 0);
        assert!(width > 0);
        assert!(top+height <= self.height());
        assert!(left+width <= self.width());
        let start = self.stride * top + left;
        let buf = &mut self.buf[start .. start + self.stride * height + width - self.stride];
        Img::new_stride(buf, width, height, self.stride)
    }

    #[inline]
    pub fn sub_image(&self, left: usize, top: usize, width: usize, height: usize) -> ImgRef<T> {
        self.as_ref().sub_image(left, top, width, height)
    }

    /// If you need a mutable reference, see `sub_image_mut()`
    #[inline]
    pub fn as_ref(&self) -> ImgRef<T> {
        self.new_buf(self.buf.as_ref())
    }
}

impl<T> Img<T> {
    #[inline]
    pub fn new_stride(buf: T, width: usize, height: usize, stride: usize) -> Self {
        assert!(height > 0);
        assert!(width > 0);
        assert!(stride >= width as usize);
        debug_assert!(height < <u32>::max_value() as usize);
        debug_assert!(width < <u32>::max_value() as usize);
        Img {
            buf: buf,
            width: width as u32,
            height: height as u32,
            stride: stride,
        }
    }

    #[inline]
    pub fn new(buf: T, width: usize, height: usize) -> Self {
        Self::new_stride(buf, width, height, width)
    }
}

impl<OldContainer> Img<OldContainer> {
    #[inline]
    pub fn new_buf<NewContainer, OldPixel, NewPixel>(&self, new_buf: NewContainer) -> Img<NewContainer>
        where NewContainer: AsRef<[NewPixel]>, OldContainer: AsRef<[OldPixel]> {
        assert_eq!(self.buf.as_ref().len(), new_buf.as_ref().len());
        Img::new_stride(new_buf, self.width(), self.height(), self.stride)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn with_vec() {
        let bytes = vec![0u8;20];
        let old = Img::new_stride(bytes, 10,2,10);
        let _ = old.new_buf(vec![6u16;20]);
    }
    #[test]
    fn with_slice() {
        let bytes = vec![0u8;20];
        let _ = Img::new_stride(bytes.as_slice(), 10,2,10);
        let vec = ImgVec::new_stride(bytes, 10,2,10);
        for _ in vec {}
    }
    #[test]
    fn sub() {
        let img = Img::new_stride(vec![1,2,3,4,
                       5,6,7,8,
                       9], 3, 2, 4);
        assert_eq!(img.buf[img.stride], 5);
        assert_eq!(img.buf[img.stride + img.width()-1], 7);

        {
        let refimg = img.as_ref();
        let refimg2 = refimg; // Test is Copy
        let subimg = refimg.sub_image(1, 1, 2, 1);
        assert_eq!(subimg.buf[0], 6);
        assert_eq!(subimg.stride, refimg2.stride);
        assert!(subimg.stride() * subimg.height() + subimg.width() - subimg.stride <= subimg.buf.len());
        assert_eq!(refimg.buf[0], 1);
        }

        let mut img = img;
        let subimg = img.sub_image_mut(1, 1, 2, 1);
        assert_eq!(subimg.buf[0], 6);
    }
}
