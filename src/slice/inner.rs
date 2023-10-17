use core::marker::PhantomData;
use core::ops::RangeBounds;
use crate::range::derange;
use crate::size::Size;
use crate::Container;

/// Unsafe due to `T` without a lifetime. Needs to be paired with a `PhantomData`.
pub(crate) struct RawSlice<C: Container, Usize> {
    pub width: Usize,
    pub height: Usize,
    pub data: C::RawData,
    _data_lifetime: PhantomData<C>,
}

/// Unsafe due to `T` without a lifetime. Needs to be paired with a `PhantomData`.
pub(crate) struct RawStride<C: Container, Usize> {
    pub width: Usize,
    /// Unsafe due to alignment
    pub stride_bytes: usize,
    pub height: Usize,
    pub data: C::RawData,
    _data_lifetime: PhantomData<C>,
}

// Minimum required buffer size for given dimenions
fn buffer_len_bytes<T: Sized, S: Size>(width: S, height: S, stride_bytes: usize) -> Option<usize> {
    if height == S::ZERO || width == S::ZERO {
        return Some(0);
    }
    let width_bytes = width.usize().checked_mul(std::mem::size_of::<T>())?;
    if stride_bytes < width_bytes {
        return None;
    }
    // check here would bloat constructors that use element-sized stride
    debug_assert_eq!(0, stride_bytes % std::mem::align_of::<T>());
    Some(stride_bytes.checked_mul(height.usize())? + width_bytes - stride_bytes)
}

impl<T, S: Size> RawSlice<&[T], S> {
    /// Creates a new slice with the given dimensions. Checks that the buffer is large enough.
    #[inline(always)]
    pub(crate) fn from_slice(slice: &[T], width: S, height: S) -> Option<Self> {
        let required_len = width.usize().checked_mul(height.usize())?;
        if slice.len() < required_len { return None; }
        Some(Self {
            width, height,
            data: <&[T] as Container>::into_raw(slice),
            _data_lifetime: PhantomData,
        })
    }
}
impl<T, S: Size> RawSlice<&mut [T], S> {
    /// Creates a new slice with the given dimensions. Checks that the buffer is large enough.
    #[inline(always)]
    pub(crate) fn from_slice_mut(slice: &mut [T], width: S, height: S) -> Option<Self> {
        let required_len = width.usize().checked_mul(height.usize())?;
        if slice.len() < required_len { return None; }
        Some(Self {
            width, height,
            data: <&mut [T] as Container>::into_raw(slice),
            _data_lifetime: PhantomData,
        })
    }
}

impl<C: Container, S: Size> RawSlice<C, S> {
    pub(crate) fn new(container: C, width: S, height: S) -> Option<Self> {
        let required_len = width.usize().checked_mul(height.usize())?;
        // if container.len() < required_len { return None; }
        Some(Self {
            width, height,
            data: C::into_raw(container),
            _data_lifetime: PhantomData,
        })

    }

    pub(crate) unsafe fn borrowed(&self) -> RawSlice<C::Borrowed<'_>, S> {
        RawSlice {
            width: self.width,
            height: self.height,
            data: C::borrow(&self.data, 0),
            _data_lifetime: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn area(&self) -> usize {
        // guaranteed by from_slice_mut
        debug_assert!(self.width.usize().checked_mul(self.height.usize()).unwrap() < isize::MAX as usize);
        self.width.usize() * self.height.usize()
    }

    #[inline]
    pub(crate) fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<RawStride<C::Borrowed<'_>, S>> where X1X2: RangeBounds<S>, Y1Y2: RangeBounds<S> {
        let (top, height) = derange(vertical, self.height)?;
        let (left, width) = derange(horizontal, self.width)?;
        let stride_bytes = self.width.mul_size_of::<C::Item>()?;
        Some(RawStride {
            width,
            height,
            stride_bytes: self.width.mul_size_of::<C::Item>()?,
            data: unsafe {
                // it can't overflow, because is within existing allocation
                let offset_bytes = stride_bytes.checked_mul(top.usize()).unwrap_unchecked()
                    .checked_add(left.usize()).unwrap_unchecked();
                C::borrow(&self.data, offset_bytes)
            },
            _data_lifetime: PhantomData,
        })
    }

