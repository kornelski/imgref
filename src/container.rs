use crate::{Stride2D};
use crate::{Slice2D};
use std::{ptr::NonNull, mem::ManuallyDrop};


pub unsafe trait Container {
    type Item: Sized;
    type Borrowed<'tmp>: Container<Item=Self::Item> where Self::Item: 'tmp;
    /// Make sure to give it Drop if needed
    type RawData: Sized;

    fn into_imgref(self, width: usize, height: usize) -> Slice2D<Self, usize> where Self: Sized {
        Slice2D::new(self, width, height)
    }

    fn new_stride(self, width: usize, height: usize, stride: usize) -> Stride2D<Self, usize> where Self: Sized {
        Stride2D::new(self, width, height, stride).unwrap()
    }

    /// This is irreversible
    fn into_raw(self) -> Self::RawData;

    fn ptr(raw: &Self::RawData) -> NonNull<Self::Item>;

    unsafe fn borrow(raw: &Self::RawData, from_byte_offset: usize) -> <Self::Borrowed<'_> as Container>::RawData;
    // fn borrow_mut(raw: &'mut Self::RawData) -> <Self::Borrowed<'_> as Container>::RawData;
}

unsafe impl<'slice, T> Container for &'slice [T] {
    type Item = T;
    type Borrowed<'tmp> = &'tmp [T] where Self::Item: 'tmp;
    type RawData = Shared<T>;

    fn into_raw(self) -> Self::RawData {
        unsafe {
            // this cast is fine, because raw pointers don't really care
            // about their mutability, and NonNull is completely agnostic about it.
            Shared(NonNull::new_unchecked(self.as_ptr().cast_mut()))
        }
    }

    unsafe fn borrow(raw: &Shared<T>, from_byte_offset: usize) -> Shared<T> {
       Shared(NonNull::new_unchecked(raw.0.as_ptr().byte_add(from_byte_offset)))
    }

    fn ptr(raw: &Self::RawData) -> NonNull<Self::Item> {
        raw.0
    }
}

unsafe impl<'slice, T> Container for &'slice mut [T] {
    type Item = T;
    type Borrowed<'tmp> = &'tmp mut [T] where Self::Item: 'tmp;
    type RawData = Unique<T>;

    fn into_raw(self) -> Self::RawData {
        unsafe {
            Unique(NonNull::new_unchecked(self.as_mut_ptr()))
        }
    }

    unsafe fn borrow(raw: &Unique<T>, from_byte_offset: usize) -> Unique<T> {
       Unique(NonNull::new_unchecked(raw.0.as_ptr().byte_add(from_byte_offset)))
    }

    fn ptr(raw: &Self::RawData) -> NonNull<Self::Item> {
        raw.0
    }
}

/// `!Copy`
pub struct Unique<T>(NonNull<T>);

/// Copy (does not copy the elements, only the pointer)
#[derive(Clone, Copy)]
pub struct Shared<T>(NonNull<T>);

unsafe impl<T> Container for Vec<T> where T: Copy {
    type Item = T;
    type Borrowed<'tmp> = &'tmp [T] where Self::Item: 'tmp;
    type RawData = OwnedCapacity<T>;

    fn into_raw(self) -> Self::RawData {
        unsafe {
            // Taking ownership
            let mut vec = ManuallyDrop::new(self);
            let data = NonNull::new_unchecked(vec.as_mut_ptr());
            let capacity = vec.capacity();
            OwnedCapacity(data, capacity)
        }
    }

    unsafe fn borrow(raw: &OwnedCapacity<T>, from_byte_offset: usize) -> Shared<T> {
       Shared(NonNull::new_unchecked(raw.0.as_ptr().byte_add(from_byte_offset)))
    }

    fn ptr(raw: &Self::RawData) -> NonNull<Self::Item> {
        raw.0
    }
}

unsafe impl<T> Container for Box<[T]> where T: Copy {
    type Item = T;
    type Borrowed<'tmp> = &'tmp [T] where Self::Item: 'tmp;
    type RawData = OwnedCapacity<T>;

    fn into_raw(self) -> Self::RawData {
        unsafe {
            // Taking ownership
            let mut boxed = ManuallyDrop::new(self);
            let data = NonNull::new_unchecked(boxed.as_mut_ptr());
            let capacity = boxed.len();
            OwnedCapacity(data, capacity)
        }
    }

    unsafe fn borrow(raw: &OwnedCapacity<T>, from_byte_offset: usize) -> Shared<T> {
       Shared(NonNull::new_unchecked(raw.0.as_ptr().byte_add(from_byte_offset)))
    }

    fn ptr(raw: &Self::RawData) -> NonNull<Self::Item> {
        raw.0
    }
}

pub struct OwnedCapacity<T: Copy>(NonNull<T>, usize);

impl<T: Copy> Drop for OwnedCapacity<T> {
    fn drop(&mut self) {
        unsafe {
            // 0-length vec, because Copy elements don't need drop
            // and we've lost original exact length (width*height may have been smaller than the len)
            // and trying to free uninit memory would be way worse.
            let _ = Vec::from_raw_parts(self.0.as_ptr(), 0, self.1);
        }
    }
}
