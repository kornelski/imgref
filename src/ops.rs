use std::ops;
use super::Img;

macro_rules! impl_imgref_index {
    ($container:ty, $index:ty) => {
        impl<'a, Pixel: Copy> ops::Index<($index, $index)> for Img<$container> {
            type Output = Pixel;
            #[inline(always)]
            /// Read a pixel at `(x,y)` location (e.g. px = `img[(x,y)]`)
            ///
            /// Coordinates may be outside `width`/`height` if the buffer has enough padding.
            /// The x coordinate can't exceed `stride`.
            fn index(&self, index: ($index, $index)) -> &Self::Output {
                let stride = self.stride();
                debug_assert_eq!(stride, stride as $index as usize);
                debug_assert!(index.0 < stride as $index);
                &self.buf()[(index.1 * (stride as $index) + index.0) as usize]
            }
        }
    }
}

macro_rules! impl_imgref_index_mut {
    ($container:ty, $index:ty) => {
        impl<'a, Pixel: Copy> ops::IndexMut<($index, $index)> for Img<$container> {
            #[inline(always)]
            /// Write a pixel at `(x,y)` location (e.g. `img[(x,y)] = px`)
            ///
            /// Coordinates may be outside `width`/`height` if the buffer has enough padding.
            /// The x coordinate can't exceed `stride`.
            fn index_mut(&mut self, index: ($index, $index)) -> &mut Self::Output {
                let stride = self.stride();
                debug_assert_eq!(stride, stride as $index as usize);
                debug_assert!(index.0 < stride as $index);
                &mut self.buf_mut()[(index.1 * (stride as $index) + index.0) as usize]
            }
        }
    }
}

impl_imgref_index! {&'a [Pixel], usize}
impl_imgref_index! {&'a [Pixel], u32}
impl_imgref_index! {&'a mut [Pixel], usize}
impl_imgref_index! {&'a mut [Pixel], u32}
impl_imgref_index_mut! {&'a mut [Pixel], usize}
impl_imgref_index_mut! {&'a mut [Pixel], u32}
impl_imgref_index! {Vec<Pixel>, usize}
impl_imgref_index! {Vec<Pixel>, u32}
impl_imgref_index_mut! {Vec<Pixel>, usize}
impl_imgref_index_mut! {Vec<Pixel>, u32}

#[test]
fn index() {
    let mut img = Img::new_stride(vec![1,2,3,4,5,6,7,8], 2, 2, 3);
    assert_eq!(1, img[(0u32,0u32)]);
    assert_eq!(2, img.as_ref()[(1usize,0usize)]);
    assert_eq!(3, img.as_ref()[(2u32,0u32)]);
    assert_eq!(4, img[(0usize,1usize)]);
    assert_eq!(8, img[(1usize,2usize)]);
    assert_eq!(5, img.sub_image_mut(1,1,1,1)[(0usize,0usize)]);
}