    #[inline]
    pub(crate) fn split_at_col(&self, columns: S) -> Option<(RawStride<C::Borrowed<'_>, S>, RawStride<C::Borrowed<'_>, S>)> {
        let new_width = self.width.checked_limited_sub(columns)?;
        Some((
            RawStride {
                width: columns,
                height: self.height,
                stride_bytes: self.width.mul_size_of::<C::Item>()?,
                data: unsafe {
                    C::borrow(&self.data, 0)
                },
                _data_lifetime: PhantomData,
            },
            RawStride {
                width: new_width,
                height: self.height,
                stride_bytes: self.width.mul_size_of::<C::Item>()?,
                data: unsafe {
                    let offset_bytes = columns.mul_size_of::<C::Item>()?;
                    C::borrow(&self.data, offset_bytes)
                },
                _data_lifetime: PhantomData,
            },
        ))
    }

    #[inline]
    pub(crate) fn split_at_row(&self, rows: S) -> Option<(RawSlice<C::Borrowed<'_>, S>, RawSlice<C::Borrowed<'_>, S>)> {
        let Some(remaining_height) = self.height.checked_limited_sub(rows) else {
            return None;
        };
        Some((
            RawSlice {
                width: self.width,
                height: rows,
                data: unsafe {
                    C::borrow(&self.data, 0)
                },
                _data_lifetime: PhantomData,
            },
            RawSlice {
                width: self.width,
                height: remaining_height,
                data: unsafe {
                    // it can't overflow, since it's within existing bounds
                    let byte_offset = rows.usize()
                        .checked_mul(self.width.usize())
                        .and_then(|area| area.checked_mul(std::mem::size_of::<C::Item>()))
                        .unwrap_unchecked();
                    C::borrow(&self.data, byte_offset)
                },
                _data_lifetime: PhantomData,
            },
        ))
    }
}

impl<T, S: Size> RawStride<&[T], S> {
    /// Creates a new stride-slice with the given dimensions. Checks that the buffer is large enough.
    #[inline]
    pub(crate) fn from_slice(slice: &[T], width: S, height: S, stride: S) -> Option<Self> {
        let stride_bytes = stride.mul_size_of::<T>()?;
        let required_len_bytes = buffer_len_bytes::<T, S>(width, height, stride_bytes)?;
        if std::mem::size_of_val(slice) < required_len_bytes { return None; }
        Some(Self {
            data: <&[T] as Container>::into_raw(slice),
            width, height, stride_bytes,
            _data_lifetime: PhantomData,
        })
    }
}

impl<T, S: Size> RawStride<&mut [T], S> {
    /// Creates a new stride-slice with the given dimensions. Checks that the buffer is large enough.
    #[inline]
    pub(crate) fn from_slice_mut(slice: &mut [T], width: S, height: S, stride: S) -> Option<Self> {
        let stride_bytes = stride.mul_size_of::<T>()?;
        let required_len_bytes = buffer_len_bytes::<T, S>(width, height, stride_bytes)?;
        if std::mem::size_of_val(slice) < required_len_bytes { return None; }
        Some(Self {
            data: <&mut [T] as Container>::into_raw(slice),
            width, height, stride_bytes,
            _data_lifetime: PhantomData,
        })
    }
}

impl<C: Container, S: Size> RawStride<C, S> {
    #[inline]
    pub(crate) unsafe fn from_raw_parts_type_erased(data: C::RawData, width: S, height: S, stride_bytes: usize) -> Option<Self> {
        if stride_bytes % std::mem::align_of::<C::Item>() != 0 {
            return None;
        }
        // TODO: can this be checked?
        let _ = buffer_len_bytes::<C::Item, S>(width, height, stride_bytes)?;
        Some(Self {
            data,
            width, height, stride_bytes,
            _data_lifetime: PhantomData,
        })
    }
}

impl<C: Container, S: Size> RawStride<C, S> {
    pub(crate) unsafe fn borrowed(&self) -> RawStride<C::Borrowed<'_>, S> {
        RawStride {
            width: self.width,
            height: self.height,
            stride_bytes: self.stride_bytes,
            data: C::borrow(&self.data, 0),
            _data_lifetime: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn slice<X1X2, Y1Y2>(&self, horizontal: X1X2, vertical: Y1Y2) -> Option<RawStride<C::Borrowed<'_>, S>> where X1X2: RangeBounds<S>, Y1Y2: RangeBounds<S> {
        let (top, height) = derange(vertical, self.height)?;
        let (left, width) = derange(horizontal, self.width)?;
        Some(RawStride {
            width,
            height,
            stride_bytes: self.width.mul_size_of::<C::Item>()?,
            data: unsafe {
                // it can't overflow, because is within existing allocation
                let offset_bytes = self.stride_bytes.checked_mul(top.usize()).unwrap_unchecked().checked_add(left.usize()).unwrap_unchecked();
                C::borrow(&self.data, offset_bytes)
            },
            _data_lifetime: PhantomData,
        })
    }

