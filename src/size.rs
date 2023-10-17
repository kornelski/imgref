use core::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize};

mod you_cant_implement_this {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
}

pub trait Size: Copy + Eq + PartialEq + Ord + PartialOrd + TryFrom<usize> + you_cant_implement_this::Sealed + 'static {
    type NonZero;

    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;

    #[must_use]
    fn usize(self) -> usize;

    /// Subtract number of elements
    #[must_use]
    fn checked_limited_sub(self, other: Self) -> Option<Self>;

    /// Add number of elements
    #[must_use]
    fn checked_limited_add(self, other: Self) -> Option<Self>;

    /// TODO: should I just prevent cration of slices that overflow and never check that again?
    /// Multiply by number of bytes (per element) and check if it fits in `isize::MAX`
    #[must_use]
    fn mul_size_of<T: Sized>(self) -> Option<usize>;

    fn non_zero(self) -> Option<Self::NonZero>;
}

impl Size for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn checked_limited_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn checked_limited_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn mul_size_of<T: Sized>(self) -> Option<usize> {
        (self as usize).checked_mul(std::mem::size_of::<T>())
    }

    type NonZero = NonZeroU8;
    fn non_zero(self) -> Option<Self::NonZero> {
        NonZeroU8::new(self)
    }

    #[inline]
    fn usize(self) -> usize { self as usize }
}

impl Size for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn checked_limited_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn checked_limited_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn mul_size_of<T: Sized>(self) -> Option<usize> {
        (self as usize).checked_mul(std::mem::size_of::<T>())
    }

    type NonZero = NonZeroU16;
    fn non_zero(self) -> Option<Self::NonZero> {
        NonZeroU16::new(self)
    }

    #[inline]
    fn usize(self) -> usize { self as usize }
}

impl Size for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn checked_limited_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn checked_limited_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn mul_size_of<T: Sized>(self) -> Option<usize> {
        (self as usize).checked_mul(std::mem::size_of::<T>())
            .filter(|&len| len < isize::MAX as usize)
    }

    type NonZero = NonZeroU32;
    fn non_zero(self) -> Option<Self::NonZero> {
        NonZeroU32::new(self)
    }

    #[inline]
    fn usize(self) -> usize { self as usize }
}

impl Size for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    fn checked_limited_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn checked_limited_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn mul_size_of<T: Sized>(self) -> Option<usize> {
        (self as usize).checked_mul(std::mem::size_of::<T>())
            .filter(|&len| len < isize::MAX as usize)
    }

    type NonZero = NonZeroU64;
    fn non_zero(self) -> Option<Self::NonZero> {
        NonZeroU64::new(self)
    }

    #[inline]
    fn usize(self) -> usize { self as usize }

}

impl Size for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;

    #[inline(always)]
    #[must_use]
    fn checked_limited_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    #[must_use]
    fn checked_limited_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    #[must_use]
    fn mul_size_of<T: Sized>(self) -> Option<usize> {
        self.checked_mul(std::mem::size_of::<T>())
            .filter(|&len| len < isize::MAX as usize)
    }

    type NonZero = NonZeroUsize;
    #[must_use]
    fn non_zero(self) -> Option<Self::NonZero> {
        NonZeroUsize::new(self)
    }

    #[inline]
    #[must_use]
    fn usize(self) -> usize { self }
}
