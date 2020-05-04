use std::hash::{Hasher, Hash};
use crate::{ImgRef, ImgVec, ImgRefMut};

impl<'a, T: Hash> Hash for ImgRef<'a, T> {
    #[allow(deprecated)]
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        for row in self.rows() {
            Hash::hash_slice(row, state);
        }
    }
}

impl<'a, T: Hash> Hash for ImgRefMut<'a, T> {
    #[allow(deprecated)]
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<T: Hash> Hash for ImgVec<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<'a, T: PartialEq> PartialEq for ImgRef<'a, T> {
    #[allow(deprecated)]
    #[inline]
    fn eq(&self, other: &ImgRef<T>) -> bool {
        self.width == other.width &&
        self.height == other.height &&
        self.rows().zip(other.rows()).all(|(a,b)| a == b)
    }
}

impl<'a, T: PartialEq> PartialEq for ImgRefMut<'a, T> {
    #[allow(deprecated)]
    #[inline]
    fn eq(&self, other: &ImgRefMut<T>) -> bool {
        self.as_ref().eq(&other.as_ref())
    }
}


impl<T: PartialEq> PartialEq for ImgVec<T> {
    #[allow(deprecated)]
    #[inline(always)]
    fn eq(&self, other: &ImgVec<T>) -> bool {
        self.as_ref().eq(&other.as_ref())
    }
}

impl<'a, T: PartialEq> PartialEq<ImgRef<'a, T>> for ImgVec<T> {
    #[allow(deprecated)]
    #[inline(always)]
    fn eq(&self, other: &ImgRef<'a, T>) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a, T: PartialEq> PartialEq<ImgVec<T>> for ImgRef<'a, T> {
    #[allow(deprecated)]
    #[inline(always)]
    fn eq(&self, other: &ImgVec<T>) -> bool {
        self.eq(&other.as_ref())
    }
}

impl<'a, 'b, T: PartialEq> PartialEq<ImgRef<'a, T>> for ImgRefMut<'b, T> {
    #[allow(deprecated)]
    #[inline(always)]
    fn eq(&self, other: &ImgRef<'a, T>) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a, 'b, T: PartialEq> PartialEq<ImgRefMut<'b, T>> for ImgRef<'a, T> {
    #[allow(deprecated)]
    #[inline(always)]
    fn eq(&self, other: &ImgRefMut<'b, T>) -> bool {
        self.eq(&other.as_ref())
    }
}

impl<'a, T: Eq> Eq for ImgRefMut<'a, T> {
}

impl<'a, T: Eq> Eq for ImgRef<'a, T> {
}

impl<T: Eq> Eq for ImgVec<T> {
}

#[test]
fn test_eq_hash() {
    let mut img1 = ImgVec::new(vec![0u8, 1, 2, 3], 2, 2);
    let img_ne = ImgVec::new(vec![0u8, 1, 2, 3], 4, 1);
    let img2 = ImgVec::new_stride(vec![0u8, 1, 255, 2, 3, 255], 2, 2, 3);
    let mut img3 = ImgVec::new_stride(vec![0u8, 1, 255, 2, 3], 2, 2, 3);

    equiv(&img1, &img2);
    equiv(&img2, &img3);
    equiv(&img1, &img3);

    assert_ne!(img1, img_ne);
    assert_eq!(img1.as_ref(), img2);
    assert_eq!(img2, img3.as_ref());
    equiv(&img1.as_ref(), &img3.as_ref());
    equiv(&img1.as_mut(), &img3.as_mut());
    assert_eq!(img2.as_ref(), img3.as_mut());

    let mut map = HashSet::new();
    img3[(0usize,0usize)] = 100;
    assert_ne!(img1, img3);
    assert!(map.insert(img1));
    assert!(map.insert(img3));
    assert!(map.insert(img_ne));
    assert!(!map.insert(img2));
}

#[cfg(test)]
use std::fmt::Debug;
#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
fn equiv<A>(a: &A, b: &A) where A: Eq + PartialEq + Hash + Debug {
    assert_eq!(a, b);
    let mut map = HashSet::new();
    assert!(map.insert(a));
    assert!(!map.insert(b));
    assert!(!map.insert(a));
    assert!(map.remove(b));
    assert!(map.is_empty());
}