    #[inline]
    pub(crate) fn split_at_col(&self, columns: S) -> Option<(RawStride<C::Borrowed<'_>, S>, RawStride<C::Borrowed<'_>, S>)> {
        let new_width = self.width.checked_limited_sub(columns)?;
        Some((
            RawStride {
                width: columns,
                height: self.height,
                stride_bytes: self.stride_bytes,
                data: unsafe {
                    C::borrow(&self.data, 0)
                },
                _data_lifetime: PhantomData,
            },
            RawStride {
                width: new_width,
                height: self.height,
                stride_bytes: self.stride_bytes,
                data: unsafe {
                    // TODO: can't overflow, since it's < width
                    let columns_bytes = columns.mul_size_of::<C::Item>()?;
                    C::borrow(&self.data, columns_bytes)
                },
                _data_lifetime: PhantomData,
            },
        ))
    }

    #[inline]
    pub(crate) fn split_at_row(&self, rows: S) -> Option<(RawStride<C::Borrowed<'_>, S>, RawStride<C::Borrowed<'_>, S>)> {
        let new_height = self.height.checked_limited_sub(rows)?;
        Some((
            RawStride {
                width: self.width,
                stride_bytes: self.stride_bytes,
                height: rows,
                data: unsafe {
                    C::borrow(&self.data, 0)
                },
                _data_lifetime: PhantomData,
            },
            RawStride {
                width: self.width,
                stride_bytes: self.stride_bytes,
                height: new_height,
                data: unsafe {
                        C::borrow(&self.data, if new_height > S::ZERO {
                        // it's less than the full size, so it can't overflow
                        rows.usize() * self.stride_bytes
                    } else {
                        // height*stride may exceed buffer, because the last row may be width wide, not stride wide.
                        0
                    })
                },
                _data_lifetime: PhantomData,
            },
        ))
    }
}

impl<C: Container, S: Size> From<RawSlice<C, S>> for RawStride<C, S> {
    #[inline]
    fn from(s: RawSlice<C, S>) -> Self {
        Self {
            width: s.width,
            stride_bytes: s.width.mul_size_of::<C::Item>().unwrap(), // TODO: should this be a contract?
            height: s.height,
            data: s.data,
            _data_lifetime: PhantomData,
        }
    }
}

impl<C: Clone + Container, S: Copy> Clone for RawSlice<C, S> where C::RawData: Clone {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            data: self.data.clone(),
            _data_lifetime: PhantomData,
        }
    }
}

impl<C: Copy + Container, S: Copy> Copy for RawSlice<C, S> where C::RawData: Copy {}

impl<C: Container + Clone, S: Copy> Clone for RawStride<C, S> where C::RawData: Clone {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            stride_bytes: self.stride_bytes,
            data: self.data.clone(),
            _data_lifetime: PhantomData,
        }
    }
}

impl<C: Copy + Container, S: Copy> Copy for RawStride<C, S> where C::RawData: Copy {}

#[test]
fn buflen_1() {
    assert_eq!(buffer_len_bytes::<u8, _>(254u8, 253, 255), Some(252*255+254));
    assert_eq!(buffer_len_bytes::<u8, _>(25u16, 34, 50), Some(33*50+25));
    assert_eq!(buffer_len_bytes::<u8, _>(325u32, 1, 325), Some(325));
    assert_eq!(buffer_len_bytes::<u8, _>(325u32, 0, 325), Some(0));
    assert_eq!(buffer_len_bytes::<u8, _>(0u32, 123, 325), Some(0));
    assert_eq!(buffer_len_bytes::<u8, _>(0usize, 123, 0), Some(0));
    assert_eq!(buffer_len_bytes::<u8, _>(0u64, 0, 0), Some(0));

    assert_eq!(buffer_len_bytes::<u8, _>(10u64, 1, 1), None);
    assert_eq!(buffer_len_bytes::<u8, _>(10u8, 1, 9), None);
}

