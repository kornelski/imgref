use core::ops::Bound;
use core::ops::RangeBounds;
use crate::size::Size;

#[inline]
pub(crate) fn derange<R, S>(r: R, max: S) -> Option<(S, S)> where R: RangeBounds<S>, S: Size {
    let start = match r.start_bound() {
        Bound::Unbounded => S::ZERO,
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start.checked_limited_add(S::ONE)?,
    };
    let len = match r.end_bound() {
        Bound::Unbounded => max.checked_limited_sub(start)?,
        Bound::Excluded(&end) if end <= max => end.checked_limited_sub(start)?,
        Bound::Included(&end) if end < max && start <= end => (end.usize()+1).checked_sub(start.usize())?.try_into().ok()?,
        _ => return None,
    };
    debug_assert!(start.usize() + len.usize() <= max.usize());
    Some((start, len))
}

#[test]
fn ranges() {
    assert_eq!(0u16.checked_limited_sub(u16::ZERO), Some(0));

    assert_eq!(derange(.., 10u64), Some((0, 10)));
    assert_eq!(derange(.., 0u16), Some((0, 0)));
    assert_eq!(derange(..10, 10u16), Some((0, 10)));
    assert_eq!(derange(..5, 10u16), Some((0, 5)));

    assert_eq!(derange(1u8..5, 10), Some((1, 4)));
    assert_eq!(derange((Bound::Excluded(0u8), Bound::Unbounded), 10), Some((1, 9)));
    assert_eq!(derange((Bound::Excluded(8u8), Bound::Included(9)), 10), Some((9, 1)));
    assert_eq!(derange(1u8..5, 5), Some((1, 4)));
    assert_eq!(derange(4u8..5, 10), Some((4, 1)));
    assert_eq!(derange(1u8..5, 4), None);
    assert_eq!(derange(2u8..1, 4), None);
    assert_eq!(derange(5u8..5, 10), Some((5, 0)));
    assert_eq!(derange(255u8..255, 255), Some((255, 0)));
    assert_eq!(derange(254u8..255, 255), Some((254, 1)));
    assert_eq!(derange(0u32..0, 10), Some((0, 0)));

    assert_eq!(derange(11u8..=11, 255), Some((11, 1)));
    assert_eq!(derange(1u8..=4, 10), Some((1, 4)));
    assert_eq!(derange(1u8..=4, 5), Some((1, 4)));
    assert_eq!(derange(4u8..=4, 10), Some((4, 1)));
    assert_eq!(derange(1u8..=4, 4), None);
    assert_eq!(derange(2u8..=0, 4), None);
    assert_eq!(derange(5u8..=4, 10), None);
    assert_eq!(derange(255u8..=255, 255), None);
    assert_eq!(derange(254u8..=254, 255), Some((254, 1)));
    assert_eq!(derange(0u32..=0, 10), Some((0, 1)));
}


// use core::ops::Bound;
// use core::ops::RangeBounds;
// use crate::size::Size;

// /// Unsafe, because it must guarantee `start+len <= max`.
// pub unsafe trait SizeRange<S> where S: Size {
//     // start + len. Return None if exceeds max or empty
//     fn range(&self, max: S) -> Option<(S, S)>;
// }

// unsafe impl<S: Size, R: RangeBounds<S>> SizeRange<S> for R {

// }

// // unsafe impl<S: Size> SizeRange<S> for Range<S> {
// //     #[inline]
// //     #[must_use]
// //     fn range(&self, max: S) -> Option<(S, S)> {
// //         let start = self.start;
// //         if start >= self.end {
// //             return None;
// //         }
// //         if self.end > max {
// //             return None;
// //         }
// //         let len = self.end - self.start;
// //         debug_assert!(start + len <= self.end);
// //         debug_assert!(start.usize() + len.usize() <= self.end.usize());
// //         Some((start, len))
// //     }
// // }

// // unsafe impl<S: Size> SizeRange<S> for RangeFull {
// //     #[inline]
// //     #[must_use]
// //     fn range(&self, max: S) -> Option<(S, S)> {
// //         let zero = S::ZERO;
// //         (max > zero).then_some((zero, max))
// //     }
// // }

// // unsafe impl<S: Size> SizeRange<S> for RangeTo<S> {
// //     #[inline]
// //     #[must_use]
// //     fn range(&self, max: S) -> Option<(S, S)> {
// //         let zero = S::ZERO;
// //         (max > zero).then_some((zero, max))
// //     }
// // }

// // unsafe impl<S: Size> SizeRange<S> for RangeInclusive<S> {
// //     #[inline]
// //     #[must_use]
// //     fn range(&self, max: S) -> Option<(S, S)> {
// //         let start = *self.start();
// //         let end_inclusive = *self.end();
// //         if start > end_inclusive {
// //             return None;
// //         }
// //         if end_inclusive >= max {
// //             return None;
// //         }
// //         let len = end_inclusive - start + S::ONE;
// //         debug_assert!(start.usize() + len.usize() <= end_inclusive.usize() + 1);
// //         Some((start, len))
// //     }
// // }

// #[test]
// fn ranges() {
//     assert_eq!((..).range(10u64), Some((0, 10)));
//     assert_eq!((..).range(0u16), None);
//     assert_eq!((..5).range(10u16), None);

//     assert_eq!((1u8..5).range(10), Some((1, 4)));
//     assert_eq!((1u8..5).range(5), Some((1, 4)));
//     assert_eq!((4u8..5).range(10), Some((4, 1)));
//     assert_eq!((1u8..5).range(4), None);
//     assert_eq!((2u8..1).range(4), None);
//     assert_eq!((5u8..5).range(10), None);
//     assert_eq!((255u8..255).range(255), None);
//     assert_eq!((254u8..255).range(255), Some((254, 1)));
//     assert_eq!((0u32..0).range(10), None);

//     assert_eq!((1u8..=4).range(10), Some((1, 4)));
//     assert_eq!((1u8..=4).range(5), Some((1, 4)));
//     assert_eq!((4u8..=4).range(10), Some((4, 1)));
//     assert_eq!((1u8..=4).range(4), None);
//     assert_eq!((2u8..=0).range(4), None);
//     assert_eq!((5u8..=4).range(10), None);
//     assert_eq!((255u8..=255).range(255), None);
//     assert_eq!((254u8..=254).range(255), Some((254, 1)));
//     assert_eq!((0u32..=0).range(10), Some((0, 1)));
// }