#[test]
fn buflen_4() {
    assert_eq!(buffer_len_bytes::<u32, _>(254u8, 253, 255*4), Some(252*(255*4) + 254*4));
    assert_eq!(buffer_len_bytes::<u32, _>(25u16, 34, 50*4), Some(4*33*50+25*4));
    assert_eq!(buffer_len_bytes::<u32, _>(325u32, 1, 325*4), Some(4*325));
    assert_eq!(buffer_len_bytes::<u32, _>(325u32, 0, 325*4), Some(0));
    assert_eq!(buffer_len_bytes::<u32, _>(0u32, 123, 325*4), Some(0));
    assert_eq!(buffer_len_bytes::<u32, _>(0usize, 123, 0*4), Some(0));
    assert_eq!(buffer_len_bytes::<u32, _>(0u64, 0, 0*4), Some(0));

    assert_eq!(buffer_len_bytes::<u32, _>(10u64, 1, 4), None);
    assert_eq!(buffer_len_bytes::<u32, _>(10u8, 1, 9*4), None);
}

#[test]
fn stride_buf() {
    assert!(RawSlice::from_slice(&[1u8], 1u8, 1).is_some());
    assert!(RawSlice::from_slice(&[1u8], 0u8, 1).is_some());
    assert!(RawSlice::from_slice(&[1u8], 0u8, 1).is_some());
    assert!(RawSlice::from_slice(&[1u8], 0u8, 0).is_some());

    assert!(RawSlice::from_slice(&[1u8], 2u8, 1).is_none());
    assert!(RawSlice::from_slice(&[1u8], 1u8, 2).is_none());
    assert!(RawSlice::<&[u8], u8>::from_slice(&[], 1u8, 1).is_none());

    assert!(RawStride::from_slice(&[1u8], 1u8, 1, 1u8).is_some());
    assert!(RawStride::from_slice(&[1u8], 0u8, 1, 0u8).is_some());
    assert!(RawStride::from_slice(&[1u8], 0u8, 1, 0u8).is_some());
    assert!(RawStride::from_slice(&[1u8], 0u8, 0, 0u8).is_some());
    assert!(RawStride::<&[u8], u8>::from_slice(&[], 0u8, 0, 99).is_some());
    assert!(RawStride::<&[u8], u8>::from_slice(&[], 0u8, 0, 0).is_some());

    assert!(RawStride::from_slice(&[1u8], 2u8, 1, 2u8).is_none());
    assert!(RawStride::from_slice(&[1u8], 1u8, 2, 1u8).is_none());
    assert!(RawStride::from_slice(&[1u8], 1u8, 2, 100).is_none());
    assert!(RawStride::<&[u8], u8>::from_slice(&[], 1u8, 1, 1).is_none());

    assert_eq!(RawSlice::from_slice(&[1u8; 20], 2u8, 3).unwrap().area(), 6);
}

#[test]
fn splits() {
    let s = RawStride::from_slice(&[1u8; 20], 4u8, 3, 5).unwrap();
    assert_eq!(s.width, 4);
    assert_eq!(s.height, 3);
    assert_eq!(s.stride_bytes, 5);

    let (r1, r2) = s.split_at_row(1).unwrap();
    assert_eq!(r1.width, 4);
    assert_eq!(r1.stride_bytes, 5);
    assert_eq!(r1.height, 1);
    assert_eq!(r2.width, 4);
    assert_eq!(r2.stride_bytes, 5);
    assert_eq!(r2.height, 2);

    let (c1, c2) = s.split_at_col(1).unwrap();
    assert_eq!(c1.height, 3);
    assert_eq!(c2.height, 3);
    assert_eq!(c1.width, 1);
    assert_eq!(c2.width, 3);
    assert_eq!(c1.stride_bytes, 5);
    assert_eq!(c2.stride_bytes, 5);
}

#[test]
fn splits_u32() {
    let s = RawStride::from_slice(&[1u32; 20], 4u8, 3, 5).unwrap();
    assert_eq!(s.width, 4);
    assert_eq!(s.height, 3);
    assert_eq!(s.stride_bytes, 4*5);

    let (r1, r2) = s.split_at_row(1).unwrap();
    assert_eq!(r1.width, 4);
    assert_eq!(r1.stride_bytes, 4*5);
    assert_eq!(r1.height, 1);
    assert_eq!(r2.width, 4);
    assert_eq!(r2.stride_bytes, 4*5);
    assert_eq!(r2.height, 2);

    let (c1, c2) = s.split_at_col(1).unwrap();
    assert_eq!(c1.height, 3);
    assert_eq!(c2.height, 3);
    assert_eq!(c1.width, 1);
    assert_eq!(c2.width, 3);
    assert_eq!(c1.stride_bytes, 4*5);
    assert_eq!(c2.stride_bytes, 4*5);
}
